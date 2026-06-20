use crate::hotkey::Event;

#[derive(Clone, Debug)]
pub(crate) enum RuntimeEvent {
    Hotkey(Event),
    Status(String),
    Error(String),
}
