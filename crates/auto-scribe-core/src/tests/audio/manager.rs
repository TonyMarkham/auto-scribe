use crate::{AudioError, audio::AudioManager};

/// WHAT: AudioManager rejects non-existent model path
/// WHY: Early validation prevents runtime failures
#[test]
fn given_invalid_model_path_when_creating_manager_then_model_not_found_error() {
    // Given: Path to non-existent Whisper model
    let invalid_path = std::path::PathBuf::from("/nonexistent/model.bin");

    // When: Attempting to create AudioManager
    let result = AudioManager::new(&invalid_path, false);

    // Then: Returns ModelNotFound error
    assert!(result.is_err());
    assert!(matches!(result, Err(AudioError::ModelNotFound { .. })));
}

/// WHAT: Empty audio samples cause NoAudioCaptured error
/// WHY: Ensures transcription does not run on silence
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn given_empty_samples_when_transcribing_then_no_audio_captured_error() {
    // Given: AudioManager with valid model
    let model_path = std::env::var("TEST_WHISPER_MODEL_PATH")
        .unwrap_or_else(|_| "models/ggml-base.en.bin".to_string());
    let mut manager = AudioManager::new(&model_path, false).unwrap();
    let empty_samples: Vec<f32> = vec![];

    // When: Attempting to transcribe empty samples
    let result = manager.transcribe_samples(&empty_samples);

    // Then: Returns NoAudioCaptured error
    assert!(matches!(result, Err(AudioError::NoAudioCaptured { .. })));
}
