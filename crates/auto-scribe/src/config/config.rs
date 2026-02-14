//! Configuration management for auto-scribe.
//!
//! Handles loading and saving TOML configuration files with cross-platform
//! paths, lazy validation, and atomic write operations.

use crate::{
    AppError, AppResult,
    config::{AudioConfig, BehaviourConfig, ServerConfig, WhisperConfig},
};

use std::{fs, io::Write, panic::Location, path::PathBuf};

use crate::config::{DEFAULT_AUTO_PASTE, DEFAULT_PORT};
use directories::ProjectDirs;
use error_location::ErrorLocation;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};

/// Main configuration struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Whisper model configuration.
    pub whisper: WhisperConfig,
    /// Audio device configuration.
    pub audio: AudioConfig,
    /// Application behavior settings.
    pub behavior: BehaviourConfig,
    /// Embedded web server configuration.
    pub server: ServerConfig,
}

impl Config {
    /// Load configuration from disk, creating default if not found.
    ///
    /// Note: This does NOT validate the model path exists. Call
    /// `validate_model_path()` before recording to ensure the model
    /// is available. This allows the app to start and show the settings
    /// UI even if the model hasn't been downloaded yet.
    #[track_caller]
    #[instrument]
    pub fn load() -> AppResult<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).map_err(|e| AppError::ConfigError {
                reason: format!("Failed to read config: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

            let config: Config = toml::from_str(&contents).map_err(|e| AppError::ConfigError {
                reason: format!("Failed to parse config: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

            // Model path is NOT validated here. Validation happens lazily
            // when recording starts (via validate_model_path), so the app
            // can launch and show the settings UI for the user to configure
            // the correct model path.

            info!(config_path = ?config_path, "Configuration loaded");

            Ok(config)
        } else {
            info!("No config found, creating default");
            Self::create_default()
        }
    }

    /// Validate that the Whisper model file exists at the configured path.
    ///
    /// Call this before creating AudioManager, not at config load time.
    /// This allows the app to start and show the settings UI even if the
    /// model has not been downloaded yet.
    #[track_caller]
    #[instrument(skip(self))]
    pub fn validate_model_path(&self) -> AppResult<()> {
        if !self.whisper.model_path.exists() {
            return Err(AppError::ConfigError {
                reason: format!(
                    "Whisper model not found at: {:?}. Download a model or configure the path in Settings.",
                    self.whisper.model_path
                ),
                location: ErrorLocation::from(Location::caller()),
            });
        }
        Ok(())
    }

    /// Save configuration to disk using atomic write pattern.
    ///
    /// Writes to a temporary file first, then renames to prevent corruption
    /// if the process crashes during the write.
    #[track_caller]
    #[instrument]
    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::config_path()?;

        let contents = toml::to_string_pretty(self).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to serialize config: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        // Atomic write: write to temp file then rename
        let temp_path = config_path.with_extension("toml.tmp");

        let mut temp_file = fs::File::create(&temp_path).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to create temp config file: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        temp_file
            .write_all(contents.as_bytes())
            .map_err(|e| AppError::ConfigError {
                reason: format!("Failed to write temp config file: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        temp_file.sync_all().map_err(|e| AppError::ConfigError {
            reason: format!("Failed to sync temp config file: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        fs::rename(&temp_path, &config_path).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to rename temp config to final: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        info!(config_path = ?config_path, "Configuration saved (atomic write)");

        Ok(())
    }

    /// Get the web server URL for opening in browser.
    pub fn server_url(&self) -> String {
        format!("http://localhost:{}", self.server.port)
    }

    #[track_caller]
    fn config_path() -> AppResult<PathBuf> {
        let proj_dirs =
            ProjectDirs::from("com", "auto-scribe", "Auto-Scribe").ok_or_else(|| {
                AppError::ConfigError {
                    reason: "Failed to get config directory".to_string(),
                    location: ErrorLocation::from(Location::caller()),
                }
            })?;

        let config_dir = proj_dirs.config_dir();

        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
            debug!(config_dir = ?config_dir, "Created config directory");
        }

        Ok(config_dir.join("config.toml"))
    }

    #[track_caller]
    fn create_default() -> AppResult<Self> {
        let proj_dirs =
            ProjectDirs::from("com", "auto-scribe", "Auto-Scribe").ok_or_else(|| {
                AppError::ConfigError {
                    reason: "Failed to get project directories".to_string(),
                    location: ErrorLocation::from(Location::caller()),
                }
            })?;

        let data_dir = proj_dirs.data_dir();
        let model_path = data_dir.join("models").join("ggml-base.en.bin");

        let config = Config {
            whisper: WhisperConfig {
                model_path: model_path.clone(),
            },
            audio: AudioConfig {
                selected_device: None,
            },
            behavior: BehaviourConfig {
                auto_paste: DEFAULT_AUTO_PASTE,
            },
            server: ServerConfig { port: DEFAULT_PORT },
        };

        config.save()?;

        warn!(
            model_path = ?model_path,
            "Default config created. Whisper model must be downloaded before recording."
        );

        Ok(config)
    }
}
