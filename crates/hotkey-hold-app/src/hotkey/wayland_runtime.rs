pub(crate) struct WaylandRuntime {
    #[cfg(target_os = "linux")]
    _thread: std::thread::JoinHandle<()>,
}

#[cfg(target_os = "linux")]
impl WaylandRuntime {
    pub(crate) fn new(thread: std::thread::JoinHandle<()>) -> Self {
        Self { _thread: thread }
    }
}
