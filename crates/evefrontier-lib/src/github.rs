use std::path::Path;

use tracing::warn;

use crate::error::{Error, Result};

/// Download the latest dataset release from GitHub into `target_path`.
///
/// The downloader is not implemented yet; callers will receive an explicit
/// error so higher layers can provide guidance to users.
pub fn download_latest_dataset(target_path: &Path) -> Result<()> {
    warn!(
        "download_latest_dataset is not implemented; target {}",
        target_path.display()
    );
    Err(Error::DownloadNotImplemented)
}
