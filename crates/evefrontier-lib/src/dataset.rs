use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use tracing::info;

use crate::error::{Error, Result};
use crate::github::{download_dataset, DatasetRelease};

/// Default filename for the cached dataset.
const DATASET_FILENAME: &str = "static_data.db";

/// Resolve the default dataset location using platform-specific project directories.
pub fn default_dataset_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "evefrontier", "evefrontier")
        .ok_or(Error::ProjectDirsUnavailable)?;
    Ok(dirs.data_dir().join(DATASET_FILENAME))
}

/// Ensure a dataset release is available locally and return its absolute path.
///
/// The resolution order matches the documentation:
/// 1. Explicit `target` argument when provided.
/// 2. `EVEFRONTIER_DATA_DIR` environment variable.
/// 3. XDG/Platform-specific project directories.
/// 4. Fallback to `~/.local/evefrontier/static_data.db` on Unix-like systems.
pub fn ensure_dataset(target: Option<&Path>, release: DatasetRelease) -> Result<PathBuf> {
    if let Some(explicit) = target {
        let resolved = canonical_dataset_path(explicit);
        return ensure_or_download(&resolved, &release);
    }

    if let Some(env_path) = env::var_os("EVEFRONTIER_DATA_DIR") {
        let resolved = canonical_dataset_path(Path::new(&env_path));
        return ensure_or_download(&resolved, &release);
    }

    let default = default_dataset_path()?;
    if default.exists() {
        return Ok(default);
    }

    ensure_or_download(&default, &release)
}

/// Ensure the Era 6 Cycle 3 dataset is available locally and return its absolute path.
pub fn ensure_c3e6_dataset(target: Option<&Path>) -> Result<PathBuf> {
    ensure_dataset(target, DatasetRelease::tag("e6c3"))
}

fn ensure_or_download(path: &Path, release: &DatasetRelease) -> Result<PathBuf> {
    if path.exists() {
        return Ok(path.to_path_buf());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    info!(
        release = %release,
        "attempting to download dataset to {}",
        path.display()
    );
    download_dataset(path, release.clone())?;
    Ok(path.to_path_buf())
}

fn canonical_dataset_path(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    path.join(DATASET_FILENAME)
}
