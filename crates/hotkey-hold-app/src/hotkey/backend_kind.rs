#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackendKind {
    GlobalHotkey,
    WaylandPortal,
}

impl BackendKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::GlobalHotkey => "global-hotkey",
            Self::WaylandPortal => "XDG portal global shortcuts",
        }
    }
}
