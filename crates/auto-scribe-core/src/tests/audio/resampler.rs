use crate::audio::Resampler;

// Test constants
const INPUT_SAMPLE_RATE: u32 = 48000;
const OUTPUT_SAMPLE_RATE: u32 = 16000;
const ONE_SECOND_INPUT_SAMPLES: usize = INPUT_SAMPLE_RATE as usize;
const ONE_SECOND_OUTPUT_SAMPLES: usize = OUTPUT_SAMPLE_RATE as usize;
const LENGTH_TOLERANCE: u64 = 100;
const TEST_SIGNAL_AMPLITUDE: f32 = 0.5;
const TONE_INPUT_SAMPLES: usize = 4800;
const TONE_FREQUENCY_FACTOR: f32 = 0.1;
const TONE_OUTPUT_SAMPLES: usize = 1539;
const TONE_LENGTH_TOLERANCE: u64 = 100;
const MAX_AMPLITUDE: f32 = 1.5;

/// WHAT: Resampler converts 48kHz to 16kHz correctly
/// WHY: Ensures audio is properly downsampled for Whisper (requires 16kHz)
#[test]
fn given_48khz_audio_when_resampling_to_16khz_then_output_length_approximately_correct() {
    // Given: Resampler configured for 48kHz -> 16kHz
    let mut resampler = Resampler::new(INPUT_SAMPLE_RATE, OUTPUT_SAMPLE_RATE).unwrap();
    let input = vec![TEST_SIGNAL_AMPLITUDE; ONE_SECOND_INPUT_SAMPLES];

    // When: Resampling audio data
    let output = resampler.resample(&input).unwrap();

    // Then: Output is approximately 1 second at 16kHz
    assert!(
        (output.len() as i64 - ONE_SECOND_OUTPUT_SAMPLES as i64).unsigned_abs() < LENGTH_TOLERANCE,
        "Expected ~{} samples, got {}",
        ONE_SECOND_OUTPUT_SAMPLES,
        output.len()
    );
    assert!(output.iter().all(|&s| s.is_finite())); // No NaN/Inf
}

/// WHAT: Empty samples return empty output
/// WHY: Edge case handling for zero-length input
#[test]
fn given_empty_samples_when_resampling_then_empty_output() {
    // Given: Resampler and empty input
    let mut resampler = Resampler::new(INPUT_SAMPLE_RATE, OUTPUT_SAMPLE_RATE).unwrap();
    let empty: Vec<f32> = vec![];

    // When: Resampling empty data
    let output = resampler.resample(&empty).unwrap();

    // Then: Output is also empty
    assert!(output.is_empty());
}

/// WHAT: Resampling preserves signal characteristics
/// WHY: Validates that audio quality is maintained through resampling
#[test]
fn given_tone_signal_when_resampling_then_output_preserves_characteristics() {
    // Given: Resampler and a simple tone signal
    let mut resampler = Resampler::new(INPUT_SAMPLE_RATE, OUTPUT_SAMPLE_RATE).unwrap();
    let input: Vec<f32> = (0..TONE_INPUT_SAMPLES)
        .map(|i| (i as f32 * TONE_FREQUENCY_FACTOR).sin())
        .collect();

    // When: Resampling the signal
    let output = resampler.resample(&input).unwrap();

    // Then: Output has expected length and all samples are finite
    assert!(
        (output.len() as i64 - TONE_OUTPUT_SAMPLES as i64).unsigned_abs() < TONE_LENGTH_TOLERANCE,
        "Expected ~{} samples, got {}",
        TONE_OUTPUT_SAMPLES,
        output.len()
    );
    assert!(
        output
            .iter()
            .all(|&s| s.is_finite() && s.abs() <= MAX_AMPLITUDE)
    );
}
