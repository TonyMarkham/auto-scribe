use crate::{
    MINIMUM_HEIGHT, MINIMUM_WIDTH, WINDOW_TITLE,
    hotkey::Controller,
    icon::{APP_ID, window_icon},
    windows::window_resize_handles,
};

use crate::error::{AppResult, ResultContext};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions, div, px, rgb, size,
};
use gpui_component::{ActiveTheme, Root, StyledExt, TitleBar, v_flex};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

pub(crate) struct MainWindow {
    controller: Entity<Controller>,
}

impl MainWindow {
    pub(crate) fn new(controller: Entity<Controller>, _: &mut Context<Self>) -> Self {
        Self { controller }
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self
            .controller
            .read_with(cx, |controller, _| controller.snapshot());
        let transcript = if snapshot.stt_transcript.is_empty() {
            "Transcript".to_string()
        } else {
            snapshot.stt_transcript.clone()
        };

        let title_bar = TitleBar::new();
        #[cfg(not(target_os = "macos"))]
        let title_bar = title_bar.child(WINDOW_TITLE);

        v_flex()
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(title_bar)
            .child(
                v_flex()
                    .id("window-body")
                    .size_full()
                    .gap_4()
                    .p_6()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(div().text_2xl().font_semibold().child("Auto Scribe"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("The overlay window is shown while held and hidden on release."),
                    )
                    .child(status_row("Hotkey", snapshot.hotkey_label))
                    .child(status_row("Backend", snapshot.backend_label))
                    .child(status_row(
                        "State",
                        if snapshot.is_hotkey_down {
                            "held"
                        } else {
                            "idle"
                        },
                    ))
                    .child(status_row(
                        "Overlay",
                        if snapshot.popup_open {
                            "visible"
                        } else {
                            "hidden"
                        },
                    ))
                    .child(status_row("STT", snapshot.stt_state_label))
                    .child(status_row(
                        "Worker",
                        if snapshot.stt_worker_ready {
                            "ready"
                        } else {
                            "loading"
                        },
                    ))
                    .child(status_row(
                        "Recorder",
                        if snapshot.stt_recorder_available {
                            "available"
                        } else {
                            "unavailable"
                        },
                    ))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x60a5fa))
                            .child(snapshot.status),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x38bdf8))
                            .child(snapshot.stt_status),
                    )
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .min_h(px(160.0))
                            .border_1()
                            .border_color(rgb(0x334155))
                            .rounded_md()
                            .p_3()
                            .text_sm()
                            .child(transcript),
                    ),
            )
            .child(window_resize_handles(window))
    }
}

pub(crate) fn open_main_window(
    app: &mut App,
    controller: Entity<Controller>,
    window_bounds: WindowBounds,
) -> AppResult<()> {
    let options = WindowOptions {
        window_bounds: Some(window_bounds),
        window_min_size: Some(size(px(MINIMUM_WIDTH), px(MINIMUM_HEIGHT))),
        titlebar: Some(TitleBar::title_bar_options()),
        app_id: Some(APP_ID.to_string()),
        icon: Some(window_icon()),
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    };

    app.open_window(options, move |window, app| {
        window.set_window_title(WINDOW_TITLE);
        let main_window = app.new(|cx| MainWindow::new(controller, cx));
        app.new(|cx| Root::new(main_window, window, cx))
    })
    .context("open main window")?;

    Ok(())
}

fn status_row(label: &'static str, value: impl Into<String>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .border_1()
        .border_color(rgb(0x334155))
        .rounded_md()
        .px_3()
        .py_2()
        .child(div().text_sm().text_color(rgb(0x94a3b8)).child(label))
        .child(div().text_sm().font_semibold().child(value.into()))
}
