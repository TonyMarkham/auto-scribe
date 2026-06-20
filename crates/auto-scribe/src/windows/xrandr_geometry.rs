#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
pub(crate) struct XrandrGeometry {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}
