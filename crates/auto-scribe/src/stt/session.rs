use crate::stt::{
    AudioRecorder, MIN_RECORDING_DURATION, ModelConfig, Snapshot, SpeakerMuteGuard, State,
    SttError, SttResult, WorkerEvent, WorkerRequest, spawn_model_download, spawn_stt_worker,
    validate_model_dir,
};

use async_channel::{Receiver, Sender};

pub(crate) struct Session {
    recorder: Option<AudioRecorder>,
    worker_tx: Option<Sender<WorkerRequest>>,
    event_tx: Sender<WorkerEvent>,
    model_config: ModelConfig,
    worker_ready: bool,
    state: State,
    transcript: String,
    status: String,
    popup_recording_active: bool,
    download_file_name: String,
    download_completed_files: usize,
    download_total_files: usize,
    download_file_bytes: u64,
    download_file_total_bytes: Option<u64>,
    speaker_mute: Option<SpeakerMuteGuard>,
    worker_use_gpu: Option<bool>,
}

impl Session {
    pub(crate) fn new() -> SttResult<(Self, Receiver<WorkerEvent>)> {
        let model_config = ModelConfig::load()?;
        let (event_tx, event_rx) = async_channel::unbounded();
        let (worker_tx, mut state, mut status) =
            start_worker_if_model_is_installed(&model_config, event_tx.clone());
        let worker_use_gpu = worker_tx.as_ref().map(|_| model_config.use_gpu());

        let recorder = match AudioRecorder::new() {
            Ok(recorder) => Some(recorder),
            Err(error) => {
                if state == State::ModelMissing {
                    status = format!("{status}; audio recorder unavailable: {error}");
                } else {
                    state = State::Error;
                    status = error.to_string();
                }
                None
            }
        };

        Ok((
            Self {
                recorder,
                worker_tx,
                event_tx,
                model_config,
                worker_ready: false,
                state,
                transcript: String::new(),
                status,
                popup_recording_active: false,
                download_file_name: String::new(),
                download_completed_files: 0,
                download_total_files: 0,
                download_file_bytes: 0,
                download_file_total_bytes: None,
                speaker_mute: None,
                worker_use_gpu,
            },
            event_rx,
        ))
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            state_label: self.state.as_str(),
            worker_ready: self.worker_ready,
            recorder_available: self.recorder.is_some(),
            transcript: self.transcript.clone(),
            status: self.status.clone(),
            model_can_download: self.state == State::ModelMissing,
            model_downloading: self.state == State::Downloading,
            model_download_files_percent: self.download_files_percent(),
            model_download_files_label: self.download_files_label(),
            model_download_file_percent: self.download_file_percent(),
            model_download_file_known: self.download_file_total_bytes.is_some(),
            model_download_file_label: self.download_file_label(),
            model_dir: self.model_config.model_dir().display().to_string(),
            config_path: self.model_config.config_path().display().to_string(),
            use_gpu: self.model_config.use_gpu(),
            auto_mute_speakers: self.model_config.auto_mute_speakers(),
        }
    }

    pub(crate) fn popup_label(&self) -> String {
        match self.state {
            State::Loading => "STT loading".to_string(),
            State::ModelMissing => "Model missing".to_string(),
            State::Downloading => "Downloading model".to_string(),
            State::Idle if self.transcript.is_empty() => "Hotkey active".to_string(),
            State::Idle => "Transcription complete".to_string(),
            State::Recording => "Listening...".to_string(),
            State::Transcribing => "Transcribing...".to_string(),
            State::Error => "STT unavailable".to_string(),
        }
    }

    pub(crate) fn popup_transcript(&self) -> String {
        self.transcript.clone()
    }

    pub(crate) fn should_keep_popup_open_after_release(&self) -> bool {
        self.state == State::Transcribing
    }

    pub(crate) fn start_model_download(&mut self) {
        if self.state == State::Downloading {
            return;
        }

        if validate_model_dir(self.model_config.model_dir()).is_ok() {
            self.start_worker();
            return;
        }

        self.worker_tx = None;
        self.worker_use_gpu = None;
        self.worker_ready = false;
        self.state = State::Downloading;
        self.status = format!(
            "Downloading STT model to {}",
            self.model_config.model_dir().display()
        );
        self.download_file_name.clear();
        self.download_completed_files = 0;
        self.download_total_files = 0;
        self.download_file_bytes = 0;
        self.download_file_total_bytes = None;

        if let Err(error) = spawn_model_download(self.model_config.clone(), self.event_tx.clone()) {
            self.state = State::ModelMissing;
            self.status = format!("Model download failed: {error}");
        }
    }

    pub(crate) fn set_use_gpu(&mut self, enabled: bool) {
        if let Err(error) = self.model_config.set_use_gpu(enabled) {
            self.status = format!("Failed to save GPU setting: {error}");
            return;
        }

        if self.state == State::Downloading {
            self.status = if enabled {
                "GPU inference enabled; model download still running".to_string()
            } else {
                "GPU inference disabled; model download still running".to_string()
            };
            return;
        }

        if self.state == State::Loading {
            self.status = if enabled {
                "GPU inference enabled; it will apply after the current model load".to_string()
            } else {
                "GPU inference disabled; it will apply after the current model load".to_string()
            };
            return;
        }

        if matches!(self.state, State::Recording | State::Transcribing) {
            self.status = if enabled {
                "GPU inference enabled; it will apply after this transcription".to_string()
            } else {
                "GPU inference disabled; it will apply after this transcription".to_string()
            };
            return;
        }

        if validate_model_dir(self.model_config.model_dir()).is_ok() {
            self.start_worker();
            return;
        }

        self.status = if enabled {
            "GPU inference enabled".to_string()
        } else {
            "GPU inference disabled".to_string()
        };
    }

    pub(crate) fn set_auto_mute_speakers(&mut self, enabled: bool) {
        if let Err(error) = self.model_config.set_auto_mute_speakers(enabled) {
            self.status = format!("Failed to save auto-mute setting: {error}");
            return;
        }

        if enabled {
            if self.state == State::Recording
                && self.speaker_mute.is_none()
                && !self.mute_speakers_for_recording()
            {
                return;
            }
        } else {
            self.restore_speakers_after_recording();
        }

        self.status = if enabled {
            "Speaker auto-mute enabled".to_string()
        } else {
            "Speaker auto-mute disabled".to_string()
        };
    }

    pub(crate) fn popup_opened(&mut self) {
        self.poll_recorder_error();

        if !self.can_record() {
            self.status = self.disabled_status();
            return;
        }

        let Some(recorder) = &self.recorder else {
            self.set_error(SttError::audio_device("audio recorder is unavailable"));
            return;
        };

        match recorder.start() {
            Ok(()) => {
                self.transcript.clear();
                self.state = State::Recording;
                self.status = "Listening for speech".to_string();
                self.popup_recording_active = true;
                let _ = self.mute_speakers_for_recording();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn popup_released(&mut self) {
        self.poll_recorder_error();

        if !self.popup_recording_active {
            return;
        }

        self.popup_recording_active = false;

        if self.state != State::Recording {
            self.restore_speakers_after_recording();
            return;
        }

        let Some(recorder) = &self.recorder else {
            self.set_error(SttError::audio_device("audio recorder is unavailable"));
            return;
        };

        let stop_result = recorder.stop();
        self.restore_speakers_after_recording();

        match stop_result {
            Ok(recording) => {
                if recording.is_shorter_than(MIN_RECORDING_DURATION) {
                    self.state = State::Idle;
                    self.status = "Ready".to_string();
                    return;
                }

                self.state = State::Transcribing;
                self.status = "Transcribing captured audio".to_string();
                if recording.clipped() {
                    self.transcript =
                        "recording hit the 60 second cap; transcribing captured audio".to_string();
                }

                let Some(worker_tx) = &self.worker_tx else {
                    self.set_error(SttError::worker_channel("STT worker is unavailable"));
                    return;
                };

                if worker_tx
                    .try_send(WorkerRequest::Transcribe(recording))
                    .is_err()
                {
                    self.set_error(SttError::worker_channel(
                        "could not send recording to STT worker",
                    ));
                }
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn apply_worker_event(&mut self, event: WorkerEvent) {
        self.poll_recorder_error();

        match event {
            WorkerEvent::Ready => {
                self.worker_ready = true;
                if self.recorder.is_some() && self.state == State::Loading {
                    self.state = State::Idle;
                    self.status = self.ready_status();
                    self.restart_worker_if_gpu_setting_changed();
                }
            }
            WorkerEvent::Transcript(transcript) => {
                self.transcript = transcript;
                self.state = State::Idle;
                self.status = self.ready_status();
                self.popup_recording_active = false;
                self.restart_worker_if_gpu_setting_changed();
            }
            WorkerEvent::Error(message) => {
                self.set_error(SttError::speech_to_text(message));
            }
            WorkerEvent::ModelDownloadProgress {
                file_name,
                completed_files,
                total_files,
                file_downloaded_bytes,
                file_total_bytes,
            } => {
                self.download_file_name = file_name;
                self.download_completed_files = completed_files;
                self.download_total_files = total_files;
                self.download_file_bytes = file_downloaded_bytes;
                self.download_file_total_bytes = file_total_bytes;
                self.state = State::Downloading;
                self.status = self.download_file_label();
            }
            WorkerEvent::ModelDownloadFinished => {
                self.download_file_name.clear();
                self.download_completed_files = 0;
                self.download_total_files = 0;
                self.download_file_bytes = 0;
                self.download_file_total_bytes = None;
                self.start_worker();
            }
            WorkerEvent::ModelDownloadError(message) => {
                self.download_file_name.clear();
                self.download_completed_files = 0;
                self.download_total_files = 0;
                self.download_file_bytes = 0;
                self.download_file_total_bytes = None;
                self.worker_use_gpu = None;
                self.worker_ready = false;
                self.worker_tx = None;
                self.state = State::ModelMissing;
                self.status = format!("Model download failed: {message}");
            }
        }
    }

    pub(crate) fn stop_recording_for_shutdown(&mut self) {
        if !self.popup_recording_active && self.state != State::Recording {
            return;
        }

        self.popup_recording_active = false;
        let stop_result = self.recorder.as_ref().map(AudioRecorder::stop);
        self.restore_speakers_after_recording();
        match stop_result {
            Some(Ok(_recording)) => {
                if self.state == State::Recording {
                    self.state = State::Idle;
                    self.status = "Recording stopped before shutdown".to_string();
                }
            }
            Some(Err(error)) => self.set_error(error),
            None => self.set_error(SttError::audio_device("audio recorder is unavailable")),
        }
    }

    fn can_record(&self) -> bool {
        self.worker_ready
            && self.worker_tx.is_some()
            && self.recorder.is_some()
            && self.state == State::Idle
    }

    fn disabled_status(&self) -> String {
        match self.state {
            State::Error => self.status.clone(),
            State::ModelMissing => "Download the STT model before recording".to_string(),
            State::Downloading => "STT model download is still running".to_string(),
            State::Loading => "STT worker is loading".to_string(),
            State::Recording => "Already recording".to_string(),
            State::Transcribing => "Transcription is still running".to_string(),
            State::Idle if !self.worker_ready => "STT worker is loading".to_string(),
            State::Idle if self.recorder.is_none() => "Audio recorder is unavailable".to_string(),
            State::Idle => "STT is not ready".to_string(),
        }
    }

    fn poll_recorder_error(&mut self) {
        let stream_error = self.recorder.as_ref().map(AudioRecorder::take_stream_error);

        match stream_error {
            Some(Ok(Some(message))) => self.set_error(SttError::audio_stream(message)),
            Some(Ok(None)) | None => {}
            Some(Err(error)) => self.set_error(error),
        }
    }

    fn set_error(&mut self, error: SttError) {
        if (self.popup_recording_active || self.state == State::Recording)
            && let Some(recorder) = &self.recorder
        {
            let _ = recorder.stop();
        }
        self.restore_speakers_after_recording();

        self.state = State::Error;
        self.status = error.to_string();
        self.popup_recording_active = false;
    }

    fn mute_speakers_for_recording(&mut self) -> bool {
        if !self.model_config.auto_mute_speakers() || self.speaker_mute.is_some() {
            return true;
        }

        match SpeakerMuteGuard::mute_default_sink() {
            Ok(guard) => {
                self.speaker_mute = Some(guard);
                true
            }
            Err(error) => {
                self.status = format!("Listening for speech; speaker auto-mute failed: {error}");
                false
            }
        }
    }

    fn restore_speakers_after_recording(&mut self) {
        let Some(mut speaker_mute) = self.speaker_mute.take() else {
            return;
        };

        if let Err(error) = speaker_mute.restore() {
            self.status = format!("Speaker auto-mute restore failed: {error}");
        }
    }

    fn start_worker(&mut self) {
        let (worker_tx, state, status) =
            start_worker_if_model_is_installed(&self.model_config, self.event_tx.clone());

        self.worker_use_gpu = worker_tx.as_ref().map(|_| self.model_config.use_gpu());
        self.worker_tx = worker_tx;
        self.worker_ready = false;
        self.state = state;
        self.status = status;
    }

    fn restart_worker_if_gpu_setting_changed(&mut self) {
        if self.worker_use_gpu == Some(self.model_config.use_gpu()) {
            return;
        }

        if validate_model_dir(self.model_config.model_dir()).is_ok() {
            self.start_worker();
        }
    }

    fn ready_status(&self) -> String {
        if self.model_config.use_gpu() {
            "Ready; GPU inference".to_string()
        } else {
            "Ready; CPU inference".to_string()
        }
    }

    fn download_files_percent(&self) -> f32 {
        percent(
            self.download_completed_files as u64,
            self.download_total_files as u64,
        )
    }

    fn download_files_label(&self) -> String {
        if self.download_total_files == 0 {
            return "Preparing".to_string();
        }

        format!(
            "{}/{} complete",
            self.download_completed_files, self.download_total_files
        )
    }

    fn download_file_percent(&self) -> f32 {
        if let Some(file_total_bytes) = self.download_file_total_bytes {
            return percent(self.download_file_bytes, file_total_bytes);
        }

        0.0
    }

    fn download_file_label(&self) -> String {
        if self.state != State::Downloading {
            return format!(
                "Model directory: {}",
                self.model_config.model_dir().display()
            );
        }

        let file_label = if self.download_file_name.is_empty() {
            "Preparing".to_string()
        } else {
            self.download_file_name.clone()
        };

        match self.download_file_total_bytes {
            Some(file_total_bytes) if file_total_bytes > 0 => format!(
                "{file_label} ({:.0}%)",
                percent(self.download_file_bytes, file_total_bytes)
            ),
            _ => file_label,
        }
    }
}

fn percent(downloaded_bytes: u64, total_bytes: u64) -> f32 {
    if total_bytes == 0 {
        return 0.0;
    }

    ((downloaded_bytes as f32 / total_bytes as f32) * 100.0).clamp(0.0, 100.0)
}

fn start_worker_if_model_is_installed(
    model_config: &ModelConfig,
    event_tx: Sender<WorkerEvent>,
) -> (Option<Sender<WorkerRequest>>, State, String) {
    let model_dir = model_config.model_dir().to_path_buf();

    if let Err(error) = validate_model_dir(&model_dir) {
        return (
            None,
            State::ModelMissing,
            format!(
                "STT model is not installed at {}; config: {}; {error}",
                model_dir.display(),
                model_config.config_path().display()
            ),
        );
    }

    let use_gpu = model_config.use_gpu();

    match spawn_stt_worker(model_dir.clone(), use_gpu, event_tx) {
        Ok(worker_tx) => (
            Some(worker_tx),
            State::Loading,
            format!(
                "Loading STT model from {} ({})",
                model_dir.display(),
                if use_gpu { "GPU" } else { "CPU" },
            ),
        ),
        Err(error) => (None, State::Error, error.to_string()),
    }
}
