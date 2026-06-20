use crate::stt::{
    AudioRecording, SttError, SttResult, TARGET_SAMPLE_RATE, WorkerEvent, WorkerRequest,
    validate_model_dir,
};

use async_channel::{Receiver, Sender};
use parakeet_rs::{ExecutionConfig, Nemotron};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction, audioadapter_buffers::owned::InterleavedOwned,
};
use std::{path::PathBuf, thread};

pub(crate) fn spawn_stt_worker(
    model_dir: PathBuf,
    use_gpu: bool,
    event_tx: Sender<WorkerEvent>,
) -> SttResult<Sender<WorkerRequest>> {
    let (request_tx, request_rx) = async_channel::unbounded();
    let error_tx = event_tx.clone();

    let _worker_thread = thread::Builder::new()
        .name("auto-scribe-stt-worker".to_string())
        .spawn(move || {
            if let Err(error) = run_stt_worker(model_dir, use_gpu, request_rx, event_tx) {
                let _ = error_tx.send_blocking(WorkerEvent::Error(error.to_string()));
            }
        })
        .map_err(|error| SttError::speech_to_text(error.to_string()))?;

    Ok(request_tx)
}

fn run_stt_worker(
    model_dir: PathBuf,
    use_gpu: bool,
    request_rx: Receiver<WorkerRequest>,
    event_tx: Sender<WorkerEvent>,
) -> SttResult<()> {
    validate_model_dir(&model_dir)?;
    let mut model = Nemotron::from_pretrained(&model_dir, execution_config(use_gpu))
        .map_err(|error| SttError::speech_to_text(error.to_string()))?;
    send_event(&event_tx, WorkerEvent::Ready)?;

    while let Ok(request) = request_rx.recv_blocking() {
        match request {
            WorkerRequest::Transcribe(recording) => {
                let result =
                    prepare_audio(recording).and_then(|audio| transcribe_audio(&mut model, &audio));
                match result {
                    Ok(transcript) => send_event(&event_tx, WorkerEvent::Transcript(transcript))?,
                    Err(error) => send_event(&event_tx, WorkerEvent::Error(error.to_string()))?,
                }
            }
        }
    }

    Ok(())
}

fn execution_config(use_gpu: bool) -> Option<ExecutionConfig> {
    if use_gpu {
        return Some(ExecutionConfig::new().with_custom_configure(|builder| {
            Ok(builder.with_execution_providers([
                ort::ep::CUDA::default().build().error_on_failure(),
                ort::ep::CPU::default().build().error_on_failure(),
            ])?)
        }));
    }

    None
}

fn send_event(event_tx: &Sender<WorkerEvent>, event: WorkerEvent) -> SttResult<()> {
    event_tx
        .send_blocking(event)
        .map_err(|_| SttError::worker_channel("UI event receiver has disconnected"))
}

fn transcribe_audio(model: &mut Nemotron, audio_16k_mono: &[f32]) -> SttResult<String> {
    model
        .transcribe_audio(audio_16k_mono)
        .map_err(|error| SttError::speech_to_text(error.to_string()))
}

fn prepare_audio(recording: AudioRecording) -> SttResult<Vec<f32>> {
    let (samples, sample_rate) = recording.into_parts();
    let mut audio = if sample_rate == TARGET_SAMPLE_RATE {
        samples
    } else {
        resample_to_16k(samples, sample_rate)?
    };
    sanitize_samples(&mut audio);
    Ok(audio)
}

fn sanitize_samples(samples: &mut [f32]) {
    for sample in samples {
        if sample.is_finite() {
            *sample = sample.clamp(-1.0, 1.0);
        } else {
            *sample = 0.0;
        }
    }
}

fn resample_to_16k(samples: Vec<f32>, source_sample_rate: u32) -> SttResult<Vec<f32>> {
    if source_sample_rate == 0 {
        return Err(SttError::resampling(
            "source sample rate must be greater than zero",
        ));
    }

    if samples.is_empty() {
        return Ok(samples);
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let ratio = f64::from(TARGET_SAMPLE_RATE) / f64::from(source_sample_rate);
    let input_len = samples.len();
    let chunk_size = input_len.min(1024);
    let mut resampler =
        Async::<f32>::new_sinc(ratio, 2.0, &params, chunk_size, 1, FixedAsync::Input)
            .map_err(|error| SttError::resampling(error.to_string()))?;
    let output_len = resampler.process_all_needed_output_len(input_len);
    let input = InterleavedOwned::new_from(samples, 1, input_len)
        .map_err(|error| SttError::resampling(error.to_string()))?;
    let mut output = InterleavedOwned::new(0.0, 1, output_len);
    let (_, frames_out) = resampler
        .process_all_into_buffer(&input, &mut output, input_len, None)
        .map_err(|error| SttError::resampling(error.to_string()))?;
    let mut output_samples = output.take_data();
    output_samples.truncate(frames_out);
    Ok(output_samples)
}
