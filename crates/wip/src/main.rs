mod app;
pub mod app_state;
mod audio_recorder;
pub mod audio_recording;
mod error;
pub mod recorder_state;
pub mod work_event;
pub mod worker_request;

// ---------------------------------------------------------------------------------------------- //

use crate::{
    app::{App, model_dir_from_env_or_default, validate_model_dir},
    error::{WipError, WipResult},
};

fn main() -> WipResult<()> {
    let model_dir = model_dir_from_env_or_default();
    validate_model_dir(&model_dir)?;

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 420.0])
            .with_min_inner_size([360.0, 320.0]),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Push-to-talk STT",
        native_options,
        Box::new(move |_creation_context| Ok(Box::new(App::new(model_dir)?))),
    )
    .map_err(|error| WipError::ui(error.to_string()))
}
