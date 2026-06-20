use crate::error::{AppResult, ResultContext};
use crate::hotkey::{BackendKind, Controller};
use crate::windows::{
    HOTKEY_WINDOW_HEIGHT, HOTKEY_WINDOW_WIDTH, HotkeyWindowPlacement, hotkey_window_placement,
};

use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Pixels, Render, Styled, Window,
    WindowBackgroundAppearance, WindowHandle, WindowKind, WindowOptions, div, px, rgb, rgba, size,
};
use gpui_component::StyledExt;

#[cfg(all(target_os = "linux", feature = "wayland"))]
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

const HOTKEY_WINDOW_TITLE: &str = "Hotkey Overlay";

pub(crate) struct HotkeyWindow {
    backend_kind: BackendKind,
    is_visible: bool,
    stt_label: String,
    transcript: String,
}

impl HotkeyWindow {
    pub(crate) fn new(
        backend_kind: BackendKind,
        stt_label: String,
        transcript: String,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            backend_kind,
            is_visible: true,
            stt_label,
            transcript,
        }
    }

    pub(crate) fn show(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        stt_label: String,
        transcript: String,
    ) {
        self.is_visible = true;
        self.stt_label = stt_label;
        self.transcript = transcript;
        window.resize(size(px(HOTKEY_WINDOW_WIDTH), px(HOTKEY_WINDOW_HEIGHT)));
        cx.notify();
    }

    pub(crate) fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_visible = false;
        window.resize(size(px(1.0), px(1.0)));
        cx.notify();
    }

    pub(crate) fn set_content(
        &mut self,
        stt_label: String,
        transcript: String,
        cx: &mut Context<Self>,
    ) {
        self.stt_label = stt_label;
        self.transcript = transcript;
        cx.notify();
    }
}

impl Render for HotkeyWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if !self.is_visible {
            return div().size_full().hidden();
        }

        let mut container = div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .bg(rgba(0x111827f2))
            .border_1()
            .border_color(rgb(0x38bdf8))
            .text_color(rgb(0xf8fafc))
            .child(
                div()
                    .text_2xl()
                    .font_semibold()
                    .child(self.stt_label.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcbd5e1))
                    .child(format!("Release {} to close", Controller::hotkey_label())),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x93c5fd))
                    .child(self.backend_kind.label()),
            );

        if !self.transcript.is_empty() {
            container = container
                .child(
                    div()
                        .w_full()
                        .h(px(104.0))
                        .overflow_hidden()
                        .border_1()
                        .border_color(rgb(0x475569))
                        .rounded_md()
                        .bg(rgba(0x02061799))
                        .p_3()
                        .text_sm()
                        .text_color(rgb(0xf8fafc))
                        .child(self.transcript.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x86efac))
                        .child("Copied to clipboard"),
                );
        }

        container
    }
}

pub(crate) fn open_hotkey_window(
    app: &mut App,
    backend_kind: BackendKind,
    stt_label: String,
    transcript: String,
) -> AppResult<WindowHandle<HotkeyWindow>> {
    let placement = hotkey_window_placement(app);

    #[cfg(all(target_os = "linux", feature = "wayland"))]
    if app.compositor_name() == "Wayland" {
        return open_hotkey_window_with_kind(
            app,
            backend_kind,
            stt_label,
            transcript,
            layer_shell_window_kind(placement.layer_shell_margin),
            placement,
        )
        .context("open Wayland layer-shell hotkey overlay");
    }

    open_hotkey_window_with_kind(
        app,
        backend_kind,
        stt_label,
        transcript,
        WindowKind::PopUp,
        placement,
    )
}

fn open_hotkey_window_with_kind(
    app: &mut App,
    backend_kind: BackendKind,
    stt_label: String,
    transcript: String,
    kind: WindowKind,
    placement: HotkeyWindowPlacement,
) -> AppResult<WindowHandle<HotkeyWindow>> {
    let options = hotkey_window_options(kind, placement);

    app.open_window(options, move |window, app| {
        window.set_window_title(HOTKEY_WINDOW_TITLE);
        app.new(|cx| HotkeyWindow::new(backend_kind, stt_label, transcript, cx))
    })
    .context("open hotkey overlay window")
}

fn hotkey_window_options(kind: WindowKind, placement: HotkeyWindowPlacement) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(placement.window_bounds),
        titlebar: None,
        focus: false,
        show: true,
        kind,
        is_resizable: false,
        is_minimizable: false,
        display_id: placement.display_id,
        window_background: WindowBackgroundAppearance::Transparent,
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
fn layer_shell_window_kind(margin: (Pixels, Pixels, Pixels, Pixels)) -> WindowKind {
    WindowKind::LayerShell(LayerShellOptions {
        namespace: "auto-scribe-overlay".to_string(),
        layer: Layer::Overlay,
        anchor: Anchor::BOTTOM | Anchor::LEFT,
        margin: Some(margin),
        keyboard_interactivity: KeyboardInteractivity::None,
        ..Default::default()
    })
}
