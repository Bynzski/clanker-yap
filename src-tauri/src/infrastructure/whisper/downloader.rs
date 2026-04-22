//! Helpers for downloading the default Whisper model.

use std::path::PathBuf;
use std::process::Command;

use crate::domain::error::{AppError, Result};
use crate::domain::{DEFAULT_MODEL_FILE, DEFAULT_MODEL_URL};
use crate::infrastructure::persistence::paths::app_data_dir;

pub fn default_model_destination() -> Result<PathBuf> {
    Ok(app_data_dir()?.join(DEFAULT_MODEL_FILE))
}

pub fn download_default_model() -> Result<PathBuf> {
    let destination = default_model_destination()?;
    let parent = destination
        .parent()
        .ok_or_else(|| AppError::Io(std::io::Error::other("Invalid model destination")))?;
    std::fs::create_dir_all(parent).map_err(AppError::Io)?;

    let temp_path = destination.with_extension("bin.part");
    if temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }

    download_with_available_tool(DEFAULT_MODEL_URL, &temp_path)?;

    let metadata = std::fs::metadata(&temp_path).map_err(AppError::Io)?;
    if metadata.len() == 0 {
        return Err(AppError::Io(std::io::Error::other(
            "Downloaded model file is empty",
        )));
    }

    std::fs::rename(&temp_path, &destination).map_err(AppError::Io)?;
    Ok(destination)
}

fn download_with_available_tool(url: &str, destination: &PathBuf) -> Result<()> {
    if run_curl(url, destination)? {
        return Ok(());
    }

    if run_wget(url, destination)? {
        return Ok(());
    }

    Err(AppError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Neither curl nor wget is available for model download",
    )))
}

fn run_curl(url: &str, destination: &PathBuf) -> Result<bool> {
    let status = match Command::new("curl")
        .args([
            "--location",
            "--fail",
            "--silent",
            "--show-error",
            "--output",
        ])
        .arg(destination)
        .arg(url)
        .status()
    {
        Ok(status) => status,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(AppError::Io(err)),
    };

    if status.success() {
        Ok(true)
    } else {
        Err(AppError::Io(std::io::Error::other(
            "curl failed while downloading the model",
        )))
    }
}

fn run_wget(url: &str, destination: &PathBuf) -> Result<bool> {
    let status = match Command::new("wget")
        .args(["--quiet", "--output-document"])
        .arg(destination)
        .arg(url)
        .status()
    {
        Ok(status) => status,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(AppError::Io(err)),
    };

    if status.success() {
        Ok(true)
    } else {
        Err(AppError::Io(std::io::Error::other(
            "wget failed while downloading the model",
        )))
    }
}
