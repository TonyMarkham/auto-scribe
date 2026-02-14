//! Clipboard integration and auto-paste functionality.
//!
//! Handles copying transcribed text to the clipboard and optionally simulating
//! Ctrl+V to paste it into the active window.

use crate::{AppError, AppResult, CtrlKeyGuard};

use std::panic::Location;
use std::time::Duration;

use arboard::Clipboard;
use error_location::ErrorLocation;
use tracing::{debug, info, instrument, warn};

/// Delay between clipboard write and paste simulation.
///
/// This gives the OS clipboard manager time to process the write before
/// we simulate Ctrl+V. Too short and the paste may get stale content;
/// too long and the user perceives lag. 50ms is empirically reliable
/// across Windows, macOS, and Linux desktop environments.
const CLIPBOARD_SETTLE_DELAY: Duration = Duration::from_millis(50);

/// Delay between key events in the paste simulation.
///
/// Keyboard event timing: some applications and input method editors
/// need a small gap between key_down, key_click, and key_up to register
/// events correctly. 10ms is the minimum reliable interval.
const KEY_EVENT_DELAY: Duration = Duration::from_millis(10);

/// Output handler for clipboard and auto-paste operations.
pub struct OutputHandler {
    pub(crate) clipboard: Clipboard,
}

impl OutputHandler {
    /// Create a new output handler.
    #[track_caller]
    #[instrument]
    pub fn new() -> AppResult<Self> {
        let clipboard = Clipboard::new().map_err(|e| AppError::ClipboardError {
            reason: format!("Failed to initialize clipboard: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        info!("OutputHandler initialized");

        Ok(Self { clipboard })
    }

    /// Output text to clipboard and optionally auto-paste.
    ///
    /// Always copies to clipboard first. If `auto_paste` is true,
    /// simulates Ctrl+V after a short delay.
    #[instrument(skip(self, text))]
    pub async fn output_text(&mut self, text: &str, auto_paste: bool) -> AppResult<()> {
        // Step 1: Always copy to clipboard first
        self.clipboard
            .set_text(text)
            .map_err(|e| AppError::ClipboardError {
                reason: format!("Failed to set clipboard: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        debug!(text_len = text.len(), "Text copied to clipboard");

        // Step 2: Auto-paste if enabled
        if auto_paste {
            // Allow clipboard manager to process the write before pasting.
            // See CLIPBOARD_SETTLE_DELAY documentation for rationale.
            tokio::time::sleep(CLIPBOARD_SETTLE_DELAY).await;

            if let Err(e) = self.paste().await {
                // Log paste failure but text is already in clipboard
                warn!(
                    error = ?e,
                    "Auto-paste failed, but text is in clipboard"
                );
                return Err(e);
            }
        }

        info!(
            text_len = text.len(),
            auto_pasted = auto_paste,
            "Text output complete"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    async fn paste(&mut self) -> AppResult<()> {
        use enigo::{Direction, Key, Keyboard};

        // Simulate Ctrl+V using spawn_blocking since enigo operations are
        // synchronous and involve small sleeps for key event timing.
        //
        // NOTE: A new Enigo instance is created inside spawn_blocking because:
        // 1. Enigo is not Send, so it cannot be moved across thread boundaries
        // 2. spawn_blocking requires 'static + Send closure
        // 3. Enigo::new() is cheap (no heavy platform initialization)
        // This is intentional, not a bug.
        //
        // RAII SAFETY: CtrlKeyGuard ensures Ctrl is released on drop, even if
        // key operations fail or panic. Without this, a failure after pressing Ctrl
        // would leave Ctrl stuck, making the keyboard unusable.
        let paste_result = tokio::task::spawn_blocking(|| {
            let mut guard = CtrlKeyGuard::new()?;

            std::thread::sleep(KEY_EVENT_DELAY);

            guard
                .enigo_mut()
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::AutoPasteFailed {
                    reason: format!("Failed to press V: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            std::thread::sleep(KEY_EVENT_DELAY);

            // Guard drops here, calling key(Control, Release) automatically.
            // No explicit release needed -- RAII guarantees cleanup.
            Ok::<(), AppError>(())
        })
        .await
        .map_err(|e| AppError::AutoPasteFailed {
            reason: format!("Paste task panicked: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        paste_result?;

        debug!("Auto-paste simulated");

        Ok(())
    }
}
