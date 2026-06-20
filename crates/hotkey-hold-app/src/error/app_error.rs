use error_location::ErrorLocation;
use std::fmt::Display;
use std::panic::Location;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("{message} {location}")]
    Operation {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    DesktopMetadata {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    MainWindow {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    HotkeyRuntime {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    SpeechToText {
        message: String,
        location: ErrorLocation,
    },
}

impl AppError {
    #[track_caller]
    pub(crate) fn operation(message: impl Into<String>) -> Self {
        Self::Operation {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn with_context(context: impl Display, error: impl Display) -> Self {
        Self::operation(format!("{context}: {error}"))
    }

    #[track_caller]
    pub(crate) fn desktop_metadata(error: AppError) -> Self {
        Self::DesktopMetadata {
            message: format!("Failed to install desktop metadata: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn main_window(error: AppError) -> Self {
        Self::MainWindow {
            message: format!("Failed to open main window: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn hotkey_runtime(error: AppError) -> Self {
        Self::HotkeyRuntime {
            message: format!("Failed to start hotkey runtime: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn speech_to_text(error: crate::stt::SttError) -> Self {
        Self::SpeechToText {
            message: format!("Failed to start speech-to-text: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Operation { .. } => "Application Error",
            Self::DesktopMetadata { .. } => "Desktop Metadata Error",
            Self::MainWindow { .. } => "Main Window Error",
            Self::HotkeyRuntime { .. } => "Hotkey Runtime Error",
            Self::SpeechToText { .. } => "Speech-to-Text Error",
        }
    }

    pub(crate) fn location(&self) -> ErrorLocation {
        match self {
            Self::Operation { location, .. }
            | Self::DesktopMetadata { location, .. }
            | Self::MainWindow { location, .. }
            | Self::HotkeyRuntime { location, .. }
            | Self::SpeechToText { location, .. } => *location,
        }
    }
}
