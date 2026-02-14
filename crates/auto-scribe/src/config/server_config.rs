use crate::config::default_port;

use serde::{Deserialize, Serialize};

/// Embedded web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port for the embedded web server.
    #[serde(default = "default_port")]
    pub port: u16,
}
