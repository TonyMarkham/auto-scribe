#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) hotkey_label: &'static str,
    pub(crate) backend_label: &'static str,
    pub(crate) is_hotkey_down: bool,
    pub(crate) popup_open: bool,
    pub(crate) status: String,
    pub(crate) stt_state_label: &'static str,
    pub(crate) stt_worker_ready: bool,
    pub(crate) stt_recorder_available: bool,
    pub(crate) stt_transcript: String,
    pub(crate) stt_status: String,
}
