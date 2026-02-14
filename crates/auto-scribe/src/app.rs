use crate::{AppCommand, AppResult, OutputHandler, TrayCommand, TrayIconState, config::Config};

use std::sync::Arc;

use auto_scribe_core::AudioManager;
use tokio::sync::{Mutex, mpsc, watch};
use tracing::{error, info, instrument};
use tray_icon::menu::MenuEvent;
use uuid::Uuid;

/// Main application state.
///
/// Runs on the async runtime thread. Communicates tray icon updates
/// back to the main thread via `tray_tx` because `TrayIcon` is `!Send`
/// and must remain on the UI thread.
pub struct App {
    pub(crate) audio_manager: Arc<Mutex<AudioManager>>,
    pub(crate) output_handler: Arc<Mutex<OutputHandler>>,
    pub(crate) tray_tx: std::sync::mpsc::Sender<TrayCommand>,
    pub(crate) config: Arc<Mutex<Config>>,
    pub(crate) command_tx: mpsc::Sender<AppCommand>,
    pub(crate) command_rx: mpsc::Receiver<AppCommand>,
    pub(crate) shutdown_tx: watch::Sender<bool>,
    pub(crate) settings_menu_id: tray_icon::menu::MenuId,
    pub(crate) exit_menu_id: tray_icon::menu::MenuId,
}

impl App {
    /// Run the main application event loop.
    #[instrument(skip(self))]
    pub(crate) async fn run(mut self) -> AppResult<()> {
        info!("Auto-Scribe starting");

        // Tray event forwarding via single persistent blocking task.
        //
        // MenuEvent::receiver() returns a crossbeam_channel::Receiver which
        // HAS blocking recv() -- zero polling, instant response, one thread.
        //
        // Shutdown: when tray_event_rx is dropped (main loop breaks),
        // tray_event_tx.blocking_send() fails, breaking the blocking loop.
        let (tray_event_tx, mut tray_event_rx) = mpsc::channel(32);
        let tray_handle = tokio::task::spawn_blocking(move || {
            let receiver = MenuEvent::receiver();
            while let Ok(event) = receiver.recv() {
                if tray_event_tx.blocking_send(event).is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                Some(event) = tray_event_rx.recv() => {
                    if let Err(e) = self.handle_tray_event(event).await {
                        error!(error = ?e, "Failed to handle tray event");
                    }
                }

                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        AppCommand::StartRecording { session_id } => {
                            if let Err(e) = self.start_recording(session_id).await {
                                error!(session_id = %session_id, error = ?e, "Failed to start recording");
                            }
                        }
                        AppCommand::StopRecording { session_id } => {
                            self.stop_and_transcribe(session_id).await;
                        }
                        AppCommand::Shutdown => {
                            info!("Shutdown requested");
                            break;
                        }
                    }
                }

                else => {
                    info!("All channels closed, shutting down");
                    break;
                }
            }
        }

        drop(tray_event_rx);

        match tokio::time::timeout(std::time::Duration::from_secs(1), tray_handle).await {
            Ok(Ok(())) => info!("Tray event forwarder stopped cleanly"),
            Ok(Err(e)) => error!(error = ?e, "Tray event forwarder task panicked"),
            Err(_) => info!(
                "Tray event forwarder did not stop within timeout, \
                     will be cleaned up on exit"
            ),
        }

        let _ = self.shutdown_tx.send(true);
        info!("Auto-Scribe shut down successfully");

        Ok(())
    }

    /// Start a recording session.
    #[instrument(skip(self))]
    async fn start_recording(&self, session_id: Uuid) -> AppResult<()> {
        {
            let cfg = self.config.lock().await;
            cfg.validate_model_path()?;
        }

        let mut audio_mgr = self.audio_manager.lock().await;
        audio_mgr.start_recording()?;

        let _ = self
            .tray_tx
            .send(TrayCommand::SetState(TrayIconState::Recording));

        info!(session_id = %session_id, "Recording started");

        Ok(())
    }

    /// Stop recording and start transcription in background.
    #[instrument(skip(self))]
    async fn stop_and_transcribe(&self, session_id: Uuid) {
        let _ = self
            .tray_tx
            .send(TrayCommand::SetState(TrayIconState::Processing));

        let samples = {
            let mut audio_mgr = self.audio_manager.lock().await;
            match audio_mgr.stop_recording_raw() {
                Ok(s) => s,
                Err(e) => {
                    error!(session_id = %session_id, error = ?e, "Failed to stop recording");
                    let _ = self
                        .tray_tx
                        .send(TrayCommand::SetState(TrayIconState::Idle));
                    return;
                }
            }
        };

        let resampled = {
            let mut audio_mgr = self.audio_manager.lock().await;
            match audio_mgr.prepare_for_transcription(&samples) {
                Ok(r) => r.into_owned(),
                Err(e) => {
                    error!(session_id = %session_id, error = ?e, "Failed to resample audio");
                    let _ = self
                        .tray_tx
                        .send(TrayCommand::SetState(TrayIconState::Idle));
                    return;
                }
            }
        };

        let audio_manager = Arc::clone(&self.audio_manager);
        let output_handler = Arc::clone(&self.output_handler);
        let config = Arc::clone(&self.config);
        let tray_tx = self.tray_tx.clone();

        tokio::task::spawn(async move {
            let start = std::time::Instant::now();

            let transcription = {
                let mut audio_mgr = audio_manager.lock().await;
                match audio_mgr.transcribe_prepared(&resampled) {
                    Ok(text) => text,
                    Err(e) => {
                        error!(session_id = %session_id, error = ?e, "Transcription failed");
                        let _ = tray_tx.send(TrayCommand::SetState(TrayIconState::Idle));
                        return;
                    }
                }
            };

            let duration = start.elapsed();
            info!(
                session_id = %session_id,
                duration_ms = duration.as_millis(),
                text_len = transcription.len(),
                "Transcription complete"
            );

            let cfg = config.lock().await;
            let auto_paste = cfg.behavior.auto_paste;
            drop(cfg);

            let mut output = output_handler.lock().await;
            if let Err(e) = output.output_text(&transcription, auto_paste).await {
                error!(session_id = %session_id, error = ?e, "Failed to output text");
            }

            // Tray icon back to Idle - this now works because tray_tx is Send
            let _ = tray_tx.send(TrayCommand::SetState(TrayIconState::Idle));
        });
    }

    /// Handle tray menu events.
    #[instrument(skip(self))]
    async fn handle_tray_event(&mut self, event: MenuEvent) -> AppResult<()> {
        let event_id = &event.id;

        if *event_id == self.settings_menu_id {
            let cfg = self.config.lock().await;
            let url = cfg.server_url();
            drop(cfg);
            let _ = open::that(url);
            info!("Opened settings UI");
        } else if *event_id == self.exit_menu_id {
            info!("Exit requested from tray menu");
            let _ = self.tray_tx.send(TrayCommand::Shutdown);
            if let Err(e) = self.command_tx.send(AppCommand::Shutdown).await {
                error!(error = ?e, "Failed to send shutdown command");
            }
        }

        Ok(())
    }
}
