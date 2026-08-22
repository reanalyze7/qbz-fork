use std::fs;
use std::path::{Path, PathBuf};

use crate::LibraryError;

use super::{AlbumTagSidecar, SIDECAR_FILE_NAME};

pub fn sidecar_path(album_dir: &Path) -> PathBuf {
    album_dir.join(SIDECAR_FILE_NAME)
}

pub fn read_album_sidecar(album_dir: &Path) -> Result<Option<AlbumTagSidecar>, LibraryError> {
    let path = sidecar_path(album_dir);
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(&path).map_err(LibraryError::Io)?;
    let sidecar: AlbumTagSidecar =
        serde_json::from_slice(&bytes).map_err(|e| LibraryError::Metadata(e.to_string()))?;
    Ok(Some(sidecar))
}

pub fn write_album_sidecar(
    album_dir: &Path,
    sidecar: &AlbumTagSidecar,
) -> Result<(), LibraryError> {
    fs::create_dir_all(album_dir).map_err(LibraryError::Io)?;

    let target = sidecar_path(album_dir);
    let tmp = album_dir.join(format!("{}.tmp", SIDECAR_FILE_NAME));
    let content =
        serde_json::to_vec_pretty(sidecar).map_err(|e| LibraryError::Metadata(e.to_string()))?;

    fs::write(&tmp, content).map_err(LibraryError::Io)?;
    fs::rename(&tmp, &target).map_err(LibraryError::Io)?;
    Ok(())
}

pub fn delete_album_sidecar(album_dir: &Path) -> Result<(), LibraryError> {
    let path = sidecar_path(album_dir);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path).map_err(LibraryError::Io)?;
    Ok(())
}
