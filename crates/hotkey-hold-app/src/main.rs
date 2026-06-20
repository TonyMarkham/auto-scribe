mod error;
mod hotkey;
mod icon;
mod stt;
mod windows;

// ---------------------------------------------------------------------------------------------- //

use crate::{
    error::{AppError, AppResult},
    hotkey::{
        Controller, new_event_channel, select_backend_kind, start_event_task, start_runtime,
        start_stt_event_task,
    },
    stt::Session,
    windows::open_main_window,
};

use gpui::{App, AppContext, Pixels, WindowBounds, px, size};
use gpui_component_assets::Assets as GpuiComponentAssets;

const WINDOW_TITLE: &str = "Hotkey Hold App";
const RESIZE_EDGE_SIZE: Pixels = px(6.0);
const RESIZE_CORNER_SIZE: Pixels = px(14.0);
const DEFAULT_START_WIDTH: f32 = 520.0;
const DEFAULT_START_HEIGHT: f32 = 850.0;
const MINIMUM_WIDTH: f32 = 360.0;
const MINIMUM_HEIGHT: f32 = 220.0;

fn main() {
    prefer_x11_windowing_for_gnome_wayland();

    gpui_platform::application()
        .with_assets(GpuiComponentAssets)
        .run(run_app);
}

#[cfg(target_os = "linux")]
fn prefer_x11_windowing_for_gnome_wayland() {
    let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
    let has_x11 = std::env::var_os("DISPLAY").is_some_and(|value| !value.is_empty());
    let is_gnome = std::env::var_os("XDG_CURRENT_DESKTOP")
        .and_then(|value| value.into_string().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("gnome"));

    if has_wayland && has_x11 && is_gnome {
        // GPUI's Wayland always-on-top path requires wlr-layer-shell, which GNOME does not expose.
        // This runs before GPUI starts or any app threads are spawned, so changing process env is safe.
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn prefer_x11_windowing_for_gnome_wayland() {}

fn run_app(app: &mut App) {
    gpui_component::init(app);

    if let Err(error) = start(app) {
        report_error(&error);
    }
}

fn start(app: &mut App) -> AppResult<()> {
    install_desktop_metadata()?;

    let backend_kind = select_backend_kind();
    let (sender, receiver) = new_event_channel();
    let (stt, stt_receiver) = Session::new().map_err(AppError::speech_to_text)?;
    let controller = app.new(|cx| Controller::new(backend_kind, stt, cx));
    let runtime = start_runtime(backend_kind, sender).map_err(AppError::hotkey_runtime)?;
    let event_task = start_event_task(controller.clone(), receiver, app);
    let stt_event_task = start_stt_event_task(controller.clone(), stt_receiver, app);
    let window_closed_subscription = app.on_window_closed({
        let controller = controller.clone();
        move |app, _| {
            controller.update(app, |controller, cx| {
                controller.window_closed(cx);
            });
        }
    });

    controller.update(app, |controller, cx| {
        controller.install_runtime(
            runtime,
            event_task,
            stt_event_task,
            window_closed_subscription,
            cx,
        );
    });

    let window_bounds =
        WindowBounds::centered(size(px(DEFAULT_START_WIDTH), px(DEFAULT_START_HEIGHT)), app);

    open_main_window(app, controller, window_bounds).map_err(AppError::main_window)
}

#[cfg(target_os = "linux")]
fn install_desktop_metadata() -> AppResult<()> {
    icon::ensure_desktop_entry()
        .map(|_| ())
        .map_err(AppError::desktop_metadata)
}

#[cfg(not(target_os = "linux"))]
fn install_desktop_metadata() -> AppResult<()> {
    Ok(())
}

fn report_error(error: &AppError) {
    eprintln!("{} {}", error.message(), error.location());
}
