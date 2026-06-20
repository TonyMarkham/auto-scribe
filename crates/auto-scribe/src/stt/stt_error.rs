use error_location::ErrorLocation;
use std::panic::Location;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum SttError {
    #[error("audio device error: {message} {location}")]
    AudioDevice {
        message: String,
        location: ErrorLocation,
    },

    #[error("audio stream error: {message} {location}")]
    AudioStream {
        message: String,
        location: ErrorLocation,
    },

    #[error("model path error: {message} {location}")]
    ModelPath {
        message: String,
        location: ErrorLocation,
    },

    #[error("recorder state error: {message} {location}")]
    RecorderState {
        message: String,
        location: ErrorLocation,
    },

    #[error("resampling error: {message} {location}")]
    Resampling {
        message: String,
        location: ErrorLocation,
    },

    #[error("speech-to-text error: {message} {location}")]
    SpeechToText {
        message: String,
        location: ErrorLocation,
    },

    #[error("worker channel error: {message} {location}")]
    WorkerChannel {
        message: String,
        location: ErrorLocation,
    },
}

impl SttError {
    #[track_caller]
    pub(crate) fn audio_device(message: impl Into<String>) -> Self {
        Self::AudioDevice {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn audio_stream(message: impl Into<String>) -> Self {
        Self::AudioStream {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn model_path(message: impl Into<String>) -> Self {
        Self::ModelPath {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn recorder_state(message: impl Into<String>) -> Self {
        Self::RecorderState {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn resampling(message: impl Into<String>) -> Self {
        Self::Resampling {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn speech_to_text(message: impl Into<String>) -> Self {
        Self::SpeechToText {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn worker_channel(message: impl Into<String>) -> Self {
        Self::WorkerChannel {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}
