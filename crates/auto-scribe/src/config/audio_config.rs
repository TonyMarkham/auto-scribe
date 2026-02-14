use serde::{Deserialize, Serialize};

/// Audio device configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Selected audio device name (None = default device).
    #[serde(default)]
    pub selected_device: Option<String>,
}
