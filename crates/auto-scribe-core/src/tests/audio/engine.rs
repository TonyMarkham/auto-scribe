use crate::{AudioError, audio::SttEngine};

use std::path::PathBuf;

/// WHAT: SttEngine rejects non-existent model path
/// WHY: Early validation prevents runtime failures
#[test]
fn given_invalid_model_path_when_creating_engine_then_model_not_found_error() {
    // Given: Path to non-existent Whisper model
    let invalid_path = PathBuf::from("/nonexistent/model.bin");

    // When: Attempting to create SttEngine
    let result = SttEngine::new(&invalid_path, false);

    // Then: Returns ModelNotFound error
    assert!(matches!(result, Err(AudioError::ModelNotFound { .. })));
}

/// WHAT: Empty samples cause NoAudioCaptured error
/// WHY: Transcription should not run on empty audio
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn given_empty_samples_when_transcribing_then_no_audio_captured_error() {
    // Given: SttEngine with valid model
    let model_path = std::env::var("TEST_WHISPER_MODEL_PATH")
        .unwrap_or_else(|_| "models/ggml-base.en.bin".to_string());
    let mut engine = SttEngine::new(&model_path, false).unwrap();
    let empty_samples: Vec<f32> = vec![];

    // When: Attempting to transcribe empty samples
    let result = engine.transcribe(&empty_samples);

    // Then: Returns NoAudioCaptured error
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AudioError::NoAudioCaptured { .. }
    ));
}
