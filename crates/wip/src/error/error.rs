use error_location::ErrorLocation;
use std::panic::Location;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WipError {
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
    Stt {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    Unexpected {
        message: String,
        location: ErrorLocation,
    },

    #[error("UI error: {message} {location}")]
    Ui {
        message: String,
        location: ErrorLocation,
    },

    #[error("worker channel error: {message} {location}")]
    WorkerChannel {
        message: String,
        location: ErrorLocation,
    },
}

impl WipError {
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
    pub(crate) fn stt(message: impl Into<String>) -> Self {
        Self::Stt {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[allow(unused)]
    #[track_caller]
    pub(crate) fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn ui(message: impl Into<String>) -> Self {
        Self::Ui {
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

    #[allow(unused)]
    pub fn message(&self) -> &str {
        match self {
            Self::AudioDevice { .. } => "Audio Device Error",
            Self::AudioStream { .. } => "Audio Stream Error",
            Self::ModelPath { .. } => "Model Path Error",
            Self::RecorderState { .. } => "Recorder State Error",
            Self::Resampling { .. } => "Resampling Error",
            Self::Stt { .. } => "Speech-to-Text Error",
            Self::Unexpected { .. } => "Unexpected Error",
            Self::Ui { .. } => "UI Error",
            Self::WorkerChannel { .. } => "Worker Channel Error",
        }
    }

    #[allow(unused)]
    pub fn location(&self) -> ErrorLocation {
        match self {
            Self::AudioDevice { location, .. }
            | Self::AudioStream { location, .. }
            | Self::ModelPath { location, .. }
            | Self::RecorderState { location, .. }
            | Self::Resampling { location, .. }
            | Self::Stt { location, .. }
            | Self::Unexpected { location, .. }
            | Self::Ui { location, .. }
            | Self::WorkerChannel { location, .. } => *location,
        }
    }
}
