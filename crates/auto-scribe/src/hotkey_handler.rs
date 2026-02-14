//! Global hotkey handler with recording state machine.
//!
//! Registers CTRL+SHIFT+Space as a global hotkey and manages recording state
//! transitions. Uses async channels to communicate with the main application.

use crate::{AppCommand, AppError, AppResult, RecordingState};

use std::{
    panic::Location,
    sync::Arc,
    time::{Duration, Instant},
};

use error_location::ErrorLocation;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use tokio::sync::{Mutex, mpsc, watch};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Global hotkey handler with recording state machine.
pub struct HotkeyHandler {
    hotkey_id: u32,
    state: Arc<Mutex<RecordingState>>,
    command_tx: mpsc::Sender<AppCommand>,
}

impl HotkeyHandler {
    /// Register CTRL+SHIFT+Space as the global hotkey.
    ///
    /// Must be called on a thread with a message pump (e.g. the main thread
    /// running a `tao`/`winit` event loop) so that `WM_HOTKEY` messages are
    /// dispatched on Windows. The returned [`GlobalHotKeyManager`] must be
    /// kept alive on that thread for the hotkey to remain registered.
    #[track_caller]
    #[instrument]
    pub fn register_hotkey() -> AppResult<(GlobalHotKeyManager, u32)> {
        let manager =
            GlobalHotKeyManager::new().map_err(|e| AppError::HotkeyRegistrationFailed {
                reason: format!("Failed to create manager: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);

        manager
            .register(hotkey)
            .map_err(|e| AppError::HotkeyRegistrationFailed {
                reason: format!("Failed to register CTRL+SHIFT+Space: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        info!(hotkey = "CTRL+SHIFT+Space", "Global hotkey registered");

        Ok((manager, hotkey.id()))
    }

    /// Create a handler for a previously registered hotkey.
    ///
    /// The `hotkey_id` should come from [`register_hotkey`]. This struct is
    /// `Send` and can live on any thread — it only listens on the global
    /// [`GlobalHotKeyEvent`] channel.
    pub fn new(hotkey_id: u32, command_tx: mpsc::Sender<AppCommand>) -> Self {
        Self {
            hotkey_id,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            command_tx,
        }
    }

    /// Run the hotkey handler event loop.
    ///
    /// This method blocks until a shutdown signal is received.
    #[instrument(skip(self))]
    pub async fn run(&self, mut shutdown_rx: watch::Receiver<bool>) -> AppResult<()> {
        let receiver = GlobalHotKeyEvent::receiver().clone();
        let (event_tx, mut event_rx) = mpsc::channel(32);

        // Single persistent blocking task that forwards hotkey events.
        // GlobalHotKeyEvent::receiver() returns a crossbeam_channel::Receiver
        // which has blocking recv() -- zero polling, instant response, one thread.
        //
        // Shutdown: when event_rx is dropped (loop breaks), the next
        // event_tx.blocking_send() fails, breaking the blocking loop.
        // The JoinHandle is awaited with a timeout after the main loop exits.
        let handle = tokio::task::spawn_blocking(move || {
            while let Ok(event) = receiver.recv() {
                if event_tx.blocking_send(event).is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    info!("Hotkey handler shutting down");
                    break;
                }
                Some(event) = event_rx.recv() => {
                    if event.id == self.hotkey_id {
                        self.handle_hotkey_press().await?;
                    }
                }
            }
        }

        // Drop event_rx to unblock the blocking task's next blocking_send().
        // The task will break out of its loop when blocking_send returns Err.
        drop(event_rx);

        // Best-effort join: the blocking task may be stuck in recv() if no
        // hotkey event arrives after shutdown. Use a timeout to avoid hanging.
        // The task is cleaned up by the runtime on process exit regardless.
        match tokio::time::timeout(Duration::from_secs(1), handle).await {
            Ok(Ok(())) => debug!("Hotkey event forwarder stopped cleanly"),
            Ok(Err(e)) => warn!(error = ?e, "Hotkey event forwarder task panicked"),
            Err(_) => debug!(
                "Hotkey event forwarder did not stop within timeout, \
                   will be cleaned up on exit"
            ),
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_hotkey_press(&self) -> AppResult<()> {
        let mut state = self.state.lock().await;

        match *state {
            RecordingState::Idle => {
                let session_id = Uuid::new_v4();

                // Send command FIRST -- if this fails, state remains Idle.
                // This prevents the app from being stuck in Recording state
                // with no command sent (e.g., if the channel is full or closed).
                self.command_tx
                    .send(AppCommand::StartRecording { session_id })
                    .await
                    .map_err(|e| AppError::ChannelSendFailed {
                        message: format!("Failed to send StartRecording: {}", e),
                        location: ErrorLocation::from(Location::caller()),
                    })?;

                // Only update state AFTER command sent successfully
                *state = RecordingState::Recording {
                    started_at: Instant::now(),
                    session_id,
                };

                info!(session_id = %session_id, "Recording started");
            }
            RecordingState::Recording {
                started_at,
                session_id,
            } => {
                let duration = started_at.elapsed();

                // Send command FIRST -- if this fails, state remains Recording.
                // The user can retry by pressing the hotkey again.
                self.command_tx
                    .send(AppCommand::StopRecording { session_id })
                    .await
                    .map_err(|e| AppError::ChannelSendFailed {
                        message: format!("Failed to send StopRecording: {}", e),
                        location: ErrorLocation::from(Location::caller()),
                    })?;

                // Only update state AFTER command sent successfully
                *state = RecordingState::Idle;

                info!(
                    session_id = %session_id,
                    duration_ms = duration.as_millis(),
                    "Recording stopped"
                );
            }
        }

        Ok(())
    }
}
