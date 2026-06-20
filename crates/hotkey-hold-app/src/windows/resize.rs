use crate::{RESIZE_CORNER_SIZE, RESIZE_EDGE_SIZE};

use gpui::{
    AnyElement, App, CursorStyle, Decorations, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, ResizeEdge, Styled, Tiling, Window, div,
};

pub(crate) fn window_resize_handles(window: &Window) -> impl IntoElement {
    let Decorations::Client { tiling } = window.window_decorations() else {
        return div();
    };

    div().absolute().size_full().children([
        resize_handle(ResizeEdge::Top, tiling),
        resize_handle(ResizeEdge::Right, tiling),
        resize_handle(ResizeEdge::Bottom, tiling),
        resize_handle(ResizeEdge::Left, tiling),
        resize_handle(ResizeEdge::TopLeft, tiling),
        resize_handle(ResizeEdge::TopRight, tiling),
        resize_handle(ResizeEdge::BottomRight, tiling),
        resize_handle(ResizeEdge::BottomLeft, tiling),
    ])
}

fn resize_handle(edge: ResizeEdge, tiling: Tiling) -> AnyElement {
    if !resize_edge_enabled(edge, tiling) {
        return div().into_any_element();
    }

    let handle = div()
        .absolute()
        .cursor(cursor_style_for_resize_edge(edge))
        .on_mouse_down(MouseButton::Left, resize_handler_for_edge(edge));

    match edge {
        ResizeEdge::Top => handle.top_0().left_0().right_0().h(RESIZE_EDGE_SIZE),
        ResizeEdge::Right => handle.top_0().right_0().bottom_0().w(RESIZE_EDGE_SIZE),
        ResizeEdge::Bottom => handle.right_0().bottom_0().left_0().h(RESIZE_EDGE_SIZE),
        ResizeEdge::Left => handle.top_0().bottom_0().left_0().w(RESIZE_EDGE_SIZE),
        ResizeEdge::TopLeft => handle
            .top_0()
            .left_0()
            .w(RESIZE_CORNER_SIZE)
            .h(RESIZE_CORNER_SIZE),
        ResizeEdge::TopRight => handle
            .top_0()
            .right_0()
            .w(RESIZE_CORNER_SIZE)
            .h(RESIZE_CORNER_SIZE),
        ResizeEdge::BottomRight => handle
            .right_0()
            .bottom_0()
            .w(RESIZE_CORNER_SIZE)
            .h(RESIZE_CORNER_SIZE),
        ResizeEdge::BottomLeft => handle
            .bottom_0()
            .left_0()
            .w(RESIZE_CORNER_SIZE)
            .h(RESIZE_CORNER_SIZE),
    }
    .into_any_element()
}

fn resize_handler_for_edge(edge: ResizeEdge) -> fn(&MouseDownEvent, &mut Window, &mut App) {
    match edge {
        ResizeEdge::Top => resize_top,
        ResizeEdge::Right => resize_right,
        ResizeEdge::Bottom => resize_bottom,
        ResizeEdge::Left => resize_left,
        ResizeEdge::TopLeft => resize_top_left,
        ResizeEdge::TopRight => resize_top_right,
        ResizeEdge::BottomRight => resize_bottom_right,
        ResizeEdge::BottomLeft => resize_bottom_left,
    }
}

fn resize_top(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::Top, window, app);
}

fn resize_right(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::Right, window, app);
}

fn resize_bottom(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::Bottom, window, app);
}

fn resize_left(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::Left, window, app);
}

fn resize_top_left(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::TopLeft, window, app);
}

fn resize_top_right(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::TopRight, window, app);
}

fn resize_bottom_right(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::BottomRight, window, app);
}

fn resize_bottom_left(_: &MouseDownEvent, window: &mut Window, app: &mut App) {
    start_resize(ResizeEdge::BottomLeft, window, app);
}

fn start_resize(edge: ResizeEdge, window: &mut Window, app: &mut App) {
    window.start_window_resize(edge);
    app.stop_propagation();
}

fn resize_edge_enabled(edge: ResizeEdge, tiling: Tiling) -> bool {
    match edge {
        ResizeEdge::Top => !tiling.top,
        ResizeEdge::Right => !tiling.right,
        ResizeEdge::Bottom => !tiling.bottom,
        ResizeEdge::Left => !tiling.left,
        ResizeEdge::TopLeft => !tiling.top && !tiling.left,
        ResizeEdge::TopRight => !tiling.top && !tiling.right,
        ResizeEdge::BottomRight => !tiling.bottom && !tiling.right,
        ResizeEdge::BottomLeft => !tiling.bottom && !tiling.left,
    }
}

fn cursor_style_for_resize_edge(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}
