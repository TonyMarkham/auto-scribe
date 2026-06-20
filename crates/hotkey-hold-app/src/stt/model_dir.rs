use crate::stt::{DEFAULT_MODEL_DIR, REQUIRED_MODEL_FILES, SttError, SttResult};

use std::{
    env,
    path::{Path, PathBuf},
};

pub(crate) fn model_dir_from_env_or_default() -> PathBuf {
    env::var_os("NEMOTRON_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL_DIR))
}

pub(crate) fn validate_model_dir(model_dir: &Path) -> SttResult<()> {
    if !model_dir.is_dir() {
        return Err(SttError::model_path(format!(
            "model directory does not exist: {}",
            model_dir.display()
        )));
    }

    for file_name in REQUIRED_MODEL_FILES {
        let file_path = model_dir.join(file_name);
        if !file_path.is_file() {
            return Err(SttError::model_path(format!(
                "required model file is missing: {}",
                file_path.display()
            )));
        }
    }

    Ok(())
}
