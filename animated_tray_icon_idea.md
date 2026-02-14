# Animated Tray Icon Idea

System tray icons don't support GIFs natively on Windows or macOS. Animation can be simulated by cycling through static frames on a timer.

## Approach

1. Create frame PNGs for each animated state (e.g., 3-4 frames each for Recording and Processing)
2. When entering an animated state, start a timer that swaps the tray icon every ~500ms
3. When returning to Idle, stop the timer and set the static idle icon

## Frame Assets

```
crates/auto-scribe/resources/icons/
  idle.png
  recording_1.png
  recording_2.png
  recording_3.png
  processing_1.png
  processing_2.png
  processing_3.png
```

## Implementation Sketch

### TrayManager changes

Add animation state and a list of preloaded frames:

```rust
pub struct TrayManager {
    tray_icon: TrayIcon,
    settings_item_id: MenuId,
    exit_item_id: MenuId,
    animation_frames: HashMap<TrayIconState, Vec<Icon>>,
    current_frame: usize,
}
```

Preload all frames in `TrayManager::new()` using `include_bytes!` so there's no filesystem dependency.

### Animation driver

The main thread runs the `tao` event loop. Use `ControlFlow::WaitUntil(next_frame_time)` instead of `ControlFlow::Wait` while animating. On each tick:

```rust
Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
    tray_manager.advance_frame();
}
```

When idle, switch back to `ControlFlow::Wait` to avoid unnecessary CPU usage.

### TrayCommand changes

```rust
pub enum TrayCommand {
    SetState(TrayIconState),
    Shutdown,
}
```

No changes needed - `SetState(Recording)` starts the animation, `SetState(Idle)` stops it. The animation logic lives entirely in `TrayManager` on the main thread.

### Key considerations

- Preload and cache all `Icon` instances at startup to avoid allocation during animation
- Use `ControlFlow::WaitUntil` rather than a separate timer thread to keep everything on the main thread (TrayIcon is `!Send`)
- 2-4 frames at 400-500ms intervals is sufficient for a subtle pulse/spin effect
- Keep frame PNGs small (32x32 or 64x64) to minimize memory
