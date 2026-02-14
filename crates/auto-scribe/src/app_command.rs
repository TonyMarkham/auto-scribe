use uuid::Uuid;

/// Commands sent from hotkey handler to main application.
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Start a new recording session.
    StartRecording {
        /// Unique session ID for this recording.
        session_id: Uuid,
    },
    /// Stop the current recording session.
    StopRecording {
        /// Session ID of the recording to stop.
        session_id: Uuid,
    },
    /// Request application shutdown.
    Shutdown,
}
