use crate::TrayIconState;

/// Commands sent from the async runtime to the main UI thread.
///
/// The main thread owns `TrayManager` (because `TrayIcon` is `!Send`),
/// so all tray mutations and process lifecycle events flow through this enum.
#[derive(Debug, Clone, Copy)]
pub enum TrayCommand {
    /// Update the tray icon to a new state.
    SetState(TrayIconState),
    /// Shut down the application. The main thread will exit the event loop.
    Shutdown,
}
