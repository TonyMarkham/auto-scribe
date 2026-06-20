#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) state_label: &'static str,
    pub(crate) worker_ready: bool,
    pub(crate) recorder_available: bool,
    pub(crate) transcript: String,
    pub(crate) status: String,
}
