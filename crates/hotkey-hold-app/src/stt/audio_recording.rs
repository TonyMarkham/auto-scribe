use std::time::Duration;

pub(crate) struct AudioRecording {
    pub(crate) samples: Vec<f32>,
    pub(crate) sample_rate: u32,
    pub(crate) clipped: bool,
}

impl AudioRecording {
    pub(crate) fn is_shorter_than(&self, duration: Duration) -> bool {
        let minimum_samples = (duration.as_secs_f64() * f64::from(self.sample_rate)).ceil();
        (self.samples.len() as f64) < minimum_samples
    }

    pub(crate) fn clipped(&self) -> bool {
        self.clipped
    }

    pub(crate) fn into_parts(self) -> (Vec<f32>, u32) {
        (self.samples, self.sample_rate)
    }
}
