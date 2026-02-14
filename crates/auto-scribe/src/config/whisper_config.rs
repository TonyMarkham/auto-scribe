use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Whisper model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// Path to the Whisper model file (e.g., ggml-base.en.bin).
    pub model_path: PathBuf,
}
