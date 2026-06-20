mod hotkey_window;
mod hotkey_window_placement;
mod main_window;
mod resize;
#[cfg(target_os = "linux")]
mod xrandr_geometry;
#[cfg(target_os = "linux")]
mod xrandr_size;

// ---------------------------------------------------------------------------------------------- //

pub(crate) use hotkey_window::{HotkeyWindow, open_hotkey_window};
pub(crate) use hotkey_window_placement::{HotkeyWindowPlacement, hotkey_window_placement};
pub(crate) use main_window::open_main_window;
pub(crate) use resize::window_resize_handles;
#[cfg(target_os = "linux")]
pub(crate) use xrandr_geometry::XrandrGeometry;
#[cfg(target_os = "linux")]
pub(crate) use xrandr_size::XrandrSize;

pub(crate) const HOTKEY_WINDOW_WIDTH: f32 = 360.0;
pub(crate) const HOTKEY_WINDOW_HEIGHT: f32 = 280.0;
pub(crate) const HOTKEY_WINDOW_BOTTOM_OFFSET: f32 = 120.0;
