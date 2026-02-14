# Auto-Scribe

A cross-platform desktop speech-to-text application using local, offline Whisper transcription. Press a global hotkey to record audio, and the transcribed text is automatically copied to your clipboard and optionally pasted into the active window.

## Features

- **Fully Offline** - Local Whisper inference via whisper-rs, no cloud APIs
- **Global Hotkey** - CTRL+SHIFT+Space to start/stop recording from anywhere
- **Cross-Platform** - macOS, Windows, and Linux
- **System Tray** - Tray icon showing state: Idle, Recording, Processing
- **Auto-Paste** - Transcribed text is pasted directly into the active window (Cmd+V on macOS, Ctrl+V on Windows/Linux)
- **Configurable** - TOML configuration for model path, audio device, auto-paste, and server port

## Requirements

- Rust (Edition 2024)
- A Whisper GGML model file (e.g., `ggml-base.en.bin`) - download from [whisper.cpp models](https://huggingface.co/ggerganov/whisper.cpp)
- **macOS only**: Accessibility permissions for auto-paste (System Settings > Privacy & Security > Accessibility)

## Installation

```bash
git clone https://github.com/TonyMarkham/auto-scribe.git
cd auto-scribe
cargo build --release
```

### Model Download

Download a Whisper GGML model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp/tree/main) and place it in the models directory. English-only models are recommended for best speed and accuracy:

| Model | File | Size | Speed | Accuracy |
|-------|------|------|-------|----------|
| Base | `ggml-base.en.bin` | 148 MB | Fastest | Good |
| Small | `ggml-small.en.bin` | 488 MB | Fast | Better |
| Medium | `ggml-medium.en.bin` | 1.53 GB | Moderate | Best |

**macOS:**
```bash
mkdir -p ~/Library/Application\ Support/com.auto-scribe.Auto-Scribe/models
curl -L -o ~/Library/Application\ Support/com.auto-scribe.Auto-Scribe/models/ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

**Linux:**
```bash
mkdir -p ~/.local/share/auto-scribe/models
curl -L -o ~/.local/share/auto-scribe/models/ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

**Windows (PowerShell):**
```powershell
New-Item -ItemType Directory -Force -Path "$env:APPDATA\auto-scribe\Auto-Scribe\data\models"
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" `
  -OutFile "$env:APPDATA\auto-scribe\Auto-Scribe\data\models\ggml-base.en.bin"
```

Replace `ggml-base.en.bin` with `ggml-small.en.bin` or `ggml-medium.en.bin` for higher accuracy. The model path is configured automatically on first run, or set manually in `config.toml`.

### Default Paths

The `directories` crate determines platform-specific paths using the qualifier `com.auto-scribe.Auto-Scribe`:

| Platform | Config | Data (models) |
|----------|--------|---------------|
| macOS | `~/Library/Application Support/com.auto-scribe.Auto-Scribe/config.toml` | `~/Library/Application Support/com.auto-scribe.Auto-Scribe/models/` |
| Linux | `~/.config/auto-scribe/config.toml` | `~/.local/share/auto-scribe/models/` |
| Windows | `%APPDATA%\auto-scribe\Auto-Scribe\config\config.toml` | `%APPDATA%\auto-scribe\Auto-Scribe\data\models\` |

## Usage

1. Run the application:
   ```bash
   cargo run -p auto-scribe --release
   ```
2. Press **CTRL+SHIFT+Space** to start recording
3. Speak
4. Press **CTRL+SHIFT+Space** again to stop and transcribe
5. Text is copied to clipboard and auto-pasted if enabled

### Tray Menu

- **Settings** - Opens configuration UI in browser (localhost)
- **Exit** - Shuts down the application

## Configuration

Configuration is stored as TOML and created automatically on first run:

```toml
[whisper]
model_path = "/path/to/models/ggml-base.en.bin"

[audio]
# selected_device = "Device Name"  # Optional, defaults to system default

[behavior]
auto_paste = true

[server]
port = 7878
```

## Architecture

Rust workspace with two crates:

### `auto-scribe-core` - Speech-to-Text Library

Audio capture (CPAL), resampling (Rubato, 48kHz to 16kHz), and transcription (whisper-rs). Provides `AudioManager` with a two-step API for async-friendly lock management.

### `auto-scribe` - Desktop Application

System tray app using tao for the event loop, tray-icon for the tray, global-hotkey for hotkey detection, arboard for clipboard, and enigo for keyboard simulation.

The tao event loop runs on the main thread (required because `TrayIcon` is `!Send`). A tokio async runtime runs on a separate thread for hotkey handling, recording, and transcription. The two communicate via `std::sync::mpsc` channels.

## Development

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo test                   # Unit tests
cargo test -p auto-scribe-core --features integration-tests  # Integration tests (needs mic + model)
cargo clippy --workspace --all-targets  # Lint
cargo fmt -- --check         # Check formatting
```

## License

MIT
