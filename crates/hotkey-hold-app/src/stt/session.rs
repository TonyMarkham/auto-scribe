use crate::stt::{
    AudioRecorder, MIN_RECORDING_DURATION, Snapshot, State, SttError, SttResult, WorkerEvent,
    WorkerRequest, model_dir_from_env_or_default, spawn_stt_worker,
};

use async_channel::{Receiver, Sender};

pub(crate) struct Session {
    recorder: Option<AudioRecorder>,
    worker_tx: Sender<WorkerRequest>,
    worker_ready: bool,
    state: State,
    transcript: String,
    status: String,
    popup_recording_active: bool,
}

impl Session {
    pub(crate) fn new() -> SttResult<(Self, Receiver<WorkerEvent>)> {
        let model_dir = model_dir_from_env_or_default();
        let (worker_tx, worker_rx) = spawn_stt_worker(model_dir.clone())?;
        let (recorder, state, status) = match AudioRecorder::new() {
            Ok(recorder) => (
                Some(recorder),
                State::Loading,
                format!("Loading STT model from {}", model_dir.display()),
            ),
            Err(error) => (None, State::Error, error.to_string()),
        };

        Ok((
            Self {
                recorder,
                worker_tx,
                worker_ready: false,
                state,
                transcript: String::new(),
                status,
                popup_recording_active: false,
            },
            worker_rx,
        ))
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            state_label: self.state.as_str(),
            worker_ready: self.worker_ready,
            recorder_available: self.recorder.is_some(),
            transcript: self.transcript.clone(),
            status: self.status.clone(),
        }
    }

    pub(crate) fn popup_label(&self) -> String {
        match self.state {
            State::Loading => "STT loading".to_string(),
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
            return;
        }

        let Some(recorder) = &self.recorder else {
            self.set_error(SttError::audio_device("audio recorder is unavailable"));
            return;
        };

        match recorder.stop() {
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

                if self
                    .worker_tx
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
                    self.status = "Ready".to_string();
                }
            }
            WorkerEvent::Transcript(transcript) => {
                self.transcript = transcript;
                self.state = State::Idle;
                self.status = "Ready".to_string();
                self.popup_recording_active = false;
            }
            WorkerEvent::Error(message) => {
                self.set_error(SttError::speech_to_text(message));
            }
        }
    }

    pub(crate) fn stop_recording_for_shutdown(&mut self) {
        if !self.popup_recording_active && self.state != State::Recording {
            return;
        }

        self.popup_recording_active = false;
        let stop_result = self.recorder.as_ref().map(AudioRecorder::stop);
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
        self.worker_ready && self.recorder.is_some() && self.state == State::Idle
    }

    fn disabled_status(&self) -> String {
        match self.state {
            State::Error => self.status.clone(),
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

        self.state = State::Error;
        self.status = error.to_string();
        self.popup_recording_active = false;
    }
}
