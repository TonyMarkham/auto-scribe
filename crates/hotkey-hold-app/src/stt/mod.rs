mod audio_recorder;
mod audio_recording;
mod model_dir;
mod recorder_state;
mod session;
mod snapshot;
mod state;
mod stt_error;
mod stt_result;
mod worker_event;
mod worker_request;
mod worker_runtime;

// ---------------------------------------------------------------------------------------------- //

pub(crate) use audio_recorder::AudioRecorder;
pub(crate) use audio_recording::AudioRecording;
pub(crate) use model_dir::model_dir_from_env_or_default;
pub(crate) use model_dir::validate_model_dir;
pub(crate) use recorder_state::RecorderState;
pub(crate) use session::Session;
pub(crate) use snapshot::Snapshot;
pub(crate) use state::State;
pub(crate) use stt_error::SttError;
pub(crate) use stt_result::SttResult;
pub(crate) use worker_event::WorkerEvent;
pub(crate) use worker_request::WorkerRequest;
pub(crate) use worker_runtime::spawn_stt_worker;

use std::time::Duration;

pub(crate) const MIN_RECORDING_DURATION: Duration = Duration::from_millis(200);
pub(crate) const DEFAULT_MODEL_DIR: &str =
    "submodules/parakeet-rs/nemotron-speech-streaming-en-0.6b";
pub(crate) const TARGET_SAMPLE_RATE: u32 = 16_000;
pub(crate) const MAX_RECORDING_SECONDS: usize = 60;
pub(crate) const REQUIRED_MODEL_FILES: [&str; 4] = [
    "encoder.onnx",
    "encoder.onnx.data",
    "decoder_joint.onnx",
    "tokenizer.model",
];
