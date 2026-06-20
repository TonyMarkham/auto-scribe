use crate::stt::{SttError, SttResult};

use std::process::Command;

#[derive(Debug)]
pub(crate) struct SpeakerMuteGuard {
    backend: SpeakerMuteBackend,
    restore_muted: bool,
    active: bool,
}

#[derive(Clone, Copy, Debug)]
enum SpeakerMuteBackend {
    Wpctl,
    Pactl,
}

impl SpeakerMuteGuard {
    pub(crate) fn mute_default_sink() -> SttResult<Self> {
        let mut errors = Vec::new();

        match Self::mute_with_backend(SpeakerMuteBackend::Wpctl) {
            Ok(guard) => return Ok(guard),
            Err(error) => errors.push(error.to_string()),
        }

        match Self::mute_with_backend(SpeakerMuteBackend::Pactl) {
            Ok(guard) => return Ok(guard),
            Err(error) => errors.push(error.to_string()),
        }

        Err(SttError::speaker_mute(errors.join("; ")))
    }

    pub(crate) fn restore(&mut self) -> SttResult<()> {
        if !self.active {
            return Ok(());
        }

        set_muted(self.backend, self.restore_muted)?;
        self.active = false;
        Ok(())
    }

    fn mute_with_backend(backend: SpeakerMuteBackend) -> SttResult<Self> {
        let restore_muted = is_muted(backend)?;
        set_muted(backend, true)?;

        Ok(Self {
            backend,
            restore_muted,
            active: true,
        })
    }
}

impl Drop for SpeakerMuteGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn is_muted(backend: SpeakerMuteBackend) -> SttResult<bool> {
    match backend {
        SpeakerMuteBackend::Wpctl => {
            let output = command_output("wpctl", ["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
            Ok(output.contains("[MUTED]"))
        }
        SpeakerMuteBackend::Pactl => {
            let output = command_output("pactl", ["get-sink-mute", "@DEFAULT_SINK@"])?;
            let value = output
                .split_once(':')
                .map(|(_, value)| value.trim())
                .unwrap_or(output.trim());
            Ok(value.eq_ignore_ascii_case("yes"))
        }
    }
}

fn set_muted(backend: SpeakerMuteBackend, muted: bool) -> SttResult<()> {
    let muted_arg = if muted { "1" } else { "0" };

    match backend {
        SpeakerMuteBackend::Wpctl => {
            run_command("wpctl", ["set-mute", "@DEFAULT_AUDIO_SINK@", muted_arg])
        }
        SpeakerMuteBackend::Pactl => {
            run_command("pactl", ["set-sink-mute", "@DEFAULT_SINK@", muted_arg])
        }
    }
}

fn command_output<const N: usize>(program: &str, args: [&str; N]) -> SttResult<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| SttError::speaker_mute(format!("{program}: {error}")))?;

    if !output.status.success() {
        return Err(command_error(program, output.status.code(), &output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_command<const N: usize>(program: &str, args: [&str; N]) -> SttResult<()> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| SttError::speaker_mute(format!("{program}: {error}")))?;

    if output.status.success() {
        return Ok(());
    }

    Err(command_error(program, output.status.code(), &output.stderr))
}

fn command_error(program: &str, code: Option<i32>, stderr: &[u8]) -> SttError {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        "no stderr".to_string()
    } else {
        stderr
    };

    SttError::speaker_mute(format!(
        "{program} failed with status {}; {detail}",
        code.map(|code| code.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    ))
}
