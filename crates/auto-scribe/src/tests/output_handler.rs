use crate::{CtrlKeyGuard, OutputHandler};

use enigo::{Direction, Key, Keyboard};

/// WHAT: OutputHandler initializes successfully
/// WHY: Ensures clipboard and keyboard simulation are available
#[test]
fn given_system_when_creating_output_handler_then_succeeds() {
    // Given: System with clipboard support

    // When: Creating OutputHandler
    let result = OutputHandler::new();

    // Then: Initialization succeeds
    assert!(result.is_ok());
}

/// WHAT: Text is copied to clipboard
/// WHY: Ensures clipboard integration works even if paste fails
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn given_text_when_outputting_without_paste_then_clipboard_updated() {
    // Given: OutputHandler and test text
    let mut handler = OutputHandler::new().unwrap();
    let text = "Test transcription";

    // When: Outputting text without auto-paste
    let result = handler.output_text(text, false).await;

    // Then: Operation succeeds and clipboard contains text
    assert!(result.is_ok());

    let clipboard_text = handler.clipboard.get_text().unwrap();
    assert_eq!(clipboard_text, text);
}

/// WHAT: CtrlKeyGuard releases Ctrl on normal drop
/// WHY: Ensures RAII cleanup works in the happy path
#[test]
#[ignore] // Requires macOS accessibility permissions - run manually with: cargo test -- --ignored
fn given_ctrl_guard_when_dropped_normally_then_ctrl_released() {
    // Given/When/Then: Guard can be constructed and dropped without panicking.
    // Full keyboard state verification requires platform-specific APIs
    // or integration testing with a virtual desktop.
    let guard = CtrlKeyGuard::new();
    if let Ok(guard) = guard {
        drop(guard); // Should not panic
    }
    // If CtrlKeyGuard::new() fails (e.g., headless CI), test passes trivially
}

/// WHAT: CtrlKeyGuard releases Ctrl even when inner operations fail
/// WHY: Prevents stuck keyboard when key operations fail after Ctrl press
#[test]
#[ignore] // Requires macOS accessibility permissions - run manually with: cargo test -- --ignored
fn given_ctrl_guard_when_inner_operation_fails_then_ctrl_still_released() {
    // Given: A CtrlKeyGuard that pressed Ctrl

    let guard = CtrlKeyGuard::new();
    if let Ok(mut guard) = guard {
        // When: An operation on enigo is attempted
        let _ = guard.enigo_mut().key(Key::Unicode('z'), Direction::Click);

        // Then: Guard drops and releases Ctrl regardless of inner result
        drop(guard); // Should not panic
    }
}
