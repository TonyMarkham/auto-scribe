use auto_scribe_core::AudioError;

use std::{panic::Location, result::Result as StdResult};

use error_location::ErrorLocation;
use thiserror::Error;

/// Application-level errors for the auto-scribe binary.
///
/// All variants include `ErrorLocation` for call-site tracking.
#[derive(Error, Debug)]
pub enum AppError {
    /// Audio subsystem error from auto-scribe-core.
    #[error("Audio error: {source} {location}")]
    Audio {
        /// The underlying audio error.
        #[source]
        source: AudioError,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// Failed to register global hotkey.
    #[error("Hotkey registration failed: {reason} {location}")]
    HotkeyRegistrationFailed {
        /// Human-readable reason for failure.
        reason: String,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// Failed to copy text to clipboard.
    #[error("Failed to copy to clipboard: {reason} {location}")]
    ClipboardError {
        /// Human-readable reason for failure.
        reason: String,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// Failed to simulate keyboard paste.
    #[error("Auto-paste failed: {reason} {location}")]
    AutoPasteFailed {
        /// Human-readable reason for failure.
        reason: String,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// Failed to send message through async channel.
    #[error("Channel send failed: {message} {location}")]
    ChannelSendFailed {
        /// Human-readable error message.
        message: String,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// Configuration loading or saving error.
    #[error("Configuration error: {reason} {location}")]
    ConfigError {
        /// Human-readable reason for failure.
        reason: String,
        /// Location where this error was created.
        location: ErrorLocation,
    },

    /// IO error from filesystem operations.
    #[error("IO error: {source} {location}")]
    IoError {
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
        /// Location where this error was created.
        location: ErrorLocation,
    },
}

// Manual From<AudioError> with location tracking.
// Cannot use #[from] because it does not support extra fields.
impl From<AudioError> for AppError {
    #[track_caller]
    fn from(source: AudioError) -> Self {
        AppError::Audio {
            source,
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<std::io::Error> for AppError {
    #[track_caller]
    fn from(source: std::io::Error) -> Self {
        AppError::IoError {
            source,
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

/// Convenience type alias for Results using `AppError`.
pub type Result<T> = StdResult<T, AppError>;
