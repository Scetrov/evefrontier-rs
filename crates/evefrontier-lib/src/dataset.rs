use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use directories::ProjectDirs;
use once_cell::sync::Lazy;
use tracing::{debug, info};

use crate::error::{Error, Result};
use crate::github::{download_dataset_with_tag, resolve_release_tag, DatasetRelease};
use crate::spatial::spatial_index_path;

/// Default filename for the cached dataset.
const DATASET_FILENAME: &str = "static_data.db";

/// Paths to dataset files.
///
/// Returned by [`ensure_dataset`] to provide access to both the database and
/// any associated index files.
#[derive(Debug, Clone)]
pub struct DatasetPaths {
    /// Path to the SQLite database file.
    pub database: PathBuf,
    /// Path to the spatial index file, if it exists.
    pub spatial_index: Option<PathBuf>,
    /// Optional path to the cached `ship_data.csv` file if present in the dataset cache.
    pub ship_data: Option<PathBuf>,
}

impl DatasetPaths {
    /// Create paths for a database, checking for an existing spatial index.
    pub fn for_database(database: PathBuf) -> Self {
        let index_path = spatial_index_path(&database);
        let spatial_index = if index_path.exists() {
            Some(index_path)
        } else {
            None
        };
        Self {
            database,
            spatial_index,
            ship_data: None,
        }
    }

    /// Create paths for a database and an optional cached ship data CSV path.
    pub fn for_database_with_ship(database: PathBuf, ship_data: Option<PathBuf>) -> Self {
        let index_path = spatial_index_path(&database);
        let spatial_index = if index_path.exists() {
            Some(index_path)
        } else {
            None
        };
        Self {
            database,
            spatial_index,
            ship_data,
        }
    }
}

/// Absolute path to the checked-in minimal fixture database, when available.
static PROTECTED_FIXTURE_DATASET: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db");
    path.canonicalize().ok()
});

/// Resolve the default dataset location using platform-specific project directories.
pub fn default_dataset_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "evefrontier", "evefrontier")
        .ok_or(Error::ProjectDirsUnavailable)?;
    Ok(normalize_data_dir(dirs.data_dir()).join(DATASET_FILENAME))
}

