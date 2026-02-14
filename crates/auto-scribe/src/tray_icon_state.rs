/// Tray icon states corresponding to application workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconState {
    /// Ready to start recording.
    Idle,
    /// Currently recording audio.
    Recording,
    /// Processing/transcribing audio.
    Processing,
}
