use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};

pub(crate) struct GlobalRuntime {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl GlobalRuntime {
    pub(crate) fn new(manager: GlobalHotKeyManager, hotkey: HotKey) -> Self {
        Self { manager, hotkey }
    }
}

impl Drop for GlobalRuntime {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
        GlobalHotKeyEvent::set_event_handler::<fn(GlobalHotKeyEvent)>(None);
    }
}
