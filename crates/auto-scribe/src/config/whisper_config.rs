use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Whisper model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// Path to the Whisper model file (e.g., ggml-base.en.bin).
    pub model_path: PathBuf,

    /// Use GPU for inference if a GPU backend was compiled in (Metal/Vulkan).
    #[serde(default = "default_use_gpu")]
    pub use_gpu: bool,
}

fn default_use_gpu() -> bool {
    true
}
