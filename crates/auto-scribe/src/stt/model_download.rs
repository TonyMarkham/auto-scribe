use crate::stt::{
    ModelConfig, REQUIRED_MODEL_FILES, SttError, SttResult, WorkerEvent, validate_model_dir,
};

use async_channel::Sender;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use ureq::{
    Agent,
    tls::{TlsConfig, TlsProvider},
};

const DOWNLOAD_BUFFER_SIZE: usize = 1024 * 1024;
const MAX_MODEL_FILE_BYTES: u64 = 5 * 1024 * 1024 * 1024;

pub(crate) fn spawn_model_download(
    config: ModelConfig,
    event_tx: Sender<WorkerEvent>,
) -> SttResult<()> {
    thread::Builder::new()
        .name("auto-scribe-model-download".to_string())
        .spawn(move || {
            if let Err(error) = download_model(&config, &event_tx) {
                let _ = event_tx.send_blocking(WorkerEvent::ModelDownloadError(error.to_string()));
            }
        })
        .map(|_| ())
        .map_err(|error| SttError::model_path(format!("spawn model download thread: {error}")))
}

fn download_model(config: &ModelConfig, event_tx: &Sender<WorkerEvent>) -> SttResult<()> {
    let agent = download_agent();
    let staging_dir = staging_dir(config.model_dir())?;

    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(|error| {
            SttError::model_path(format!("remove {}: {error}", staging_dir.display()))
        })?;
    }

    fs::create_dir_all(&staging_dir)
        .map_err(|error| SttError::model_path(format!("create staging directory: {error}")))?;

    let total_files = REQUIRED_MODEL_FILES.len();
    for (file_index, &file_name) in REQUIRED_MODEL_FILES.iter().enumerate() {
        download_file(
            &agent,
            config,
            file_name,
            &staging_dir,
            file_index,
            total_files,
            event_tx,
        )?;
    }

    validate_model_dir(&staging_dir)?;
    install_staged_model(&staging_dir, config.model_dir())?;
    event_tx
        .send_blocking(WorkerEvent::ModelDownloadFinished)
        .map_err(|_| SttError::worker_channel("model download receiver has disconnected"))
}

fn download_agent() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60 * 60)))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
        .build()
        .into()
}

fn download_file(
    agent: &Agent,
    config: &ModelConfig,
    file_name: &str,
    staging_dir: &Path,
    completed_files: usize,
    total_files: usize,
    event_tx: &Sender<WorkerEvent>,
) -> SttResult<()> {
    let url = config.model_url(file_name);
    let mut response = agent
        .get(&url)
        .call()
        .map_err(|error| SttError::model_path(format!("download {url}: {error}")))?;
    let file_total_bytes = content_length(response.headers());
    let mut reader = response
        .body_mut()
        .with_config()
        .limit(MAX_MODEL_FILE_BYTES)
        .reader();
    let file_path = staging_dir.join(file_name);
    let mut file = File::create(&file_path).map_err(|error| {
        SttError::model_path(format!("create {}: {error}", file_path.display()))
    })?;
    let mut buffer = vec![0; DOWNLOAD_BUFFER_SIZE];
    let mut file_downloaded_bytes = 0;

    send_progress(
        event_tx,
        file_name,
        completed_files,
        total_files,
        file_downloaded_bytes,
        file_total_bytes,
    )?;

    loop {
        let byte_count = reader
            .read(&mut buffer)
            .map_err(|error| SttError::model_path(format!("read {url}: {error}")))?;
        if byte_count == 0 {
            break;
        }

        file.write_all(&buffer[..byte_count]).map_err(|error| {
            SttError::model_path(format!("write {}: {error}", file_path.display()))
        })?;
        file_downloaded_bytes += byte_count as u64;
        send_progress(
            event_tx,
            file_name,
            completed_files,
            total_files,
            file_downloaded_bytes,
            file_total_bytes,
        )?;
    }

    file.sync_all()
        .map_err(|error| SttError::model_path(format!("sync {}: {error}", file_path.display())))?;

    send_progress(
        event_tx,
        file_name,
        completed_files + 1,
        total_files,
        file_downloaded_bytes,
        file_total_bytes,
    )
}

fn send_progress(
    event_tx: &Sender<WorkerEvent>,
    file_name: &str,
    completed_files: usize,
    total_files: usize,
    file_downloaded_bytes: u64,
    file_total_bytes: Option<u64>,
) -> SttResult<()> {
    event_tx
        .send_blocking(WorkerEvent::ModelDownloadProgress {
            file_name: file_name.to_string(),
            completed_files,
            total_files,
            file_downloaded_bytes,
            file_total_bytes,
        })
        .map_err(|_| SttError::worker_channel("model download receiver has disconnected"))
}

fn install_staged_model(staging_dir: &Path, model_dir: &Path) -> SttResult<()> {
    if let Some(parent) = model_dir.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            SttError::model_path(format!("create {}: {error}", parent.display()))
        })?;
    }

    if model_dir.exists() {
        fs::remove_dir_all(model_dir).map_err(|error| {
            SttError::model_path(format!("remove {}: {error}", model_dir.display()))
        })?;
    }

    fs::rename(staging_dir, model_dir).map_err(|error| {
        SttError::model_path(format!(
            "install {} to {}: {error}",
            staging_dir.display(),
            model_dir.display()
        ))
    })
}

fn staging_dir(model_dir: &Path) -> SttResult<PathBuf> {
    let Some(parent) = model_dir.parent() else {
        return Err(SttError::model_path(format!(
            "model directory has no parent: {}",
            model_dir.display()
        )));
    };
    let name = model_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model");

    Ok(parent.join(format!(".{name}.download")))
}

fn content_length(headers: &ureq::http::HeaderMap) -> Option<u64> {
    headers
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
}
