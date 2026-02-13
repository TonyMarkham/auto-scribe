use crate::{AudioError, CoreResult};

use std::{
    collections::VecDeque,
    panic::Location,
    sync::{
        atomic::{AtomicBool, Ordering},
        {Arc, Mutex},
    },
};

use cpal::{
    Device, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use error_location::ErrorLocation;
use tracing::{debug, error, info, instrument};

/// Maximum samples to buffer (5 minutes at 48kHz mono).
/// Prevents unbounded memory growth during long recordings.
///
/// **Memory footprint at max capacity:**
/// - 48,000 Hz * 60s * 5 min * 4 bytes/f32 = ~58MB
/// - This is a hard upper bound; typical recordings are shorter
pub(crate) const MAX_BUFFER_SAMPLES: usize = 48_000 * 60 * 5;

pub struct AudioCapturer {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    samples: Arc<Mutex<VecDeque<f32>>>,
    /// Signals the audio callback to stop writing. Set to `true` before
    /// dropping the stream to ensure no in-flight callback writes after
    /// the lock is acquired in `stop()`.
    shutdown: Arc<AtomicBool>,
}

impl AudioCapturer {
    #[track_caller]
    #[instrument]
    pub fn new() -> CoreResult<Self> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or(AudioError::NoMicrophoneFound {
                location: ErrorLocation::from(Location::caller()),
            })?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::DeviceError {
                reason: format!("Failed to get config: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        info!(
            device_id = ?device.id(),
            sample_rate = config.sample_rate(),
            channels = config.channels(),
            "AudioCapturer initialized"
        );

        Ok(Self {
            device,
            config: config.into(),
            stream: None,
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_SAMPLES))),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    #[track_caller]
    #[instrument(skip(self))]
    pub fn start(&mut self) -> CoreResult<()> {
        let samples = Arc::clone(&self.samples);
        let shutdown = Arc::clone(&self.shutdown);

        // Reset shutdown flag for new recording session
        self.shutdown.store(false, Ordering::Release);

        // Clear previous samples
        samples
            .lock()
            .map_err(|e| AudioError::DeviceError {
                reason: format!("Failed to lock samples: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?
            .clear();

        let stream = self
            .device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Check shutdown flag before acquiring lock. This provides
                    // explicit synchronization: once stop() sets this flag,
                    // no new samples will be written even if CPAL fires one
                    // more callback before the stream is dropped.
                    if shutdown.load(Ordering::Acquire) {
                        return;
                    }
                    // Recover from lock poison rather than silently dropping audio.
                    // A poisoned mutex means a previous holder panicked, but the
                    // VecDeque data is still valid and usable.
                    let mut buf = samples.lock().unwrap_or_else(|e| {
                        error!("Sample buffer lock poisoned, recovering: {}", e);
                        e.into_inner()
                    });
                    buf.extend(data.iter().copied());
                    // Ring buffer: O(1) amortized drop of oldest samples via VecDeque
                    while buf.len() > MAX_BUFFER_SAMPLES {
                        buf.pop_front();
                    }
                },
                |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioError::DeviceError {
                reason: format!("Failed to build stream: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        stream.play().map_err(|e| AudioError::DeviceError {
            reason: format!("Failed to start stream: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        self.stream = Some(stream);
        info!("Audio capture started");

        Ok(())
    }

    #[track_caller]
    #[instrument(skip(self))]
    pub fn stop(&mut self) -> CoreResult<Vec<f32>> {
        // Signal callback to stop writing BEFORE dropping the stream.
        // This provides defense-in-depth: even if CPAL's Stream::drop()
        // is asynchronous on some backend, the callback will observe this
        // flag and return early, preventing writes after we acquire the lock.
        self.shutdown.store(true, Ordering::Release);

        if let Some(stream) = self.stream.take() {
            drop(stream);
            // Brief yield to ensure any in-flight callback observes the
            // shutdown flag and completes. On most CPAL backends, drop()
            // is synchronous and joins the audio thread, making this
            // redundant — but it costs <5ms and guarantees correctness
            // even if a backend's drop() returns before the final callback.
            std::thread::sleep(std::time::Duration::from_millis(5));
            info!("Audio capture stopped");
        }

        let samples: Vec<f32> = self
            .samples
            .lock()
            .map_err(|e| AudioError::DeviceError {
                reason: format!("Failed to lock samples: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?
            .iter()
            .copied()
            .collect();

        debug!(sample_count = samples.len(), "Captured audio samples");

        Ok(samples)
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }
}
