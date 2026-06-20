use crate::hotkey::{Runtime, RuntimeEvent};

use crate::error::{AppError, AppResult, ResultContext};
use async_channel::Sender;

#[cfg(target_os = "linux")]
use crate::hotkey::{
    Event, HOTKEY_ID, HOTKEY_LABEL, WAYLAND_APP_ID, WAYLAND_PREFERRED_TRIGGER, WaylandRuntime,
    format_error_chain,
};
#[cfg(target_os = "linux")]
use crate::icon::ensure_desktop_entry;
#[cfg(target_os = "linux")]
use ashpd::{
    AppID, Error as AshpdError,
    desktop::{
        CreateSessionOptions, ResponseError,
        global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    },
    register_host_app_with_connection, zbus,
};
#[cfg(target_os = "linux")]
use futures_util::StreamExt;
#[cfg(target_os = "linux")]
use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

#[cfg(target_os = "linux")]
const WAYLAND_DESKTOP_FILE_NAME: &str = "dev.gpui.HotkeyHoldApp.desktop";
#[cfg(target_os = "linux")]
const HOST_APP_REGISTRATION_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(target_os = "linux")]
pub(crate) fn start(sender: Sender<RuntimeEvent>) -> AppResult<Runtime> {
    let thread = thread::Builder::new()
        .name("wayland-global-shortcut-portal".to_string())
        .spawn(move || {
            if let Err(error) = run_portal(sender.clone()) {
                let message = format_error_chain(&error);
                let _ = sender.try_send(RuntimeEvent::Error(format!(
                    "Wayland global shortcut portal failed: {message}"
                )));
            }
        })
        .context("spawn Wayland global shortcut portal thread")?;

    Ok(Runtime::Wayland {
        _runtime: WaylandRuntime::new(thread),
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn start(_: Sender<RuntimeEvent>) -> AppResult<Runtime> {
    Err(AppError::operation(
        "Wayland portal hotkeys are only available on Linux",
    ))
}

#[cfg(target_os = "linux")]
fn run_portal(sender: Sender<RuntimeEvent>) -> AppResult<()> {
    pollster::block_on(async move {
        let desktop_entry_path =
            ensure_desktop_entry().context("install XDG desktop file for Wayland portal")?;
        let _ = sender.try_send(RuntimeEvent::Status(format!(
            "Using desktop entry {}",
            desktop_entry_path.display()
        )));

        let connection = zbus::Connection::session()
            .await
            .context("connect to the session bus for Wayland global shortcuts")?;
        let app_id = WAYLAND_APP_ID
            .parse::<AppID>()
            .context("parse Wayland portal app id")?;

        let _ = sender.try_send(RuntimeEvent::Status(format!(
            "Registering {WAYLAND_APP_ID} with the XDG portal"
        )));

        register_host_app_with_bounded_wait(connection.clone(), app_id)?;

        let portal = GlobalShortcuts::with_connection(connection)
            .await
            .context("connect to org.freedesktop.portal.GlobalShortcuts")?;

        let session = portal
            .create_session(CreateSessionOptions::default())
            .await
            .context("create global shortcuts session")?;

        let shortcut = NewShortcut::new(HOTKEY_ID, "Show hold overlay")
            .preferred_trigger(Some(WAYLAND_PREFERRED_TRIGGER));

        let _ = sender.try_send(RuntimeEvent::Status(format!(
            "Requesting {HOTKEY_LABEL}; approve it in the portal dialog"
        )));

        let request = portal
            .bind_shortcuts(&session, &[shortcut], None, BindShortcutsOptions::default())
            .await
            .context("request Wayland global shortcut binding")?;

        let response = request.response().map_err(describe_bind_shortcuts_error)?;

        if !response
            .shortcuts()
            .iter()
            .any(|shortcut| shortcut.id() == HOTKEY_ID)
        {
            return Err(AppError::operation(format!(
                "portal did not bind {HOTKEY_LABEL}; use the desktop shortcut dialog to allow it"
            )));
        }

        let _ = sender.try_send(RuntimeEvent::Status(format!(
            "Registered {HOTKEY_LABEL} through the Wayland portal"
        )));

        let activated = portal
            .receive_activated()
            .await
            .context("subscribe to Wayland shortcut activation")?
            .fuse();
        let deactivated = portal
            .receive_deactivated()
            .await
            .context("subscribe to Wayland shortcut deactivation")?
            .fuse();

        futures_util::pin_mut!(activated);
        futures_util::pin_mut!(deactivated);

        loop {
            futures_util::select! {
                event = activated.next() => {
                    let Some(event) = event else {
                        break;
                    };

                    if event.shortcut_id() == HOTKEY_ID {
                        let _ = sender.try_send(RuntimeEvent::Hotkey(Event::Pressed));
                    }
                }
                event = deactivated.next() => {
                    let Some(event) = event else {
                        break;
                    };

                    if event.shortcut_id() == HOTKEY_ID {
                        let _ = sender.try_send(RuntimeEvent::Hotkey(Event::Released));
                    }
                }
            }
        }

        Ok(())
    })
}

#[cfg(target_os = "linux")]
fn describe_bind_shortcuts_error(error: AshpdError) -> AppError {
    match error {
        AshpdError::Response(ResponseError::Cancelled) => {
            AppError::operation("Wayland global shortcut binding was cancelled")
        }
        AshpdError::Response(ResponseError::Other) => AppError::operation(format!(
            "Wayland portal returned response code 2 (Other) while binding {HOTKEY_LABEL}. This matches known GNOME GlobalShortcuts backend failures; update xdg-desktop-portal-gnome/gnome-control-center or use a desktop portal with working BindShortcuts support"
        )),
        error => AppError::with_context("read Wayland global shortcut binding response", error),
    }
}

#[cfg(target_os = "linux")]
fn register_host_app_with_bounded_wait(
    connection: zbus::Connection,
    app_id: AppID,
) -> AppResult<()> {
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    let _registration_thread = thread::Builder::new()
        .name("wayland-host-app-registration".to_string())
        .spawn(move || {
            let result = pollster::block_on(register_host_app_with_connection(connection, app_id));
            let _ = result_sender.send(result);
        })
        .context("spawn XDG host app registration thread")?;

    match result_receiver.recv_timeout(HOST_APP_REGISTRATION_TIMEOUT) {
        Ok(result) => result.context("register host app with the XDG portal"),
        Err(RecvTimeoutError::Timeout) => Err(AppError::operation(format!(
            "XDG host app registration timed out after {} seconds; ensure {WAYLAND_DESKTOP_FILE_NAME} is installed and the desktop global shortcuts provider is responsive",
            HOST_APP_REGISTRATION_TIMEOUT.as_secs()
        ))),
        Err(RecvTimeoutError::Disconnected) => Err(AppError::operation(
            "XDG host app registration thread stopped before sending a result",
        )),
    }
}
