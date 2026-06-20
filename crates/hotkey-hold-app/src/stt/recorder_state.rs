use crate::stt::TARGET_SAMPLE_RATE;

pub(crate) struct RecorderState {
    pub(crate) recording: bool,
    pub(crate) samples: Vec<f32>,
    pub(crate) max_samples: usize,
    pub(crate) clipped: bool,
    pub(crate) last_stream_error: Option<String>,
}

impl RecorderState {
    pub(crate) fn new(max_samples: usize) -> Self {
        Self {
            recording: false,
            samples: Vec::with_capacity(TARGET_SAMPLE_RATE as usize * 4),
            max_samples,
            clipped: false,
            last_stream_error: None,
        }
    }
}
