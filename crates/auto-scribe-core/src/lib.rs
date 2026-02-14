//! Auto-scribe Core Library
//!
//! Production-grade speech-to-text library using CPAL, Rubato, and Whisper.
//!
//! # Example
//!
//! ```no_run
//! use auto_scribe_core::{AudioManager, CoreResult};
//!
//! use std::{path::PathBuf, thread::sleep, time::Duration};
//!
//! fn main() -> CoreResult<()> {
//!     let model_path = PathBuf::from("models/ggml-base.en.bin");
//!     let mut manager = AudioManager::new(&model_path, true)?;
//!
//!     manager.start_recording()?;
//!     sleep(Duration::from_secs(3));
//!     let transcription = manager.stop_recording()?;
//!
//!     println!("Transcribed: {}", transcription);
//!     Ok(())
//! }
//! ```

mod audio;
mod error;

pub use {audio::AudioManager, error::AudioError, error::Result as CoreResult};

#[cfg(test)]
mod tests;
