//! System tray icon with state-based updates.
//!
//! Manages a system tray icon with three states (Idle, Recording, Processing)
//! and a context menu for Settings and Exit.

use crate::{AppError, AppResult, TrayIconState};

use std::panic::Location;

use error_location::ErrorLocation;
use tracing::{info, instrument};
use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// System tray icon manager.
pub struct TrayManager {
    tray_icon: TrayIcon,
    settings_item_id: MenuId,
    exit_item_id: MenuId,
}

impl TrayManager {
    /// Create a new tray manager with initial state.
    #[track_caller]
    #[instrument]
    pub fn new() -> AppResult<Self> {
        let menu = Menu::new();

        let settings_item = MenuItem::new("Settings", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        let settings_id = settings_item.id().clone();
        let exit_id = exit_item.id().clone();

        menu.append(&settings_item)
            .map_err(|e| AppError::ConfigError {
                reason: format!("Failed to add settings menu: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        menu.append(&exit_item).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to add exit menu: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        let icon = Self::load_icon(TrayIconState::Idle)?;

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Auto-Scribe - Ready")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()
            .map_err(|e| AppError::ConfigError {
                reason: format!("Failed to create tray icon: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        info!("System tray icon initialized");

        Ok(Self {
            tray_icon,
            settings_item_id: settings_id,
            exit_item_id: exit_id,
        })
    }

    /// Update the tray icon state with new icon and tooltip.
    #[track_caller]
    #[instrument(skip(self))]
    pub fn update_state(&mut self, state: TrayIconState) -> AppResult<()> {
        let (icon, tooltip) = match state {
            TrayIconState::Idle => (Self::load_icon(state)?, "Auto-Scribe - Ready"),
            TrayIconState::Recording => (Self::load_icon(state)?, "Auto-Scribe - Recording..."),
            TrayIconState::Processing => (Self::load_icon(state)?, "Auto-Scribe - Transcribing..."),
        };

        self.tray_icon
            .set_icon(Some(icon))
            .map_err(|e| AppError::ConfigError {
                reason: format!("Failed to update icon: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        self.tray_icon
            .set_tooltip(Some(tooltip))
            .map_err(|e| AppError::ConfigError {
                reason: format!("Failed to update tooltip: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        Ok(())
    }

    /// Load icon from compile-time embedded PNG bytes.
    ///
    /// Icons are embedded via include_bytes! so they work regardless of
    /// install location — no hardcoded filesystem paths.
    #[track_caller]
    fn load_icon(state: TrayIconState) -> AppResult<Icon> {
        let png_bytes: &[u8] = match state {
            TrayIconState::Idle => include_bytes!("../resources/icons/idle.png"),
            TrayIconState::Recording => include_bytes!("../resources/icons/recording.png"),
            TrayIconState::Processing => include_bytes!("../resources/icons/processing.png"),
        };

        let img = image::load_from_memory(png_bytes).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to decode embedded icon: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        let rgba = img.into_rgba8();
        let (width, height) = (rgba.width(), rgba.height());

        Icon::from_rgba(rgba.into_raw(), width, height).map_err(|e| AppError::ConfigError {
            reason: format!("Failed to create icon from RGBA: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })
    }

    /// Get the settings menu item ID.
    pub fn settings_item_id(&self) -> &MenuId {
        &self.settings_item_id
    }

    /// Get the exit menu item ID.
    pub fn exit_item_id(&self) -> &MenuId {
        &self.exit_item_id
    }
}
