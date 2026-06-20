use crate::hotkey::{Event, GlobalRuntime, HOTKEY_LABEL, Runtime, RuntimeEvent};

use crate::error::{AppResult, ResultContext};
use async_channel::Sender;
use global_hotkey::{
    GlobalHotKeyEvent, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

pub(crate) fn start(sender: Sender<RuntimeEvent>) -> AppResult<Runtime> {
    let manager =
        global_hotkey::GlobalHotKeyManager::new().context("create global hotkey manager")?;
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    let hotkey_id = hotkey.id();

    manager
        .register(hotkey)
        .with_context(|| format!("register {HOTKEY_LABEL}"))?;

    let status_sender = sender.clone();
    let _ = status_sender.try_send(RuntimeEvent::Status(format!(
        "Registered {HOTKEY_LABEL}; hold it to show the overlay"
    )));

    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.id() != hotkey_id {
            return;
        }

        let mapped_event = match event.state() {
            HotKeyState::Pressed => Event::Pressed,
            HotKeyState::Released => Event::Released,
        };

        let _ = sender.try_send(RuntimeEvent::Hotkey(mapped_event));
    }));

    Ok(Runtime::Global {
        _runtime: GlobalRuntime::new(manager, hotkey),
    })
}
