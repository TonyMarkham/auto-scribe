use std::time::Instant;

use uuid::Uuid;

/// Recording state for the hotkey handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    /// Not currently recording.
    Idle,
    /// Currently recording audio.
    Recording {
        /// When recording started.
        started_at: Instant,
        /// Unique session ID for log correlation.
        session_id: Uuid,
    },
}
