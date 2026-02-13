use crate::{AudioError, CoreResult};

use std::panic::Location;

use audioadapter_buffers::direct::InterleavedSlice;
use error_location::ErrorLocation;
use rubato::{Fft, FixedSync, Resampler as RubatoResampler};
use tracing::{debug, instrument};

pub struct Resampler {
    resampler: Fft<f32>,
    input_rate: u32,
    output_rate: u32,
    chunk_size: usize,
}

impl Resampler {
    #[track_caller]
    #[instrument]
    pub fn new(input_rate: u32, output_rate: u32) -> CoreResult<Self> {
        let chunk_size = 1024;
        let sub_chunks = 2; // Sub-chunks for processing

        let resampler = Fft::<f32>::new(
            input_rate as usize,  // sample_rate_input
            output_rate as usize, // sample_rate_output
            chunk_size,           // chunk_size
            sub_chunks,           // sub_chunks
            1,                    // nbr_channels (mono)
            FixedSync::Input,     // fixed
        )
        .map_err(|e| AudioError::ResamplingError {
            reason: format!("Failed to create resampler: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })?;

        debug!(
            input_rate = input_rate,
            output_rate = output_rate,
            chunk_size = chunk_size,
            "Resampler initialized"
        );

        Ok(Self {
            resampler,
            input_rate,
            output_rate,
            chunk_size,
        })
    }

    #[track_caller]
    #[instrument(skip(self, samples))]
    pub fn resample(&mut self, samples: &[f32]) -> CoreResult<Vec<f32>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let estimated_len =
            (samples.len() as f64 * self.output_rate as f64 / self.input_rate as f64) as usize;
        let mut output = Vec::with_capacity(estimated_len);

        for chunk in samples.chunks(self.chunk_size) {
            let input_chunk = if chunk.len() < self.chunk_size {
                let mut padded = chunk.to_vec();
                padded.resize(self.chunk_size, 0.0);
                padded
            } else {
                chunk.to_vec()
            };

            // Create adapter for input (frames, channels)
            let input_adapter =
                InterleavedSlice::new(&input_chunk, 1, self.chunk_size).map_err(|e| {
                    AudioError::ResamplingError {
                        reason: format!("Failed to create input adapter: {}", e),
                        location: ErrorLocation::from(Location::caller()),
                    }
                })?;

            // Calculate output size for this chunk
            let output_frames = self.resampler.output_frames_max();
            let mut output_chunk = vec![0.0f32; output_frames];

            let mut output_adapter = InterleavedSlice::new_mut(&mut output_chunk, 1, output_frames)
                .map_err(|e| AudioError::ResamplingError {
                    reason: format!("Failed to create output adapter: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            // Process chunk
            let (_input_frames, output_frames_written) = self
                .resampler
                .process_into_buffer(&input_adapter, &mut output_adapter, None)
                .map_err(|e| AudioError::ResamplingError {
                    reason: format!("Resampling failed: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            // Append the resampled data
            output.extend_from_slice(&output_chunk[..output_frames_written]);
        }

        output.truncate(estimated_len);

        debug!(
            input_len = samples.len(),
            output_len = output.len(),
            input_rate = self.input_rate,
            output_rate = self.output_rate,
            "Resampled audio"
        );

        Ok(output)
    }
}
