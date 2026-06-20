pub enum WorkerEvent {
    Ready,
    Transcript(String),
    Error(String),
}
