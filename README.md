# Auto Scribe

A small GPUI app that shows an overlay while `Ctrl+Alt+Space` is held.

The main window shows the active backend, current hotkey state, overlay state, and the latest registration or error message.

## Run

From the workspace root:

```bash
cargo run -p auto-scribe
```

### Detached Shell Alias

If the release binary has been published to `~/.local/share/auto-scribe/bin/auto-scribe`, add this block to `~/.bashrc`:

```bash
# >>> Auto Scribe >>>
alias auto-scribe='setsid -f "${HOME}/.local/share/auto-scribe/bin/auto-scribe" >/tmp/auto-scribe.log 2>&1 </dev/null'
# <<< Auto Scribe <<<
```

Reload the shell config:

```bash
source ~/.bashrc
```

Then launch Auto Scribe from any terminal:

```bash
auto-scribe
```

The alias uses `setsid -f` so the app keeps running after the terminal closes. stdout and stderr are written to `/tmp/auto-scribe.log`.

On first launch under Wayland, the desktop portal may show a dialog that looks like it is asking to add a new shortcut. Approve that dialog. It is authorizing this app to use `Ctrl+Alt+Space`; it is not asking you to choose a different shortcut.

On first launch without local model files, the main window shows a download button. The button downloads the required Nemotron ONNX files into the Auto Scribe app data directory and is replaced by file-count and current-file progress bars while the download is running.

## Behavior

- Hold `Ctrl+Alt+Space` to show the overlay.
- Release `Ctrl+Alt+Space` to hide the overlay.
- The overlay is still shown if the main window is minimized.
- Closing the main window exits the app.

The overlay is hidden on release instead of being destroyed. This avoids Wayland compositor behavior where closing a transient overlay can also close the main window.

When Auto-Mute is enabled, Auto Scribe mutes the default speaker sink while recording and restores the prior mute state when recording stops.

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

The config file is created automatically and includes the model directory, base download URL, GPU setting, and audio settings. `NEMOTRON_MODEL_DIR` overrides the configured model directory for local development.

```toml
[model]
directory = "models/nemotron-speech-streaming-en-0.6b"
base_url = "https://huggingface.co/altunenes/parakeet-rs/resolve/main/nemotron-speech-streaming-en-0.6b"
use_gpu = false

[audio]
auto_mute_speakers = false
```

## CUDA GPU Acceleration

The `Use GPU` toggle enables NVIDIA CUDA inference through ONNX Runtime. When disabled, Auto Scribe uses CPU inference.

The setting is persisted in `config.toml`:

```toml
[model]
use_gpu = true
```

The current GPU path is CUDA-only. Apple Silicon, Radeon, and Intel Arc are not enabled by this toggle.

For published release builds, run `just publish` after `cargo build --release`. The publish recipe copies the executable and the ONNX Runtime provider libraries into:

```text
~/.local/share/auto-scribe/bin/
```

If GPU loading fails with `libonnxruntime_providers_shared.so: cannot open shared object file`, republish the app so the ONNX Runtime provider `.so` files sit beside the executable.

If GPU loading fails with a missing CUDA library such as `libcublasLt.so.12`, install the CUDA 12 runtime libraries and cuDNN. On Ubuntu:

```bash
sudo apt-get install -y libcublas12 libcublaslt12 libcudart12 libcufft11 libcurand10 nvidia-cudnn
```

Then verify that the CUDA provider can resolve its dependencies:

```bash
ldd ~/.local/share/auto-scribe/bin/libonnxruntime_providers_cuda.so | grep 'not found'
```

That command should print nothing. `nvidia-smi` should also be able to see the GPU.

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

Speaker auto-mute uses `wpctl` first and falls back to `pactl`. If neither command can control the default sink, dictation still runs and the STT status reports the auto-mute failure.

## Packaging

The checked-in desktop template is:

```text
crates/auto-scribe/data/dev.gpui.AutoScribe.desktop
```

Runtime development builds use this template and replace `Exec=auto-scribe` with the current executable path. A packaged install should install the same desktop file under the system or user applications directory with the packaged executable path.
