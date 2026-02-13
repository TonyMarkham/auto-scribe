use error_location::ErrorLocation;
use thiserror::Error;

/// Audio processing errors with source location tracking.
#[derive(Error, Debug)]
pub enum AudioError {
    /// No audio input device found.
    #[error("No microphone found {location}")]
    NoMicrophoneFound {
        /// Source location where error occurred.
        location: ErrorLocation,
    },

    /// Whisper model file not found at specified path.
    #[error("Model not found at path: {path:?} {location}")]
    ModelNotFound {
        /// Path to the missing model file.
        path: std::path::PathBuf,
        /// Source location where error occurred.
        location: ErrorLocation,
    },

    /// Transcription process failed.
    #[error("Transcription failed: {source} {location}")]
    TranscriptionFailed {
        /// Underlying error from whisper-rs.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        /// Source location where error occurred.
        location: ErrorLocation,
    },

    /// No audio data captured or provided.
    #[error("No audio captured {location}")]
    NoAudioCaptured {
        /// Source location where error occurred.
        location: ErrorLocation,
    },

    /// Audio device operation failed.
    #[error("Audio device error: {reason} {location}")]
    DeviceError {
        /// Description of the device error.
        reason: String,
        /// Source location where error occurred.
        location: ErrorLocation,
    },

    /// Audio resampling failed.
    #[error("Resampling error: {reason} {location}")]
    ResamplingError {
        /// Description of the resampling error.
        reason: String,
        /// Source location where error occurred.
        location: ErrorLocation,
    },
}

/// Result type alias using [`AudioError`].
pub type Result<T> = std::result::Result<T, AudioError>;
