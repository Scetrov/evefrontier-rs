//! Download command handler for fetching EVE Frontier datasets.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use evefrontier_lib::{ensure_dataset, DatasetRelease};

use crate::output::OutputFormat;

/// Output structure for the download command.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadOutput {
    pub dataset_path: String,
    pub release: ReleaseRequest,
    /// Optional path to the cached ship_data CSV if available.
    pub ship_data_path: Option<String>,
}

impl DownloadOutput {
    pub fn new(dataset_path: &Path, release: &DatasetRelease, ship_data: Option<&Path>) -> Self {
        Self {
            dataset_path: dataset_path.display().to_string(),
            release: release.into(),
            ship_data_path: ship_data.map(|p| p.display().to_string()),
        }
    }
}

/// Represents the release request type (latest or specific tag).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReleaseRequest {
    Latest,
    Tag { value: String },
}

impl From<&DatasetRelease> for ReleaseRequest {
    fn from(value: &DatasetRelease) -> Self {
        match value {
            DatasetRelease::Latest => ReleaseRequest::Latest,
            DatasetRelease::Tag(tag) => ReleaseRequest::Tag { value: tag.clone() },
        }
    }
}

impl fmt::Display for ReleaseRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseRequest::Latest => write!(f, "latest"),
            ReleaseRequest::Tag { value } => write!(f, "tag {}", value),
        }
    }
}

/// Handle the download subcommand.
///
/// Downloads the EVE Frontier dataset and reports its location.
pub fn handle_download(
    target_path: Option<&Path>,
    release: DatasetRelease,
    format: OutputFormat,
) -> Result<()> {
    // Ensure dataset operation runs in a blocking region so it can perform
    // reqwest::blocking operations (which create their own runtime) without
    // being dropped from inside the async runtime which causes panics.
    let paths = tokio::task::block_in_place(|| ensure_dataset(target_path, release.clone()))
        .context("failed to locate or download the EVE Frontier dataset")?;

    // Prefer DatasetPaths.ship_data, fall back to env var if provided
    let ship_path_buf: Option<PathBuf> = if let Some(p) = &paths.ship_data {
        Some(p.clone())
    } else {
        std::env::var_os("EVEFRONTIER_SHIP_DATA").map(PathBuf::from)
    };

    let output = DownloadOutput::new(&paths.database, &release, ship_path_buf.as_deref());
    format.render_download(
        &output.dataset_path,
        &output.release,
        output.ship_data_path.as_deref(),
    )
}
