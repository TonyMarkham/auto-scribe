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

        let config_text = fs::read_to_string(&config_path).map_err(|error| {
            SttError::model_path(format!("read {}: {error}", config_path.display()))
        })?;
        let parsed = config_text.parse::<toml::Table>().map_err(|error| {
            SttError::model_path(format!("parse {}: {error}", config_path.display()))
        })?;

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

        if model_base_url.is_empty() {
            return Err(SttError::model_path(
                "config [model].base_url must not be empty",
            ));
        }

        Ok(Self {
            config_path,
            model_dir,
            model_base_url,
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
"#
    )
}
