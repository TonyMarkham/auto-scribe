use crate::{
    app_state::AppState,
    audio_recorder::AudioRecorder,
    audio_recorder::AudioRecording,
    error::{WipError, WipResult},
    work_event::WorkerEvent,
    worker_request::WorkerRequest,
};

use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use eframe::egui::{self, Button, PointerButton, TextEdit, Vec2};
use parakeet_rs::Nemotron;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::{env, path::Path, path::PathBuf, thread, time::Duration};

pub(crate) const MIN_RECORDING_DURATION: Duration = Duration::from_millis(200);
pub(crate) const DEFAULT_MODEL_DIR: &str =
    "submodules/parakeet-rs/nemotron-speech-streaming-en-0.6b";
pub(crate) const TARGET_SAMPLE_RATE: u32 = 16_000;
pub(crate) const MAX_RECORDING_SECONDS: usize = 60;
pub(crate) const REQUIRED_MODEL_FILES: [&str; 4] = [
    "encoder.onnx",
    "encoder.onnx.data",
    "decoder_joint.onnx",
    "tokenizer.model",
];

pub struct App {
    status: AppState,
    transcript: String,
    recorder: Option<AudioRecorder>,
    worker_tx: Sender<WorkerRequest>,
    worker_rx: Receiver<WorkerEvent>,
    worker_ready: bool,
    recording_press_active: bool,
}

impl App {
    pub fn new(model_dir: PathBuf) -> WipResult<Self> {
        let (worker_tx, worker_rx) = spawn_stt_worker(model_dir)?;
        let (recorder, status, transcript) = match AudioRecorder::new() {
            Ok(recorder) => (Some(recorder), AppState::Loading, String::new()),
            Err(error) => (None, AppState::Error, error.to_string()),
        };

        Ok(Self {
            status,
            transcript,
            recorder,
            worker_tx,
            worker_rx,
            worker_ready: false,
            recording_press_active: false,
        })
    }

    fn poll_worker(&mut self) {
        loop {
            match self.worker_rx.try_recv() {
                Ok(event) => self.handle_worker_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.set_error(WipError::worker_channel("STT worker has stopped"));
                    break;
                }
            }
        }
    }

    fn poll_recorder(&mut self) {
        let Some(recorder) = &self.recorder else {
            return;
        };

        match recorder.take_stream_error() {
            Ok(Some(message)) => self.set_error(WipError::audio_stream(message)),
            Ok(None) => {}
            Err(error) => self.set_error(error),
        }
    }

    fn handle_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Ready => {
                self.worker_ready = true;
                if self.recorder.is_some() && self.status == AppState::Loading {
                    self.status = AppState::Idle;
                }
            }
            WorkerEvent::Transcript(transcript) => {
                self.transcript = transcript;
                self.status = AppState::Idle;
            }
            WorkerEvent::Error(message) => {
                self.set_error(WipError::stt(message));
            }
        }
    }

    fn handle_button_state(&mut self, primary_pressed_on_button: bool, primary_down: bool) {
        if primary_pressed_on_button && !self.recording_press_active {
            self.start_recording();
            if self.status == AppState::Recording {
                self.recording_press_active = true;
            }
        } else if !primary_down && self.recording_press_active {
            self.stop_recording();
            self.recording_press_active = false;
        }
    }

    fn start_recording(&mut self) {
        if !self.can_record() {
            return;
        }

        let Some(recorder) = &self.recorder else {
            self.set_error(WipError::audio_device("audio recorder is unavailable"));
            return;
        };

        match recorder.start() {
            Ok(()) => {
                self.transcript.clear();
                self.status = AppState::Recording;
            }
            Err(error) => self.set_error(error),
        }
    }

    fn stop_recording(&mut self) {
        if self.status != AppState::Recording {
            return;
        }

        let Some(recorder) = &self.recorder else {
            self.set_error(WipError::audio_device("audio recorder is unavailable"));
            return;
        };

        match recorder.stop() {
            Ok(recording) => {
                if recording.is_shorter_than(MIN_RECORDING_DURATION) {
                    self.status = AppState::Idle;
                    return;
                }

                self.status = AppState::Transcribing;
                if recording.clipped() {
                    self.transcript =
                        "recording hit the 60 second cap; transcribing captured audio".to_string();
                }

                if self
                    .worker_tx
                    .send(WorkerRequest::Transcribe(recording))
                    .is_err()
                {
                    self.set_error(WipError::worker_channel(
                        "could not send recording to STT worker",
                    ));
                }
            }
            Err(error) => self.set_error(error),
        }
    }

    fn can_record(&self) -> bool {
        self.worker_ready && self.recorder.is_some() && self.status == AppState::Idle
    }

    fn set_error(&mut self, error: WipError) {
        self.status = AppState::Error;
        self.transcript = error.to_string();
        self.recording_press_active = false;
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();
        self.poll_recorder();
        context.request_repaint_after(Duration::from_millis(33));

        egui::CentralPanel::default().show(context, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
            ui.vertical(|ui| {
                ui.label(format!("status: {}", self.status.as_str()));

                let button_enabled = self.can_record() || self.status == AppState::Recording;
                let button_label = match self.status {
                    AppState::Recording => "Recording...",
                    AppState::Transcribing => "Transcribing...",
                    AppState::Loading => "Loading...",
                    AppState::Error | AppState::Idle => "Hold to record",
                };
                let response = ui.add_enabled(
                    button_enabled,
                    Button::new(button_label).min_size(Vec2::new(ui.available_width(), 120.0)),
                );
                let (primary_pressed, primary_down) = context.input(|input| {
                    (
                        input.pointer.button_pressed(PointerButton::Primary),
                        input.pointer.button_down(PointerButton::Primary),
                    )
                });
                let primary_pressed_on_button =
                    button_enabled && response.hovered() && primary_pressed;
                self.handle_button_state(primary_pressed_on_button, primary_down);

                ui.add(
                    TextEdit::multiline(&mut self.transcript)
                        .desired_rows(10)
                        .interactive(false)
                        .hint_text("Transcript"),
                );
            });
        });
    }
}

