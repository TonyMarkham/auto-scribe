mod audio_config;
mod behaviour_config;
#[allow(clippy::module_inception)]
mod config;
mod server_config;
mod whisper_config;

pub(crate) use {
    audio_config::AudioConfig, behaviour_config::BehaviourConfig, config::Config,
    server_config::ServerConfig, whisper_config::WhisperConfig,
};

pub(crate) const DEFAULT_AUTO_PASTE: bool = true;
pub(crate) const DEFAULT_PORT: u16 = 7878;

pub(crate) fn default_auto_paste() -> bool {
    DEFAULT_AUTO_PASTE
}

pub(crate) fn default_port() -> u16 {
    DEFAULT_PORT
}
