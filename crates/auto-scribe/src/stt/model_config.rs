use crate::stt::{DEFAULT_MODEL_BASE_URL, DEFAULT_MODEL_DIRECTORY, SttError, SttResult};

use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(crate) struct ModelConfig {
    config_path: PathBuf,
    model_dir: PathBuf,
    model_base_url: String,
    auto_mute_speakers: bool,
}

impl ModelConfig {
    pub(crate) fn load() -> SttResult<Self> {
        let app_data_dir = app_data_dir()?;
        let config_path = app_data_dir.join("config.toml");

        fs::create_dir_all(&app_data_dir)
            .map_err(|error| SttError::model_path(format!("create app data directory: {error}")))?;

        if !config_path.is_file() {
            fs::write(&config_path, default_config_text()).map_err(|error| {
                SttError::model_path(format!("write {}: {error}", config_path.display()))
            })?;
        }

        let parsed = read_config_table(&config_path)?;

        let model_table = parsed.get("model").and_then(toml::Value::as_table);
        let configured_dir = model_table
            .and_then(|table| table.get("directory"))
            .and_then(toml::Value::as_str)
            .unwrap_or(DEFAULT_MODEL_DIRECTORY);
        let configured_base_url = model_table
            .and_then(|table| table.get("base_url"))
            .and_then(toml::Value::as_str)
            .unwrap_or(DEFAULT_MODEL_BASE_URL);

        let configured_model_dir = resolve_model_dir(&app_data_dir, configured_dir)?;
        let model_dir = env::var_os("NEMOTRON_MODEL_DIR")
            .map(PathBuf::from)
            .unwrap_or(configured_model_dir);
        let model_base_url = configured_base_url.trim_end_matches('/').to_string();
        let audio_table = parsed.get("audio").and_then(toml::Value::as_table);
        let auto_mute_speakers = audio_table
            .and_then(|table| table.get("auto_mute_speakers"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);

        if model_base_url.is_empty() {
            return Err(SttError::model_path(
                "config [model].base_url must not be empty",
            ));
        }

        Ok(Self {
            config_path,
            model_dir,
            model_base_url,
            auto_mute_speakers,
        })
    }

    pub(crate) fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub(crate) fn model_dir(&self) -> &Path {
        &self.model_dir
    }

    pub(crate) fn model_url(&self, file_name: &str) -> String {
        format!("{}/{}", self.model_base_url, file_name)
    }

    pub(crate) fn auto_mute_speakers(&self) -> bool {
        self.auto_mute_speakers
    }

    pub(crate) fn set_auto_mute_speakers(&mut self, enabled: bool) -> SttResult<()> {
        let mut parsed = read_config_table(&self.config_path)?;
        upsert_audio_auto_mute(&mut parsed, enabled);
        let config_text = toml::to_string_pretty(&parsed).map_err(|error| {
            SttError::model_path(format!("serialize {}: {error}", self.config_path.display()))
        })?;

        fs::write(&self.config_path, config_text).map_err(|error| {
            SttError::model_path(format!("write {}: {error}", self.config_path.display()))
        })?;
        self.auto_mute_speakers = enabled;
        Ok(())
    }
}

fn read_config_table(config_path: &Path) -> SttResult<toml::Table> {
    let config_text = fs::read_to_string(config_path).map_err(|error| {
        SttError::model_path(format!("read {}: {error}", config_path.display()))
    })?;

    config_text
        .parse::<toml::Table>()
        .map_err(|error| SttError::model_path(format!("parse {}: {error}", config_path.display())))
}

fn upsert_audio_auto_mute(parsed: &mut toml::Table, enabled: bool) {
    let audio = parsed
        .entry("audio".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    if !audio.is_table() {
        *audio = toml::Value::Table(toml::Table::new());
    }

    if let Some(audio_table) = audio.as_table_mut() {
        audio_table.insert(
            "auto_mute_speakers".to_string(),
            toml::Value::Boolean(enabled),
        );
    }
}

fn app_data_dir() -> SttResult<PathBuf> {
    if let Some(data_home) = env::var_os("XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        return Ok(PathBuf::from(data_home).join("auto-scribe"));
    }

    let Some(home) = env::var_os("HOME") else {
        return Err(SttError::model_path(
            "HOME is not set; cannot resolve Auto Scribe app data directory",
        ));
    };

    Ok(PathBuf::from(home).join(".local/share/auto-scribe"))
}

fn resolve_model_dir(app_data_dir: &Path, configured_dir: &str) -> SttResult<PathBuf> {
    if configured_dir.trim().is_empty() {
        return Err(SttError::model_path(
            "config [model].directory must not be empty",
        ));
    }

    if let Some(relative_home_path) = configured_dir.strip_prefix("~/") {
        let Some(home) = env::var_os("HOME") else {
            return Err(SttError::model_path(
                "HOME is not set; cannot expand model directory",
            ));
        };
        return Ok(PathBuf::from(home).join(relative_home_path));
    }

    let path = PathBuf::from(configured_dir);
    if path.is_absolute() {
        return Ok(path);
    }

    Ok(app_data_dir.join(path))
}

fn default_config_text() -> String {
    format!(
        r#"# Auto Scribe local configuration.
# Relative model directories are resolved under this config file's app data directory.

[model]
directory = "{DEFAULT_MODEL_DIRECTORY}"
base_url = "{DEFAULT_MODEL_BASE_URL}"

[audio]
auto_mute_speakers = false
"#
    )
}