pub fn model_dir_from_env_or_default() -> PathBuf {
    env::var_os("NEMOTRON_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL_DIR))
}

pub fn validate_model_dir(model_dir: &Path) -> WipResult<()> {
    if !model_dir.is_dir() {
        return Err(WipError::model_path(format!(
            "model directory does not exist: {}",
            model_dir.display()
        )));
    }

    for file_name in REQUIRED_MODEL_FILES {
        let file_path = model_dir.join(file_name);
        if !file_path.is_file() {
            return Err(WipError::model_path(format!(
                "required model file is missing: {}",
                file_path.display()
            )));
        }
    }

    Ok(())
}

pub fn spawn_stt_worker(
    model_dir: PathBuf,
) -> WipResult<(Sender<WorkerRequest>, Receiver<WorkerEvent>)> {
    let (request_tx, request_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let error_tx = event_tx.clone();

    let _worker_thread = thread::Builder::new()
        .name("wip-stt-worker".to_string())
        .spawn(move || {
            if let Err(error) = run_stt_worker(model_dir, request_rx, event_tx) {
                let _ = error_tx.send(WorkerEvent::Error(error.to_string()));
            }
        })
        .map_err(|error| WipError::stt(error.to_string()))?;

    Ok((request_tx, event_rx))
}

fn run_stt_worker(
    model_dir: PathBuf,
    request_rx: Receiver<WorkerRequest>,
    event_tx: Sender<WorkerEvent>,
) -> WipResult<()> {
    validate_model_dir(&model_dir)?;
    let mut model = Nemotron::from_pretrained(&model_dir, None)
        .map_err(|error| WipError::stt(error.to_string()))?;
    send_event(&event_tx, WorkerEvent::Ready)?;

    while let Ok(request) = request_rx.recv() {
        match request {
            WorkerRequest::Transcribe(recording) => {
                let result =
                    prepare_audio(recording).and_then(|audio| transcribe_audio(&mut model, &audio));
                match result {
                    Ok(transcript) => send_event(&event_tx, WorkerEvent::Transcript(transcript))?,
                    Err(error) => send_event(&event_tx, WorkerEvent::Error(error.to_string()))?,
                }
            }
        }
    }

    Ok(())
}

fn send_event(event_tx: &Sender<WorkerEvent>, event: WorkerEvent) -> WipResult<()> {
    event_tx
        .send(event)
        .map_err(|_| WipError::worker_channel("UI event receiver has disconnected"))
}

fn transcribe_audio(model: &mut Nemotron, audio_16k_mono: &[f32]) -> WipResult<String> {
    model
        .transcribe_audio(audio_16k_mono)
        .map_err(|error| WipError::stt(error.to_string()))
}

fn prepare_audio(recording: AudioRecording) -> WipResult<Vec<f32>> {
    let (samples, sample_rate) = recording.into_parts();
    let mut audio = if sample_rate == TARGET_SAMPLE_RATE {
        samples
    } else {
        resample_to_16k(samples, sample_rate)?
    };
    sanitize_samples(&mut audio);
    Ok(audio)
}

fn sanitize_samples(samples: &mut [f32]) {
    for sample in samples {
        if sample.is_finite() {
            *sample = sample.clamp(-1.0, 1.0);
        } else {
            *sample = 0.0;
        }
    }
}

fn resample_to_16k(samples: Vec<f32>, source_sample_rate: u32) -> WipResult<Vec<f32>> {
    if source_sample_rate == 0 {
        return Err(WipError::resampling(
            "source sample rate must be greater than zero",
        ));
    }

    if samples.is_empty() {
        return Ok(samples);
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let ratio = f64::from(TARGET_SAMPLE_RATE) / f64::from(source_sample_rate);
    let input_len = samples.len();
    let mut resampler = SincFixedIn::<f32>::new(ratio, 2.0, params, input_len, 1)
        .map_err(|error| WipError::resampling(error.to_string()))?;
    let input = [samples];
    let mut output_channels = resampler
        .process(&input, None)
        .map_err(|error| WipError::resampling(error.to_string()))?;

    if output_channels.is_empty() {
        return Err(WipError::resampling(
            "resampler returned no output channels",
        ));
    }

    Ok(output_channels.remove(0))
}
