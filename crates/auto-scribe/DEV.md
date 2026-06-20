# Auto Scribe Dev Notes

## Commands

From the workspace root:

```bash
cargo run -p auto-scribe
cargo fmt --all -- --check
cargo check --workspace --all-targets --offline
cargo clippy --workspace --all-targets --offline
```

## Crate Layout

- `src/main.rs` starts GPUI, creates the controller, starts the hotkey runtime, and opens the main window.
- `src/hotkey.rs` owns backend selection, hotkey runtime setup, runtime event dispatch, and overlay lifecycle state.
- `src/windows/main_window.rs` renders the status window.
- `src/windows/hotkey_window.rs` renders the hold overlay.
- `data/dev.gpui.AutoScribe.desktop` is the desktop entry template used by the Wayland portal path and by packaging.

## Backend Selection

`select_backend_kind` uses the Wayland portal backend when `WAYLAND_DISPLAY` is present. Otherwise it uses the `global-hotkey` backend.

The user-visible shortcut is `Ctrl+Alt+Space`.

The Wayland portal preferred trigger is:

```text
CTRL+ALT+space
```

The portal shortcut ID is:

```text
hold-overlay
```

## Native Backend

The native backend uses `global-hotkey` with:

- modifiers: `CONTROL | ALT`
- key: `Space`

It maps `HotKeyState::Pressed` to `HotkeyEvent::Pressed` and `HotKeyState::Released` to `HotkeyEvent::Released`.

## Wayland Backend

The Wayland backend runs on a dedicated thread and uses `pollster` to run the async portal flow.

Startup sequence:

1. Install or update the per-user desktop entry.
2. Open a dedicated session bus connection.
3. Parse and register app ID `dev.gpui.AutoScribe`.
4. Create a GlobalShortcuts session.
5. Bind `hold-overlay` with preferred trigger `CTRL+ALT+space`.
6. Subscribe to `Activated` and `Deactivated` portal signals.
7. Map those signals to `HotkeyEvent::Pressed` and `HotkeyEvent::Released`.

The desktop entry is written to `$XDG_DATA_HOME/applications/dev.gpui.AutoScribe.desktop`, or to `~/.local/share/applications/dev.gpui.AutoScribe.desktop` when `XDG_DATA_HOME` is unset.

Host app registration is bounded by `HOST_APP_REGISTRATION_TIMEOUT` so an unresponsive portal does not block startup forever.

## Portal Dialog

The portal dialog can look like it is asking the user to add a new shortcut. For this app, approving that dialog authorizes the requested `Ctrl+Alt+Space` binding.

The app asks the portal to bind once per session. If the user cancels, restart the app to retry.

## Overlay Lifecycle

The overlay is intentionally not removed on hotkey release.

Earlier behavior called `window.remove_window()` on release. Under Wayland this could close both the overlay and main window. The current behavior keeps a cached overlay window after first activation:

- press: create the overlay if missing, otherwise call `HotkeyWindow::show`
- release: call `HotkeyWindow::hide`
- hide: render a hidden root element and resize the overlay to `1x1`
- show: resize back to `360x136` and render the overlay content

On Wayland, the overlay uses a `WindowKind::LayerShell` surface on the overlay layer with
keyboard interactivity disabled. That keeps it above normal windows without accepting keyboard
focus.

The workspace patches `gpui` to the vendored Zed GPUI source because the published `gpui 0.2.2`
crate does not expose layer-shell APIs.

## Main Window Close

Keeping the overlay alive creates one shutdown edge case: after the overlay has been used, closing the main window could otherwise leave only the hidden overlay window alive.

`HotkeyController::window_closed` checks whether the only remaining window is the cached overlay. If so, it calls `cx.quit()`.

## Desktop File Handling

There are two desktop file locations:

- `data/dev.gpui.AutoScribe.desktop` is the checked-in template.
- `$XDG_DATA_HOME/applications/dev.gpui.AutoScribe.desktop`, or `~/.local/share/applications/dev.gpui.AutoScribe.desktop`, is the generated runtime copy used by the portal.

The app writes the runtime copy automatically on Wayland startup. Users should not have to create it manually.

The template is included with `include_str!`.

For development runs, `ensure_desktop_entry` creates the user applications directory if needed, then replaces:

```text
Exec=auto-scribe
```

with the current executable path from `env::current_exe()`.

The desktop filename and app ID must stay aligned:

```text
dev.gpui.AutoScribe.desktop
dev.gpui.AutoScribe
```

Changing either value requires updating the other and retesting portal registration.

## Known Portal Failure

If `BindShortcuts` returns portal response code 2 (`Other`), the app reports a specific Wayland portal binding error. This has matched GNOME GlobalShortcuts backend failures during testing. Updating `xdg-desktop-portal-gnome` and `gnome-control-center`, or using a desktop portal with working `BindShortcuts` support, is the expected fix.
