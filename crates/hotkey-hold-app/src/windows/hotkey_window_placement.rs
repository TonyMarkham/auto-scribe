use crate::windows::{HOTKEY_WINDOW_BOTTOM_OFFSET, HOTKEY_WINDOW_HEIGHT, HOTKEY_WINDOW_WIDTH};

#[cfg(target_os = "linux")]
use crate::windows::{XrandrGeometry, XrandrSize};
#[cfg(target_os = "linux")]
use std::process::Command;

use gpui::{App, Bounds, DisplayId, Pixels, WindowBounds, point, px, size};

#[derive(Clone, Copy)]
pub(crate) struct HotkeyWindowPlacement {
    pub(crate) window_bounds: WindowBounds,
    pub(crate) display_id: Option<DisplayId>,
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    pub(crate) layer_shell_margin: (Pixels, Pixels, Pixels, Pixels),
}

pub(crate) fn hotkey_window_placement(app: &App) -> HotkeyWindowPlacement {
    let window_size = size(px(HOTKEY_WINDOW_WIDTH), px(HOTKEY_WINDOW_HEIGHT));
    let bottom_offset = px(HOTKEY_WINDOW_BOTTOM_OFFSET);
    let Some(display) = app.primary_display() else {
        return HotkeyWindowPlacement {
            window_bounds: WindowBounds::Windowed(Bounds::new(
                point(px(0.0), px(0.0)),
                window_size,
            )),
            display_id: None,
            #[cfg(all(target_os = "linux", feature = "wayland"))]
            layer_shell_margin: (px(0.0), px(0.0), bottom_offset, px(0.0)),
        };
    };

    let display_bounds = placement_display_bounds(app, display.bounds());
    let origin = point(
        display_bounds.origin.x + (display_bounds.size.width - window_size.width) / 2.0,
        display_bounds.origin.y + display_bounds.size.height - window_size.height - bottom_offset,
    );

    HotkeyWindowPlacement {
        window_bounds: WindowBounds::Windowed(Bounds::new(origin, window_size)),
        display_id: Some(display.id()),
        #[cfg(all(target_os = "linux", feature = "wayland"))]
        layer_shell_margin: (
            px(0.0),
            px(0.0),
            bottom_offset,
            origin.x - display_bounds.origin.x,
        ),
    }
}

#[cfg(target_os = "linux")]
fn placement_display_bounds(app: &App, display_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    if app.compositor_name() == "X11"
        && let Some(primary_monitor_bounds) = x11_primary_monitor_bounds(display_bounds)
    {
        return primary_monitor_bounds;
    }

    display_bounds
}

#[cfg(not(target_os = "linux"))]
fn placement_display_bounds(_: &App, display_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    display_bounds
}

#[cfg(target_os = "linux")]
fn x11_primary_monitor_bounds(display_bounds: Bounds<Pixels>) -> Option<Bounds<Pixels>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .arg("--current")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let root_size = parse_xrandr_root_size(&stdout)?;
    let monitor = parse_xrandr_primary_monitor(&stdout)?;

    let display_width = f32::from(display_bounds.size.width);
    let display_height = f32::from(display_bounds.size.height);
    if display_width <= 0.0
        || display_height <= 0.0
        || root_size.width <= 0.0
        || root_size.height <= 0.0
        || monitor.width <= 0.0
        || monitor.height <= 0.0
    {
        return None;
    }

    let scale_x = root_size.width / display_width;
    let scale_y = root_size.height / display_height;
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return None;
    }

    Some(Bounds::new(
        point(
            display_bounds.origin.x + px(monitor.x / scale_x),
            display_bounds.origin.y + px(monitor.y / scale_y),
        ),
        size(px(monitor.width / scale_x), px(monitor.height / scale_y)),
    ))
}

#[cfg(target_os = "linux")]
fn parse_xrandr_root_size(output: &str) -> Option<XrandrSize> {
    for line in output.lines() {
        let Some((_, after_current)) = line.split_once(" current ") else {
            continue;
        };
        let mut parts = after_current.split_whitespace();
        let width = parts.next()?.parse::<f32>().ok()?;
        if parts.next()? != "x" {
            return None;
        }
        let height = parts.next()?.trim_end_matches(',').parse::<f32>().ok()?;

        return Some(XrandrSize { width, height });
    }

    None
}

#[cfg(target_os = "linux")]
fn parse_xrandr_primary_monitor(output: &str) -> Option<XrandrGeometry> {
    let mut first_connected_monitor = None;

    for line in output.lines() {
        let mut words = line.split_whitespace();
        if words.next().is_none() || words.next() != Some("connected") {
            continue;
        }

        let Some(geometry) = line
            .split_whitespace()
            .find_map(parse_xrandr_geometry_token)
        else {
            continue;
        };

        if line.split_whitespace().any(|word| word == "primary") {
            return Some(geometry);
        }

        if first_connected_monitor.is_none() {
            first_connected_monitor = Some(geometry);
        }
    }

    first_connected_monitor
}

#[cfg(target_os = "linux")]
fn parse_xrandr_geometry_token(token: &str) -> Option<XrandrGeometry> {
    let width_end = token.find('x')?;
    let width = token[..width_end].parse::<f32>().ok()?;
    let after_width = &token[width_end + 1..];
    let height_end = after_width.find(['+', '-'])?;
    let height = after_width[..height_end].parse::<f32>().ok()?;
    let coordinates = &after_width[height_end..];
    let y_start = coordinates[1..].find(['+', '-'])? + 1;
    let x = coordinates[..y_start].parse::<f32>().ok()?;
    let y = coordinates[y_start..]
        .trim_end_matches(',')
        .parse::<f32>()
        .ok()?;

    Some(XrandrGeometry {
        x,
        y,
        width,
        height,
    })
}
