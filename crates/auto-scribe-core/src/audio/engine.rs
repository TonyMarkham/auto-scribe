use crate::{AudioError, CoreResult};

use std::{panic::Location, path::Path};

use error_location::ErrorLocation;
use tracing::{debug, info, instrument};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct SttEngine {
    ctx: WhisperContext,
}

impl SttEngine {
    #[track_caller]
    #[instrument(skip(model_path))]
    pub fn new<P: AsRef<Path>>(model_path: P) -> CoreResult<Self> {
        let path = model_path.as_ref();

        if !path.exists() {
            return Err(AudioError::ModelNotFound {
                path: path.to_path_buf(),
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let ctx = WhisperContext::new_with_params(
            path.to_str().ok_or(AudioError::ModelNotFound {
                path: path.to_path_buf(),
                location: ErrorLocation::from(Location::caller()),
            })?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| AudioError::TranscriptionFailed {
            source: Box::new(e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        info!(model_path = ?path, "Whisper model loaded");

        Ok(Self { ctx })
    }

    #[track_caller]
    #[instrument(skip(self, samples))]
    pub fn transcribe(&mut self, samples: &[f32]) -> CoreResult<String> {
        if samples.is_empty() {
            return Err(AudioError::NoAudioCaptured {
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Configure for English transcription
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| AudioError::TranscriptionFailed {
                source: Box::new(e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        state
            .full(params, samples)
            .map_err(|e| AudioError::TranscriptionFailed {
                source: Box::new(e),
                location: ErrorLocation::from(Location::caller()),
            })?;

        let num_segments = state.full_n_segments();

        // Pre-allocate result string to avoid repeated reallocations.
        // Average English speech is ~150 words/min, ~5 chars/word.
        // Conservative estimate: 256 bytes per segment covers most cases
        // with a single allocation.
        let mut result = String::with_capacity(num_segments as usize * 256);

        for i in 0..num_segments {
            let segment = state
                .get_segment(i)
                .ok_or_else(|| AudioError::TranscriptionFailed {
                    source: format!("Failed to get segment {}", i).into(),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            result.push_str(&segment.to_string());
            result.push(' ');
        }

        let transcription = result.trim().to_string();

        debug!(
            sample_count = samples.len(),
            segment_count = num_segments,
            text_len = transcription.len(),
            "Transcription complete"
        );

        Ok(transcription)
    }
}
