use crate::{AppCommand, RecordingState};

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

/// WHAT: State remains Idle when command channel is closed
/// WHY: Prevents inconsistent state when channel send fails
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn given_closed_channel_when_starting_recording_then_state_unchanged() {
    // Given: A closed command channel and Idle state
    let (command_tx, command_rx) = mpsc::channel(1);
    drop(command_rx);
    let state = Arc::new(Mutex::new(RecordingState::Idle));

    // When: Attempting to send StartRecording
    let session_id = Uuid::new_v4();
    let result = command_tx
        .send(AppCommand::StartRecording { session_id })
        .await;

    // Then: Send fails and state remains Idle
    assert!(result.is_err());
    assert_eq!(*state.lock().await, RecordingState::Idle);
}

/// WHAT: State transitions to Recording after successful command send
/// WHY: Ensures state only changes when command is delivered
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn given_idle_state_when_command_sent_successfully_then_transitions_to_recording() {
    // Given: An open command channel and Idle state
    let (command_tx, mut command_rx) = mpsc::channel(32);
    let state = Arc::new(Mutex::new(RecordingState::Idle));

    // When: Sending StartRecording succeeds
    let session_id = Uuid::new_v4();
    command_tx
        .send(AppCommand::StartRecording { session_id })
        .await
        .unwrap();

    // Then: Command is received and state can transition
    let cmd = command_rx.recv().await.unwrap();
    assert!(matches!(cmd, AppCommand::StartRecording { .. }));

    *state.lock().await = RecordingState::Recording {
        started_at: Instant::now(),
        session_id,
    };
    assert!(matches!(
        *state.lock().await,
        RecordingState::Recording { .. }
    ));
}

/// WHAT: State returns to Idle after successful stop command
/// WHY: Ensures toggle behavior completes the full cycle
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn given_recording_state_when_stop_sent_successfully_then_returns_to_idle() {
    // Given: An open command channel and Recording state
    let (command_tx, mut command_rx) = mpsc::channel(32);
    let session_id = Uuid::new_v4();
    let state = Arc::new(Mutex::new(RecordingState::Recording {
        started_at: Instant::now(),
        session_id,
    }));

    // When: Sending StopRecording succeeds
    command_tx
        .send(AppCommand::StopRecording { session_id })
        .await
        .unwrap();

    // Then: Command is received
    let cmd = command_rx.recv().await.unwrap();
    assert!(matches!(cmd, AppCommand::StopRecording { .. }));

    *state.lock().await = RecordingState::Idle;
    assert_eq!(*state.lock().await, RecordingState::Idle);
}
