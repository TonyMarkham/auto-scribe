use crate::config::default_auto_paste;

use serde::{Deserialize, Serialize};

/// Application behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviourConfig {
    /// Whether to automatically paste transcribed text.
    #[serde(default = "default_auto_paste")]
    pub auto_paste: bool,
}
