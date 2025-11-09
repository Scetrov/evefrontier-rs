use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use directories::ProjectDirs;
use tracing::{debug, info};

use crate::error::{Error, Result};
use crate::github::{download_dataset_with_tag, resolve_release_tag, DatasetRelease};

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
    ensure_or_download(&default, &release)
}

/// Ensure the Era 6 Cycle 3 dataset is available locally and return its absolute path.
pub fn ensure_c3e6_dataset(target: Option<&Path>) -> Result<PathBuf> {
    ensure_dataset(target, DatasetRelease::tag("e6c3"))
}

fn ensure_or_download(path: &Path, release: &DatasetRelease) -> Result<PathBuf> {
    let mut resolved_tag_hint = None;

    if path.exists() {
        match evaluate_cache_state(path, release)? {
            CacheState::Fresh => return Ok(path.to_path_buf()),
            CacheState::Stale { resolved_tag } => {
                resolved_tag_hint = resolved_tag;
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    info!(
        release = %release,
        "attempting to download dataset to {}",
        path.display()
    );
    let resolved_tag = download_dataset_with_tag(path, release.clone())?;
    let marker_tag = resolved_tag_hint.unwrap_or_else(|| resolved_tag.clone());
    write_release_marker(path, release, &marker_tag)?;
    Ok(path.to_path_buf())
}

fn canonical_dataset_path(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    path.join(DATASET_FILENAME)
}

fn evaluate_cache_state(path: &Path, release: &DatasetRelease) -> Result<CacheState> {
    let marker = read_release_marker(path)?;
    match release {
        DatasetRelease::Latest => match marker {
            Some(marker) if matches!(marker.requested, MarkerRequest::Latest) => {
                match resolve_release_tag(release) {
                    Ok(current_tag) => {
                        if marker.resolved_tag == current_tag {
                            Ok(CacheState::Fresh)
                        } else {
                            Ok(CacheState::Stale {
                                resolved_tag: Some(current_tag),
                            })
                        }
                    }
                    Err(error) => {
                        debug!(
                            %error,
                            path = %path.display(),
                            "failed to resolve latest release tag; assuming cached dataset is current"
                        );
                        Ok(CacheState::Fresh)
                    }
                }
            }
            _ => Ok(CacheState::Stale { resolved_tag: None }),
        },
        DatasetRelease::Tag(expected) => match marker {
            Some(marker) if marker.resolved_tag == *expected => Ok(CacheState::Fresh),
            _ => Ok(CacheState::Stale {
                resolved_tag: Some(expected.clone()),
            }),
        },
    }
}

fn release_marker_path(path: &Path) -> PathBuf {
    if let Some(file_name) = path.file_name() {
        let mut name = file_name.to_os_string();
        name.push(".release");
        return path.with_file_name(name);
    }

    path.with_extension("release")
}

fn read_release_marker(path: &Path) -> Result<Option<ReleaseMarker>> {
    let marker_path = release_marker_path(path);
    if !marker_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(marker_path)?;
    Ok(ReleaseMarker::from_str(&contents).ok())
}

fn write_release_marker(path: &Path, release: &DatasetRelease, tag: &str) -> Result<()> {
    let marker_path = release_marker_path(path);
    if let Some(parent) = marker_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let marker = ReleaseMarker::new(release, tag);
    fs::write(marker_path, marker.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkerRequest {
    Latest,
    Tag,
}

impl MarkerRequest {
    fn as_str(&self) -> &'static str {
        match self {
            MarkerRequest::Latest => "latest",
            MarkerRequest::Tag => "tag",
        }
    }
}

impl FromStr for MarkerRequest {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim() {
            "latest" => Ok(MarkerRequest::Latest),
            "tag" => Ok(MarkerRequest::Tag),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseMarker {
    requested: MarkerRequest,
    resolved_tag: String,
}

impl ReleaseMarker {
    fn new(release: &DatasetRelease, tag: &str) -> Self {
        let requested = match release {
            DatasetRelease::Latest => MarkerRequest::Latest,
            DatasetRelease::Tag(_) => MarkerRequest::Tag,
        };
        Self {
            requested,
            resolved_tag: tag.trim().to_string(),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "requested={}\nresolved={}\n",
            self.requested.as_str(),
            self.resolved_tag
        )
    }
}

impl FromStr for ReleaseMarker {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err(());
        }

        let mut requested = None;
        let mut resolved = None;

        for line in s.lines() {
            if let Some(value) = line.strip_prefix("requested=") {
                requested = MarkerRequest::from_str(value).ok();
            } else if let Some(value) = line.strip_prefix("resolved=") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    resolved = Some(trimmed.to_string());
                }
            }
        }

        match (requested, resolved) {
            (Some(requested), Some(resolved_tag)) => Ok(Self {
                requested,
                resolved_tag,
            }),
            _ => {
                let fallback = s.trim();
                if fallback.is_empty() {
                    Err(())
                } else {
                    Ok(Self {
                        requested: MarkerRequest::Tag,
                        resolved_tag: fallback.to_string(),
                    })
                }
            }
        }
    }
}

enum CacheState {
    Fresh,
    Stale { resolved_tag: Option<String> },
}
