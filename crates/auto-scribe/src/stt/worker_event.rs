#[derive(Clone, Debug)]
pub(crate) enum WorkerEvent {
    Ready,
    Transcript(String),
    Error(String),
}
