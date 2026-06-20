#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
pub(crate) struct XrandrSize {
    pub(crate) width: f32,
    pub(crate) height: f32,
}
