#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum State {
    Loading,
    ModelMissing,
    Downloading,
    Idle,
    Recording,
    Transcribing,
    Error,
}

impl State {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::ModelMissing => "model missing",
            Self::Downloading => "downloading",
            Self::Idle => "idle",
            Self::Recording => "recording",
            Self::Transcribing => "transcribing",
            Self::Error => "error",
        }
    }
}
