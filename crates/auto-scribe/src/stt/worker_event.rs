#[derive(Clone, Debug)]
pub(crate) enum WorkerEvent {
    Ready,
    Transcript(String),
    Error(String),
    ModelDownloadProgress {
        file_name: String,
        completed_files: usize,
        total_files: usize,
        file_downloaded_bytes: u64,
        file_total_bytes: Option<u64>,
    },
    ModelDownloadFinished,
    ModelDownloadError(String),
}
