#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppState {
    Loading,
    Idle,
    Recording,
    Transcribing,
    Error,
}

impl AppState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Idle => "idle",
            Self::Recording => "recording",
            Self::Transcribing => "transcribing",
            Self::Error => "error",
        }
    }
}
