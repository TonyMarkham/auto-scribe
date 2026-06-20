use crate::stt::{REQUIRED_MODEL_FILES, SttError, SttResult};

use std::path::Path;

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
