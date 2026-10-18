use crate::{models, runtime::Runtime};
use dictation_core::defaults;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::Path,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

pub fn install(runtime: &Runtime, id: &str) -> Result<(), String> {
    let model = models::catalog()?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or("Unknown model.")?;
    if runtime.downloading.swap(true, Ordering::AcqRel) {
        return Err("A model download is already running.".into());
    }
    runtime.cancel_download.store(false, Ordering::Release);
    let runtime = runtime.clone();
    std::thread::spawn(move || {
        let result = download(&runtime, &model);
        runtime.downloading.store(false, Ordering::Release);
        runtime.update(|status| {
            status.download = None;
            status.message = result.map_or_else(
                |error| error,
                |_| {
                    "Model installed and loaded successfully. Select it in settings and save."
                        .into()
                },
            );
        });
    });
    Ok(())
}

fn download(runtime: &Runtime, model: &models::Model) -> Result<(), String> {
    let root = &runtime.model_root;
    std::fs::create_dir_all(root).map_err(|_| "Cannot create model directory.")?;
    let available =
        fs2::available_space(root).map_err(|_| "Cannot check available model storage.")?;
    if available < model.bytes * 3 {
        return Err("Insufficient disk space for model installation.".into());
    }
    let temporary = tempfile::tempdir_in(root).map_err(|_| "Cannot create download directory.")?;
    let path = temporary.path().join(&model.file_name);
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(defaults::DOWNLOAD_TIMEOUT_SECONDS))
        .build()
        .map_err(|_| "Cannot initialize the model downloader.")?;
    let mut response = client
        .get(&model.url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|_| "Model download failed. Check the connection and retry.")?;
    let mut output =
        std::fs::File::create(&path).map_err(|_| "Cannot create model download file.")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut received = 0_u64;
    let mut last_update = Instant::now() - Duration::from_secs(1);
    loop {
        if runtime.cancel_download.load(Ordering::Acquire) {
            return Err("Model download cancelled.".into());
        }
        let count = response
            .read(&mut buffer)
            .map_err(|_| "Download interrupted. Retry to download a fresh verified copy.")?;
        if count == 0 {
            break;
        }
        received += count as u64;
        if received > model.bytes || received > defaults::MAX_MODEL_BYTES {
            return Err("Download exceeded its declared size.".into());
        }
        hasher.update(&buffer[..count]);
        output
            .write_all(&buffer[..count])
            .map_err(|_| "Model write failed. Check disk space.")?;
        if last_update.elapsed() >= Duration::from_millis(200) {
            runtime.update(|status| {
                status.download = Some(crate::runtime::Download {
                    id: model.id.clone(),
                    received,
                    total: model.bytes,
                })
            });
            last_update = Instant::now();
        }
    }
    output
        .sync_all()
        .map_err(|_| "Model could not be flushed to disk.")?;
    drop(output);
    if received != model.bytes || format!("{:x}", hasher.finalize()) != model.sha256 {
        return Err("Model integrity check failed; the downloaded file was discarded.".into());
    }
    if model.archive_directory.is_some() {
        extract(&path, temporary.path())?;
        std::fs::remove_file(&path).map_err(|_| "Cannot remove verified archive.")?;
    }
    let loaded_path = model
        .archive_directory
        .as_ref()
        .map_or_else(|| path.clone(), |folder| temporary.path().join(folder));
    models::validate(&model.kind, &loaded_path.to_string_lossy())?;
    std::fs::write(
        temporary.path().join("verified.json"),
        serde_json::to_vec(model).map_err(|_| "Invalid catalog metadata.")?,
    )
    .map_err(|_| "Cannot write model verification metadata.")?;
    let destination = root.join(&model.id);
    if destination.exists() {
        return Err("This model is already installed. Its current files were preserved.".into());
    }
    std::fs::rename(temporary.path(), &destination)
        .map_err(|_| "Atomic model installation failed.")?;
    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|_| "Cannot read verified archive.")?;
    let mut archive = tar::Archive::new(bzip2::read::BzDecoder::new(file));
    let mut expanded = 0_u64;
    for entry in archive.entries().map_err(|_| "Invalid model archive.")? {
        let mut entry = entry.map_err(|_| "Invalid model archive entry.")?;
        if !entry.header().entry_type().is_file() && !entry.header().entry_type().is_dir() {
            return Err("Model archive contains a link or special file.".into());
        }
        expanded += entry.size();
        if expanded > defaults::MAX_MODEL_BYTES {
            return Err("Model archive exceeds the expanded size limit.".into());
        }
        if !entry
            .unpack_in(destination)
            .map_err(|_| "Model extraction failed.")?
        {
            return Err("Unsafe path in model archive.".into());
        }
    }
    Ok(())
}
