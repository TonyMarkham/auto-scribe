//! Auto-Scribe: Cross-platform speech-to-text with global hotkey control.

mod app;
mod app_command;
mod config;
mod control_key_guard;
mod error;
mod hotkey_handler;
mod output_handler;
mod recording_state;
#[cfg(test)]
mod tests;
mod tray_command;
mod tray_icon_state;
mod tray_manager;

pub(crate) use {
    app::App,
    app_command::AppCommand,
    control_key_guard::CtrlKeyGuard,
    error::{AppError, Result as AppResult},
    hotkey_handler::HotkeyHandler,
    output_handler::OutputHandler,
    recording_state::RecordingState,
    tray_command::TrayCommand,
    tray_icon_state::TrayIconState,
    tray_manager::TrayManager,
};

use crate::config::Config;

use std::sync::Arc;

use auto_scribe_core::AudioManager;
use global_hotkey::GlobalHotKeyManager;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tokio::sync::{Mutex, mpsc, watch};
use tracing::error;

/// Application entry point.
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("auto_scribe=debug")
        .init();

    let event_loop = EventLoopBuilder::<TrayCommand>::with_user_event().build();
    let tray_proxy = event_loop.create_proxy();

    // TrayManager lives on the main thread - TrayIcon is !Send on all platforms.
    let mut tray_manager = match TrayManager::new() {
        Ok(tm) => tm,
        Err(e) => {
            error!("Failed to create TrayManager: {:?}", e);
            std::process::exit(1);
        }
    };

    // Persists across event loop iterations — dropping it unregisters the hotkey.
    let mut hotkey_manager: Option<GlobalHotKeyManager> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(cmd) => {
                match cmd {
                    TrayCommand::SetState(state) => {
                        if let Err(e) = tray_manager.update_state(state) {
                            error!(error = ?e, "Failed to update tray icon");
                        }
                    }
                    TrayCommand::Shutdown => {
                        *control_flow = ControlFlow::ExitWithCode(0);
                    }
                }
                return;
            }
            Event::NewEvents(tao::event::StartCause::Init) => {
                let config = match Config::load() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to load config: {:?}", e);
                        std::process::exit(1);
                    }
                };

                if let Err(e) = config.validate_model_path() {
                    error!("Model validation failed: {:?}", e);
                    std::process::exit(1);
                }

                let audio_manager = match AudioManager::new(&config.whisper.model_path) {
                    Ok(am) => Arc::new(Mutex::new(am)),
                    Err(e) => {
                        error!("Failed to create AudioManager: {:?}", e);
                        std::process::exit(1);
                    }
                };

                let output_handler = match OutputHandler::new() {
                    Ok(oh) => Arc::new(Mutex::new(oh)),
                    Err(e) => {
                        error!("Failed to create OutputHandler: {:?}", e);
                        std::process::exit(1);
                    }
                };

                #[cfg(target_os = "macos")]
                unsafe {
                    use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};
                    CFRunLoopWakeUp(CFRunLoopGetMain());
                }

                let config = Arc::new(Mutex::new(config));
                let (command_tx, command_rx) = mpsc::channel(32);
                let (shutdown_tx, shutdown_rx) = watch::channel(false);

                // Register hotkey on the main thread — tao's event loop pumps
                // the Windows messages needed for WM_HOTKEY delivery.
                // hotkey_manager is stored in the closure's captured state so it
                // lives for the entire app lifetime.
                let (manager, hotkey_id) = match HotkeyHandler::register_hotkey() {
                    Ok(pair) => pair,
                    Err(e) => {
                        error!("Failed to register hotkey: {:?}", e);
                        std::process::exit(1);
                    }
                };
                hotkey_manager = Some(manager);

                let tray_proxy = tray_proxy.clone();
                let settings_menu_id = tray_manager.settings_item_id().clone();
                let exit_menu_id = tray_manager.exit_item_id().clone();

                // Spawn tokio runtime on separate thread.
                // TrayManager and hotkey_manager stay on the main thread.
                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            error!("Failed to create tokio runtime: {:?}", e);
                            std::process::exit(1);
                        }
                    };

                    rt.block_on(async {
                        let hotkey_handler = HotkeyHandler::new(hotkey_id, command_tx.clone());

                        let app = App {
                            audio_manager,
                            output_handler,
                            tray_proxy,
                            config,
                            command_tx,
                            command_rx,
                            shutdown_tx,
                            settings_menu_id,
                            exit_menu_id,
                        };

                        tokio::join!(
                            async {
                                if let Err(e) = hotkey_handler.run(shutdown_rx).await {
                                    error!(error = ?e, "Hotkey handler error");
                                }
                            },
                            async {
                                if let Err(e) = app.run().await {
                                    error!(error = ?e, "App error");
                                }
                            }
                        );
                    });
                });
            }
            _ => {}
        }

        // Keep hotkey_manager alive in the closure for the app's lifetime.
        let _ = &hotkey_manager;
    });
}
