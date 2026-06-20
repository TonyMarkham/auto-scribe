use crate::{
    hotkey::{BackendKind, Event, HOTKEY_LABEL, Runtime, RuntimeEvent, Snapshot},
    stt::{Session, WorkerEvent},
    windows::{HotkeyWindow, open_hotkey_window},
};

use gpui::{ClipboardItem, Context, Subscription, Task, WindowHandle};
use std::{borrow::BorrowMut, time::Duration};

const TRANSCRIPT_POPUP_CLOSE_DELAY: Duration = Duration::from_secs(5);

pub(crate) struct Controller {
    runtime: Option<Runtime>,
    event_task: Option<Task<()>>,
    stt_event_task: Option<Task<()>>,
    window_closed_subscription: Option<Subscription>,
    backend_kind: BackendKind,
    popup_window: Option<WindowHandle<HotkeyWindow>>,
    popup_visible: bool,
    popup_close_generation: u64,
    is_hotkey_down: bool,
    status: String,
    stt: Session,
}

impl Controller {
    pub(crate) fn new(backend_kind: BackendKind, stt: Session, _: &mut Context<Self>) -> Self {
        Self {
            runtime: None,
            event_task: None,
            stt_event_task: None,
            window_closed_subscription: None,
            backend_kind,
            popup_window: None,
            popup_visible: false,
            popup_close_generation: 0,
            is_hotkey_down: false,
            status: format!("Starting {} backend", backend_kind.label()),
            stt,
        }
    }

