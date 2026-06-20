# Auto Scribe

A small GPUI app that shows an overlay while `Ctrl+Alt+Space` is held.

The main window shows the active backend, current hotkey state, overlay state, and the latest registration or error message.

## Run

From the workspace root:

```bash
cargo run -p auto-scribe
```

On first launch under Wayland, the desktop portal may show a dialog that looks like it is asking to add a new shortcut. Approve that dialog. It is authorizing this app to use `Ctrl+Alt+Space`; it is not asking you to choose a different shortcut.

On first launch without local model files, the main window shows a download button. The button downloads the required Nemotron ONNX files into the Auto Scribe app data directory and is replaced by file-count and current-file progress bars while the download is running.

## Behavior

- Hold `Ctrl+Alt+Space` to show the overlay.
- Release `Ctrl+Alt+Space` to hide the overlay.
- The overlay is still shown if the main window is minimized.
- Closing the main window exits the app.

The overlay is hidden on release instead of being destroyed. This avoids Wayland compositor behavior where closing a transient overlay can also close the main window.

## Model Files

Auto Scribe uses the `parakeet-rs` crate from crates.io, but the ONNX model files are downloaded separately.

The default model directory is:

```text
$XDG_DATA_HOME/auto-scribe/models/nemotron-speech-streaming-en-0.6b
```

If `XDG_DATA_HOME` is not set, it uses:

```text
~/.local/share/auto-scribe/models/nemotron-speech-streaming-en-0.6b
```

The local config file is:

```text
$XDG_DATA_HOME/auto-scribe/config.toml
```

or:

```text
~/.local/share/auto-scribe/config.toml
```

The config file is created automatically and includes the model directory and base download URL. `NEMOTRON_MODEL_DIR` overrides the configured model directory for local development.

## Linux And Wayland

Wayland global shortcuts require the XDG desktop portal GlobalShortcuts interface. When `WAYLAND_DISPLAY` is set, the app uses the portal backend automatically.

On Wayland, the app automatically installs or updates the required per-user desktop entry at startup. The user does not have to create this file manually.

The generated runtime file is written to:

```text
$XDG_DATA_HOME/applications/dev.gpui.AutoScribe.desktop
```

If `XDG_DATA_HOME` is not set, it uses:

```text
~/.local/share/applications/dev.gpui.AutoScribe.desktop
```

The desktop entry is required because the portal registers host apps by desktop app ID. The app ID is `dev.gpui.AutoScribe`, matching `dev.gpui.AutoScribe.desktop`.

## Troubleshooting

If the portal dialog is cancelled, restart the app and approve the shortcut prompt.

If the status shows a Wayland portal response code 2 error, the desktop portal backend likely failed during shortcut binding. Updating `xdg-desktop-portal-gnome` and `gnome-control-center`, or using a desktop portal with working GlobalShortcuts support, is the expected path forward.

If the shortcut does nothing after a rebuild, restart the app so the generated desktop entry points at the current executable.

## Packaging

The checked-in desktop template is:

```text
crates/auto-scribe/data/dev.gpui.AutoScribe.desktop
```

Runtime development builds use this template and replace `Exec=auto-scribe` with the current executable path. A packaged install should install the same desktop file under the system or user applications directory with the packaged executable path.