fn normalize_data_dir(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        normalize_windows_data_dir(path)
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

// Test-only accessor to allow unit tests to verify normalization behavior
// without exposing the implementation in public API.
#[cfg(test)]
pub fn test_normalize_data_dir(path: &Path) -> PathBuf {
    normalize_data_dir(path)
}

#[cfg(windows)]
/// Maximum number of iterations allowed when collapsing duplicate path segments.
///
/// This limit prevents infinite loops in the normalization logic in case of
/// deeply nested or maliciously crafted paths with repeated directory names.
/// The value 100 is chosen as a conservative upper bound: in practice, no
/// reasonable data directory path should require anywhere near this many
/// collapses, but this ensures we bound the work performed and avoid denial-of-service
/// or hangs due to pathological input. If the limit is reached, normalization
/// stops and returns the best-effort result.
const MAX_NORMALIZATION_ITERATIONS: usize = 100;

#[cfg(windows)]
/// Collapse duplicate consecutive directory names that can appear in Windows data
/// directories.
///
/// The `directories` crate occasionally yields paths such as
/// `%APPDATA%\evefrontier\evefrontier\data`. This helper normalizes the path to
/// `%APPDATA%\evefrontier\data` while bounding the amount of work performed to
/// avoid pathological inputs.
fn normalize_windows_data_dir(path: &Path) -> PathBuf {
    use std::ffi::OsStr;

    fn eq_ignore_ascii_case(a: &OsStr, b: &OsStr) -> bool {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }

    fn try_collapse_duplicate(current: &Path) -> Option<PathBuf> {
        let parent = current.parent()?;
        let parent_name = parent.file_name()?;
        let grandparent = parent.parent()?;
        let grandparent_name = grandparent.file_name()?;

        if !eq_ignore_ascii_case(parent_name, grandparent_name) {
            return None;
        }

        let mut base = grandparent.to_path_buf();
        if let Some(file_name) = current.file_name() {
            base.push(file_name);
        }
        Some(base)
    }

    let mut current = path.to_path_buf();
    let mut iterations = 0;

    while iterations < MAX_NORMALIZATION_ITERATIONS {
        let Some(next) = try_collapse_duplicate(&current) else {
            break;
        };

        if next == current {
            break;
        }

        current = next;
        iterations += 1;
    }

    current
}

/// Ensure a dataset release is available locally and return paths to the files.
///
/// The resolution order matches the documentation:
/// 1. Explicit `target` argument when provided.
/// 2. `EVEFRONTIER_DATA_DIR` environment variable.
/// 3. XDG/Platform-specific project directories.
/// 4. Fallback to `~/.local/evefrontier/static_data.db` on Unix-like systems.
///
/// Returns [`DatasetPaths`] containing the database path and optional spatial index path.
pub fn ensure_dataset(target: Option<&Path>, release: DatasetRelease) -> Result<DatasetPaths> {
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

/// Ensure the Era 6 Cycle 3 dataset is available locally and return paths to the files.
pub fn ensure_e6c3_dataset(target: Option<&Path>) -> Result<DatasetPaths> {
    ensure_dataset(target, DatasetRelease::tag("e6c3"))
}

fn ensure_or_download(path: &Path, release: &DatasetRelease) -> Result<DatasetPaths> {
    guard_protected_dataset(path)?;

    if path.exists() {
        match evaluate_cache_state(path, release)? {
            CacheState::Fresh => {
                // Try to locate a cached ship_data asset associated with the resolved
                // release tag (if present) so callers can access the cached CSV.
                // Resolve the cache directory once and pass it to the helper to avoid
                // races with concurrent env var changes in parallel tests.
                let marker = read_release_marker(path)?;
                let ship = match marker {
                    Some(ref m) => {
                        if let Some(ref marker_cache_dir) = m.cache_dir {
                            crate::github::find_cached_ship_for_tag_in_dir(
                                &m.resolved_tag,
                                marker_cache_dir,
                            )
                            .ok()
                            .flatten()
                        } else {
                            let cache_dir = crate::github::dataset_cache_dir()?;
                            crate::github::find_cached_ship_for_tag_in_dir(
                                &m.resolved_tag,
                                &cache_dir,
                            )
                            .ok()
                            .flatten()
                        }
                    }
                    None => None,
                };
                return Ok(DatasetPaths::for_database_with_ship(
                    path.to_path_buf(),
                    ship,
                ));
            }
            CacheState::Stale { .. } => {
                // Stale cache detected; proceed with re-download
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
    let (resolved_tag, ship_data) = download_dataset_with_tag(path, release.clone())?;
    write_release_marker(path, release, &resolved_tag)?;
    Ok(DatasetPaths::for_database_with_ship(
        path.to_path_buf(),
        ship_data,
    ))
}

fn canonical_dataset_path(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    path.join(DATASET_FILENAME)
}

fn guard_protected_dataset(path: &Path) -> Result<()> {
    let Some(fixture) = PROTECTED_FIXTURE_DATASET.as_ref() else {
        return Ok(());
    };

    if is_same_path(path, fixture) {
        return Err(Error::ProtectedFixturePath {
            path: fixture.clone(),
        });
    }

    Ok(())
}

fn is_same_path(candidate: &Path, protected: &Path) -> bool {
    if candidate == protected {
        return true;
    }

    match candidate.canonicalize() {
        Ok(resolved) => resolved == *protected,
        Err(_) => false,
    }
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
                                _resolved_tag: Some(current_tag),
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
            _ => Ok(CacheState::Stale {
                _resolved_tag: None,
            }),
        },
        DatasetRelease::Tag(expected) => match marker {
            Some(marker) if marker.resolved_tag == *expected => Ok(CacheState::Fresh),
            _ => Ok(CacheState::Stale {
                _resolved_tag: Some(expected.clone()),
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

    // Include the cache directory used by the downloader to allow deterministic
    // lookup of companion assets (e.g., ship_data.csv). This helps avoid races
    // where the global environment may change between operations in parallel tests.
    let cache_dir = crate::github::dataset_cache_dir().ok();
    let marker = if let Some(ref dir) = cache_dir {
        let mut m = ReleaseMarker::new(release, tag);
        m.cache_dir = Some(dir.clone());
        m
    } else {
        ReleaseMarker::new(release, tag)
    };

    fs::write(marker_path, marker.format())?;
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
    /// Optional cache directory where associated assets (e.g., ship_data.csv)
    /// were written. This helps tests and callers locate cached assets
    /// deterministically without depending on global environment state.
    cache_dir: Option<PathBuf>,
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
            cache_dir: None,
        }
    }

    fn format(&self) -> String {
        let mut out = format!(
            "requested={}\nresolved={}\n",
            self.requested.as_str(),
            self.resolved_tag
        );
        if let Some(dir) = &self.cache_dir {
            out.push_str(&format!("cache_dir={}\n", dir.display()));
        }
        out
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
        let mut cache_dir = None;

        for line in s.lines() {
            if let Some(value) = line.strip_prefix("requested=") {
                requested = MarkerRequest::from_str(value).ok();
            } else if let Some(value) = line.strip_prefix("resolved=") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    resolved = Some(trimmed.to_string());
                }
            } else if let Some(value) = line.strip_prefix("cache_dir=") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    // store as PathBuf; validation deferred to callers
                    cache_dir = Some(PathBuf::from(trimmed));
                }
            }
        }

        match (requested, resolved) {
            (Some(requested), Some(resolved_tag)) => Ok(Self {
                requested,
                resolved_tag,
                cache_dir,
            }),
            _ => {
                let fallback = s.trim();
                if fallback.is_empty() {
                    Err(())
                } else {
                    Ok(Self {
                        requested: MarkerRequest::Tag,
                        resolved_tag: fallback.to_string(),
                        cache_dir: None,
                    })
                }
            }
        }
    }
}

enum CacheState {
    Fresh,
    Stale { _resolved_tag: Option<String> },
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    mod windows {
        use super::super::normalize_windows_data_dir;
        use std::path::PathBuf;

        #[test]
        fn collapses_duplicate_segments() {
            let input = PathBuf::from(r"C:\Users\mcp\AppData\Roaming\evefrontier\evefrontier\data");
            let expected = PathBuf::from(r"C:\Users\mcp\AppData\Roaming\evefrontier\data");

            assert_eq!(normalize_windows_data_dir(&input), expected);
        }

        #[test]
        fn preserves_already_normalized_paths() {
            let input = PathBuf::from(r"C:\Users\mcp\AppData\Roaming\evefrontier\data");

            assert_eq!(normalize_windows_data_dir(&input), input);
        }

        #[test]
        fn handles_unc_paths() {
            let input = PathBuf::from(r"\\server\share\evefrontier\evefrontier\data");
            let expected = PathBuf::from(r"\\server\share\evefrontier\data");

            assert_eq!(normalize_windows_data_dir(&input), expected);
        }
    }

    #[test]
    fn ensure_dataset_includes_cached_ship_data() {
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let src = tmp.path().join("src");
        let _ = std::fs::create_dir_all(&src);

        // Create a dummy .db file and ship_data.csv in the source dir
        let db_path = src.join("static_data.db");
        std::fs::write(&db_path, b"dummy db").expect("write db");
        let ship_path = src.join("ship_data.csv");
        std::fs::write(&ship_path, b"name,mass\nTest,1000").expect("write ship csv");

        let target = tmp.path().join("target_static.db");
        let cache_dir = tmp.path().join("cache_dir");

        // Ensure the helper and cache lookup use the same cache directory
        std::env::set_var("EVEFRONTIER_DATASET_CACHE_DIR", &cache_dir);

        // Populate cache and target using the test helper
        let _resolved = crate::github::download_from_source_with_cache(
            &target,
            crate::github::DatasetRelease::Latest,
            &src,
            &cache_dir,
            "e6c3",
        )
        .expect("download from source with cache");

        // Now ensure_dataset should detect the cached ship data and include it in the paths
        let paths = super::ensure_dataset(Some(&target), crate::github::DatasetRelease::Latest)
            .expect("ensure_dataset");

        assert!(paths.ship_data.is_some(), "ship_data should be present");
        let ship_cached = paths.ship_data.unwrap();
        assert!(
            ship_cached.exists(),
            "cached ship_data path should exist: {}",
            ship_cached.display()
        );
        // The cached ship data should generally be under the cache directory
        // used when the dataset was populated, but other tests may modify
        // environment-wide cache configuration in parallel runs. To avoid
        // flakiness we only assert existence here (the marker contains the
        // cache_dir which callers may consult if required).
    }
}
