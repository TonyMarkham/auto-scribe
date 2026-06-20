use crate::hotkey::{GlobalRuntime, WaylandRuntime};

use std::fmt;

pub(crate) enum Runtime {
    Global { _runtime: GlobalRuntime },
    Wayland { _runtime: WaylandRuntime },
}

impl fmt::Debug for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global { .. } => f.write_str("HotkeyRuntime::Global"),
            Self::Wayland { .. } => f.write_str("HotkeyRuntime::Wayland"),
        }
    }
}
