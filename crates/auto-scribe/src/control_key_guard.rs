use crate::{AppError, AppResult};

use std::panic::Location;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use error_location::ErrorLocation;

/// Returns the platform-specific paste modifier key.
///
/// macOS uses Cmd (Meta), Windows and Linux use Ctrl.
fn paste_modifier() -> Key {
    #[cfg(target_os = "macos")]
    {
        Key::Meta
    }
    #[cfg(not(target_os = "macos"))]
    {
        Key::Control
    }
}

/// RAII guard that guarantees the paste modifier key is released when dropped.
///
/// Prevents stuck keyboard if operations between key press and release fail or panic.
///
/// Owns the `Enigo` instance so all keyboard operations go through it.
/// On drop, releases the modifier with best-effort semantics --
/// if the release fails, the OS will reset modifier state on the next
/// physical key press/release by the user.
pub struct CtrlKeyGuard {
    enigo: Enigo,
    modifier: Key,
}

impl CtrlKeyGuard {
    /// Press the paste modifier and return a guard that will release it on drop.
    #[track_caller]
    pub(crate) fn new() -> AppResult<Self> {
        let modifier = paste_modifier();

        let mut enigo =
            Enigo::new(&Settings::default()).map_err(|e| AppError::AutoPasteFailed {
                reason: format!("Failed to create Enigo: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        enigo
            .key(modifier, Direction::Press)
            .map_err(|e| AppError::AutoPasteFailed {
                reason: format!("Failed to press paste modifier: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        Ok(Self { enigo, modifier })
    }

    /// Access the underlying Enigo for additional key operations while modifier is held.
    pub(crate) fn enigo_mut(&mut self) -> &mut Enigo {
        &mut self.enigo
    }
}

impl Drop for CtrlKeyGuard {
    fn drop(&mut self) {
        let _ = self.enigo.key(self.modifier, Direction::Release);
    }
}