    pub(crate) fn hotkey_label() -> &'static str {
        HOTKEY_LABEL
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        let stt = self.stt.snapshot();
        Snapshot {
            hotkey_label: HOTKEY_LABEL,
            backend_label: self.backend_kind.label(),
            is_hotkey_down: self.is_hotkey_down,
            popup_open: self.popup_visible,
            status: self.status.clone(),
            stt_state_label: stt.state_label,
            stt_worker_ready: stt.worker_ready,
            stt_recorder_available: stt.recorder_available,
            stt_transcript: stt.transcript,
            stt_status: stt.status,
            stt_model_can_download: stt.model_can_download,
            stt_model_downloading: stt.model_downloading,
            stt_model_download_files_percent: stt.model_download_files_percent,
            stt_model_download_files_label: stt.model_download_files_label,
            stt_model_download_file_percent: stt.model_download_file_percent,
            stt_model_download_file_known: stt.model_download_file_known,
            stt_model_download_file_label: stt.model_download_file_label,
            stt_model_dir: stt.model_dir,
            stt_config_path: stt.config_path,
            stt_use_gpu: stt.use_gpu,
            stt_auto_mute_speakers: stt.auto_mute_speakers,
        }
    }

    pub(crate) fn install_runtime(
        &mut self,
        runtime: Runtime,
        event_task: Task<()>,
        stt_event_task: Task<()>,
        window_closed_subscription: Subscription,
        cx: &mut Context<Self>,
    ) {
        self.runtime = Some(runtime);
        self.event_task = Some(event_task);
        self.stt_event_task = Some(stt_event_task);
        self.window_closed_subscription = Some(window_closed_subscription);
        cx.notify();
    }

    pub(crate) fn apply_runtime_event(&mut self, event: RuntimeEvent, cx: &mut Context<Self>) {
        match event {
            RuntimeEvent::Hotkey(Event::Pressed) => self.hotkey_pressed(cx),
            RuntimeEvent::Hotkey(Event::Released) => self.hotkey_released(cx),
            RuntimeEvent::Status(message) => {
                self.status = message;
                cx.notify();
            }
            RuntimeEvent::Error(message) => {
                self.status = message;
                cx.notify();
            }
        }
    }

    pub(crate) fn apply_stt_event(&mut self, event: WorkerEvent, cx: &mut Context<Self>) {
        let completed_transcript = match &event {
            WorkerEvent::Transcript(transcript) => Some(transcript.clone()),
            WorkerEvent::Ready
            | WorkerEvent::Error(_)
            | WorkerEvent::ModelDownloadProgress { .. }
            | WorkerEvent::ModelDownloadFinished
            | WorkerEvent::ModelDownloadError(_) => None,
        };

        self.stt.apply_worker_event(event);
        self.update_popup_content(cx);

        if let Some(transcript) = completed_transcript {
            self.copy_transcript_to_clipboard(transcript, cx);
            self.schedule_popup_close_after_transcript(cx);
        }

        cx.notify();
    }

    pub(crate) fn download_model(&mut self, cx: &mut Context<Self>) {
        self.stt.start_model_download();
        cx.notify();
    }

    pub(crate) fn set_auto_mute_speakers(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.stt.set_auto_mute_speakers(enabled);
        cx.notify();
    }

    pub(crate) fn set_use_gpu(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.stt.set_use_gpu(enabled);
        cx.notify();
    }

    pub(crate) fn window_closed(&mut self, cx: &mut Context<Self>) {
        let popup_is_closed = self
            .popup_window
            .map(|handle| handle.update(cx, |_, _, _| ()).is_err())
            .unwrap_or(false);

        if popup_is_closed {
            self.popup_window = None;
            self.popup_visible = false;
            cx.notify();
        }

        let only_hidden_popup_remains = self.popup_window.is_some_and(|popup_window| {
            cx.windows()
                .into_iter()
                .all(|window| window.window_id() == popup_window.window_id())
        });

        if only_hidden_popup_remains {
            self.stt.stop_recording_for_shutdown();
            cx.quit();
        }
    }

    fn hotkey_pressed(&mut self, cx: &mut Context<Self>) {
        if self.is_hotkey_down {
            return;
        }

        self.popup_close_generation = self.popup_close_generation.wrapping_add(1);
        self.is_hotkey_down = true;
        self.status = "Hotkey is down".to_string();
        let popup_opened = self.show_or_open_popup(cx);

        if popup_opened {
            self.stt.popup_opened();
            self.update_popup_content(cx);
        }

        cx.notify();
    }

    fn hotkey_released(&mut self, cx: &mut Context<Self>) {
        self.is_hotkey_down = false;
        self.status = format!("Waiting for {}", HOTKEY_LABEL);
        self.stt.popup_released();
        self.update_popup_content(cx);

        if !self.stt.should_keep_popup_open_after_release() {
            self.hide_popup(cx);
        }

        cx.notify();
    }

    fn show_or_open_popup(&mut self, cx: &mut Context<Self>) -> bool {
        let popup_label = self.stt.popup_label();
        let popup_transcript = self.stt.popup_transcript();
        if let Some(window) = self.popup_window
            && window
                .update(cx, |popup, window, cx| {
                    popup.show(window, cx, popup_label, popup_transcript);
                })
                .is_err()
        {
            self.popup_window = None;
            self.popup_visible = false;
        }

        if self.popup_window.is_none() {
            let popup_label = self.stt.popup_label();
            let popup_transcript = self.stt.popup_transcript();
            match open_hotkey_window(
                cx.borrow_mut(),
                self.backend_kind,
                popup_label,
                popup_transcript,
            ) {
                Ok(window) => {
                    self.popup_window = Some(window);
                }
                Err(error) => {
                    self.status = format!("Failed to open hotkey window: {error}");
                    return false;
                }
            }
        }

        self.popup_visible = true;
        true
    }

    fn update_popup_content(&mut self, cx: &mut Context<Self>) {
        let Some(window) = self.popup_window else {
            return;
        };

        let popup_label = self.stt.popup_label();
        let popup_transcript = self.stt.popup_transcript();
        if window
            .update(cx, |popup, _, cx| {
                popup.set_content(popup_label, popup_transcript, cx);
            })
            .is_err()
        {
            self.popup_window = None;
            self.popup_visible = false;
        }
    }

    fn copy_transcript_to_clipboard(&mut self, transcript: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(transcript));
    }

    fn schedule_popup_close_after_transcript(&mut self, cx: &mut Context<Self>) {
        self.popup_close_generation = self.popup_close_generation.wrapping_add(1);
        let generation = self.popup_close_generation;

        cx.spawn(async move |controller, cx| {
            cx.background_executor()
                .timer(TRANSCRIPT_POPUP_CLOSE_DELAY)
                .await;
            let _ = controller.update(cx, |controller, cx| {
                controller.hide_popup_after_transcript(generation, cx);
            });
        })
        .detach();
    }

    fn hide_popup_after_transcript(&mut self, generation: u64, cx: &mut Context<Self>) {
        if self.is_hotkey_down || self.popup_close_generation != generation {
            return;
        }

        self.hide_popup(cx);
        cx.notify();
    }

    fn hide_popup(&mut self, cx: &mut Context<Self>) {
        self.popup_visible = false;

        if let Some(window) = self.popup_window
            && window
                .update(cx, |popup, window, cx| popup.hide(window, cx))
                .is_err()
        {
            self.popup_window = None;
        }
    }
}
