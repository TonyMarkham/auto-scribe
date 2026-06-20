mod backend_kind;
mod controller;
mod event;
mod global_runtime;
mod native;
mod runtime;
mod runtime_event;
mod snapshot;
mod wayland;
mod wayland_runtime;

// ---------------------------------------------------------------------------------------------- //

use crate::error::AppResult;
use crate::stt::WorkerEvent;
use async_channel::{Receiver, Sender};
use gpui::{App, Entity, Task, WeakEntity};

pub(crate) use backend_kind::BackendKind;
pub(crate) use controller::Controller;
pub(crate) use event::Event;
pub(crate) use global_runtime::GlobalRuntime;
pub(crate) use runtime::Runtime;
pub(crate) use runtime_event::RuntimeEvent;
pub(crate) use snapshot::Snapshot;
pub(crate) use wayland_runtime::WaylandRuntime;

// ---------------------------------------------------------------------------------------------- //

pub(crate) const HOTKEY_ID: &str = "hold-overlay";
const HOTKEY_LABEL: &str = "Ctrl+Alt+Space";
const WAYLAND_APP_ID: &str = crate::icon::APP_ID;
const WAYLAND_PREFERRED_TRIGGER: &str = "CTRL+ALT+space";

pub(crate) fn new_event_channel() -> (Sender<RuntimeEvent>, Receiver<RuntimeEvent>) {
    async_channel::unbounded()
}

pub(crate) fn select_backend_kind() -> BackendKind {
    #[cfg(target_os = "linux")]
    {
        if is_wayland_session() {
            return BackendKind::WaylandPortal;
        }
    }

    BackendKind::GlobalHotkey
}

#[cfg(target_os = "linux")]
fn is_wayland_session() -> bool {
    let has_wayland_display =
        std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
    let session_type_is_wayland = std::env::var_os("XDG_SESSION_TYPE")
        .and_then(|value| value.into_string().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("wayland"));

    has_wayland_display || session_type_is_wayland
}

pub(crate) fn start_runtime(
    backend_kind: BackendKind,
    sender: Sender<RuntimeEvent>,
) -> AppResult<Runtime> {
    match backend_kind {
        BackendKind::GlobalHotkey => native::start(sender),
        BackendKind::WaylandPortal => wayland::start(sender),
    }
}

pub(crate) fn start_event_task(
    controller: Entity<Controller>,
    receiver: Receiver<RuntimeEvent>,
    app: &mut App,
) -> Task<()> {
    let controller = controller.downgrade();

    app.spawn(move |cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Ok(event) = receiver.recv().await {
                dispatch_runtime_event(&controller, event, &mut cx);
            }
        }
    })
}

pub(crate) fn start_stt_event_task(
    controller: Entity<Controller>,
    receiver: Receiver<WorkerEvent>,
    app: &mut App,
) -> Task<()> {
    let controller = controller.downgrade();

    app.spawn(move |cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Ok(event) = receiver.recv().await {
                dispatch_stt_event(&controller, event, &mut cx);
            }
        }
    })
}

fn dispatch_runtime_event(
    controller: &WeakEntity<Controller>,
    event: RuntimeEvent,
    cx: &mut gpui::AsyncApp,
) {
    let _ = controller.update(cx, |controller, cx| {
        controller.apply_runtime_event(event, cx);
    });
}

fn dispatch_stt_event(
    controller: &WeakEntity<Controller>,
    event: WorkerEvent,
    cx: &mut gpui::AsyncApp,
) {
    let _ = controller.update(cx, |controller, cx| {
        controller.apply_stt_event(event, cx);
    });
}

fn format_error_chain(error: &crate::error::AppError) -> String {
    error.to_string()
}
