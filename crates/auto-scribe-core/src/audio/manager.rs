use crate::{
    audio::AudioCapturer,
    audio::Resampler,
    audio::SttEngine,
    {AudioError, CoreResult},
};

use std::{borrow::Cow, panic::Location, path::Path};

use error_location::ErrorLocation;
use tracing::{debug, info, instrument};

/// Orchestrates the full audio pipeline: capture, resample, transcribe.
///
/// # Memory Footprint
///
/// AudioManager holds all captured audio in memory. At maximum recording
/// duration (5 minutes at 48kHz), the memory footprint is:
///
/// - **Capture buffer**: 48,000 Hz * 300s * 4 bytes = ~58MB
/// - **Resampled copy**: 16,000 Hz * 300s * 4 bytes = ~19MB
/// - **Total peak**: ~77MB (plus Whisper internal allocations)
///
/// This is acceptable for a desktop application with short recordings.
/// For longer recordings or memory-constrained environments, consider
/// chunked streaming transcription (process 30s chunks incrementally).
///
/// # Thread Safety
///
/// AudioManager is NOT thread-safe. It owns its components and should
/// be accessed from a single thread. Use the two-step pattern
/// (`prepare_for_transcription` + `transcribe_prepared`) to minimize
/// lock-holding time when integrating with shared state.
pub struct AudioManager {
    capturer: AudioCapturer,
    resampler: Option<Resampler>,
    engine: SttEngine,
}

impl AudioManager {
    /// Creates a new AudioManager with the specified Whisper model.
    ///
    /// # Errors
    ///
    /// Returns error if no audio device found or model file doesn't exist.
    #[track_caller]
    #[instrument(skip(model_path))]
    pub fn new<P: AsRef<Path>>(model_path: P, use_gpu: bool) -> CoreResult<Self> {
        let capturer = AudioCapturer::new()?;
        let engine = SttEngine::new(model_path, use_gpu)?;

        info!("AudioManager initialized");

        Ok(Self {
            capturer,
            resampler: None,
            engine,
        })
    }

    /// Starts recording audio from the default input device.
    ///
    /// Initializes resampler if device sample rate differs from 16kHz.
    ///
    /// # Errors
    ///
    /// Returns error if audio device cannot be started.
    #[track_caller]
    #[instrument(skip(self))]
    pub fn start_recording(&mut self) -> CoreResult<()> {
        let sample_rate = self.capturer.sample_rate();

        // Create resampler if needed (target is 16kHz for Whisper)
        if sample_rate != 16000 {
            self.resampler = Some(Resampler::new(sample_rate, 16000)?);
            debug!(
                input_rate = sample_rate,
                output_rate = 16000,
                "Resampler configured"
            );
        }

        self.capturer.start()?;

        info!("Recording started");

        Ok(())
    }

    /// Stops recording and returns raw captured audio samples.
    ///
    /// # Errors
    ///
    /// Returns error if no audio was captured.
    #[track_caller]
    #[instrument(skip(self))]
    pub fn stop_recording_raw(&mut self) -> CoreResult<Vec<f32>> {
        let samples = self.capturer.stop()?;

        if samples.is_empty() {
            return Err(AudioError::NoAudioCaptured {
                location: ErrorLocation::from(Location::caller()),
            });
        }

        info!(sample_count = samples.len(), "Recording stopped");

        Ok(samples)
    }

    /// Prepare samples for transcription (resample if needed).
    ///
    /// Returns `Cow::Borrowed` when no resampling is needed (zero-copy),
    /// or `Cow::Owned` with resampled data when sample rate conversion
    /// is required.
    ///
    /// # Two-Step Pattern
    ///
    /// Call this while holding a lock, then release the lock before calling
    /// `transcribe_prepared` which is CPU-intensive (1-10 seconds).
    ///
    /// # Memory
    ///
    /// When resampling: allocates ~19MB for 5 min of 48kHz->16kHz audio.
    /// When not resampling: zero allocation (returns borrowed slice).
    #[track_caller]
    #[instrument(skip(self, samples))]
    pub fn prepare_for_transcription<'a>(
        &mut self, // Changed from &'a mut self
        samples: &'a [f32],
    ) -> CoreResult<Cow<'a, [f32]>> {
        if samples.is_empty() {
            return Err(AudioError::NoAudioCaptured {
                location: ErrorLocation::from(Location::caller()),
            });
        }

        // Resample if needed, otherwise zero-copy borrow
        if let Some(ref mut resampler) = self.resampler {
            let result = resampler.resample(samples)?;
            debug!(
                original_len = samples.len(),
                resampled_len = result.len(),
                "Audio resampled"
            );
            Ok(Cow::Owned(result))
        } else {
            Ok(Cow::Borrowed(samples))
        }
    }

    /// Transcribe pre-processed audio samples.
    ///
    /// **WARNING**: This is CPU-intensive (1-10 seconds) and should NOT
    /// be called while holding a shared lock. Use `prepare_for_transcription`
    /// first, release the lock, then call this.
    #[track_caller]
    #[instrument(skip(self, resampled))]
    pub fn transcribe_prepared(&mut self, resampled: &[f32]) -> CoreResult<String> {
        if resampled.is_empty() {
            return Err(AudioError::NoAudioCaptured {
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let start = std::time::Instant::now();
        let transcription = self.engine.transcribe(resampled)?;
        let duration = start.elapsed();

        info!(
            duration_ms = duration.as_millis(),
            text_len = transcription.len(),
            "Transcription complete"
        );

        Ok(transcription)
    }

    /// Convenience method that resamples and transcribes in one call.
    ///
    /// **WARNING**: This blocks for 1-10 seconds. Do NOT call while holding
    /// a shared lock. Prefer the two-step prepare/transcribe pattern for
    /// integration with shared state.
    #[track_caller]
    #[instrument(skip(self, samples))]
    pub fn transcribe_samples(&mut self, samples: &[f32]) -> CoreResult<String> {
        let resampled = self.prepare_for_transcription(samples)?;
        self.transcribe_prepared(&resampled)
    }

    /// Stops recording, resamples, and transcribes audio in one call.
    ///
    /// Convenience method combining stop, resample, and transcribe.
    ///
    /// # Errors
    ///
    /// Returns error if no audio captured or transcription fails.
    #[track_caller]
    #[instrument(skip(self))]
    pub fn stop_recording(&mut self) -> CoreResult<String> {
        let samples = self.stop_recording_raw()?;
        self.transcribe_samples(&samples)
    }
}
