#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) state_label: &'static str,
    pub(crate) worker_ready: bool,
    pub(crate) recorder_available: bool,
    pub(crate) transcript: String,
    pub(crate) status: String,
    pub(crate) model_can_download: bool,
    pub(crate) model_downloading: bool,
    pub(crate) model_download_files_percent: f32,
    pub(crate) model_download_files_label: String,
    pub(crate) model_download_file_percent: f32,
    pub(crate) model_download_file_known: bool,
    pub(crate) model_download_file_label: String,
    pub(crate) model_dir: String,
    pub(crate) config_path: String,
    pub(crate) use_gpu: bool,
    pub(crate) auto_mute_speakers: bool,
}
