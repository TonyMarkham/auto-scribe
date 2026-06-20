use crate::stt::{
    AudioRecording, MAX_RECORDING_SECONDS, RecorderState, SttError, SttResult, TARGET_SAMPLE_RATE,
};

use cpal::{
    FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

pub(crate) struct AudioRecorder {
    _stream: Stream,
    state: Arc<Mutex<RecorderState>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub(crate) fn new() -> SttResult<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| SttError::audio_device("no default input device is available"))?;
        let supported_config = select_input_config(&device)?;
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();
        let sample_rate = config.sample_rate;
        let channels = usize::from(config.channels);

        if channels == 0 {
            return Err(SttError::audio_device(
                "input device reported zero audio channels",
            ));
        }

        let max_samples = sample_rate as usize * MAX_RECORDING_SECONDS;
        let state = Arc::new(Mutex::new(RecorderState::new(max_samples)));
        let stream = build_stream(
            &device,
            &config,
            sample_format,
            channels,
            Arc::clone(&state),
        )?;
        stream
            .play()
            .map_err(|error| SttError::audio_stream(error.to_string()))?;

        Ok(Self {
            _stream: stream,
            state,
            sample_rate,
        })
    }

    pub(crate) fn start(&self) -> SttResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| SttError::recorder_state("recording state lock is poisoned"))?;
        state.samples.clear();
        state.recording = true;
        state.clipped = false;
        state.last_stream_error = None;
        Ok(())
    }

    pub(crate) fn stop(&self) -> SttResult<AudioRecording> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| SttError::recorder_state("recording state lock is poisoned"))?;
        state.recording = false;
        let samples = std::mem::take(&mut state.samples);
        let clipped = state.clipped;
        state.clipped = false;

        Ok(AudioRecording {
            samples,
            sample_rate: self.sample_rate,
            clipped,
        })
    }

    pub(crate) fn take_stream_error(&self) -> SttResult<Option<String>> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| SttError::recorder_state("recording state lock is poisoned"))?;
        Ok(state.last_stream_error.take())
    }
}

fn select_input_config(device: &cpal::Device) -> SttResult<cpal::SupportedStreamConfig> {
    let target_rate = TARGET_SAMPLE_RATE;
    let supported_configs = device
        .supported_input_configs()
        .map_err(|error| SttError::audio_device(error.to_string()))?;

    for config_range in supported_configs {
        if config_range.channels() == 1
            && config_range.sample_format() == SampleFormat::F32
            && let Some(config) = config_range.try_with_sample_rate(target_rate)
        {
            return Ok(config);
        }
    }

    device
        .default_input_config()
        .map_err(|error| SttError::audio_device(error.to_string()))
}

fn build_stream(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    channels: usize,
    state: Arc<Mutex<RecorderState>>,
) -> SttResult<Stream> {
    match sample_format {
        SampleFormat::I8 => build_typed_stream::<i8>(device, config, channels, state),
        SampleFormat::I16 => build_typed_stream::<i16>(device, config, channels, state),
        SampleFormat::I24 => build_typed_stream::<cpal::I24>(device, config, channels, state),
        SampleFormat::I32 => build_typed_stream::<i32>(device, config, channels, state),
        SampleFormat::I64 => build_typed_stream::<i64>(device, config, channels, state),
        SampleFormat::U8 => build_typed_stream::<u8>(device, config, channels, state),
        SampleFormat::U16 => build_typed_stream::<u16>(device, config, channels, state),
        SampleFormat::U32 => build_typed_stream::<u32>(device, config, channels, state),
        SampleFormat::U64 => build_typed_stream::<u64>(device, config, channels, state),
        SampleFormat::F32 => build_typed_stream::<f32>(device, config, channels, state),
        SampleFormat::F64 => build_typed_stream::<f64>(device, config, channels, state),
        unsupported => Err(SttError::audio_device(format!(
            "unsupported input sample format: {unsupported}"
        ))),
    }
}

fn build_typed_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    state: Arc<Mutex<RecorderState>>,
) -> SttResult<Stream>
where
    T: Sample + SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let error_state = Arc::clone(&state);
    device
        .build_input_stream(
            *config,
            move |data: &[T], _callback_info| {
                record_input_data(data, channels, &state);
            },
            move |error| {
                if let Ok(mut state) = error_state.lock() {
                    state.last_stream_error = Some(error.to_string());
                }
            },
            None,
        )
        .map_err(|error| SttError::audio_stream(error.to_string()))
}

fn record_input_data<T>(data: &[T], channels: usize, state: &Arc<Mutex<RecorderState>>)
where
    T: Sample,
    f32: FromSample<T>,
{
    let Ok(mut state) = state.lock() else {
        return;
    };

    if !state.recording {
        return;
    }

    for frame in data.chunks(channels) {
        if state.samples.len() >= state.max_samples {
            state.clipped = true;
            return;
        }

        let sample_sum = frame
            .iter()
            .map(|sample| f32::from_sample(*sample))
            .sum::<f32>();
        state.samples.push(sample_sum / frame.len() as f32);
    }
}
