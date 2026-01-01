use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::BaseDirs;
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::error::{Error, Result};

const RELEASES_API_BASE: &str =
    "https://api.github.com/repos/Scetrov/evefrontier_datasets/releases";
const CACHE_DIR_NAME: &str = "evefrontier_datasets";
const CACHE_DIR_ENV: &str = "EVEFRONTIER_DATASET_CACHE_DIR";
const DATASET_SOURCE_ENV: &str = "EVEFRONTIER_DATASET_SOURCE";
const LATEST_TAG_OVERRIDE_ENV: &str = "EVEFRONTIER_DATASET_LATEST_TAG";

// Retry/backoff policy for transient failures (timeouts, connection resets, 5xx).
// Blocking implementation (std::thread::sleep) because we use the blocking reqwest client.
// If migrating to async, swap for `tokio::time::sleep` and async reqwest.
//
// Semantics:
// - HTTP_MAX_RETRIES counts *additional* retries after the initial attempt.
// - Total attempts = 1 + HTTP_MAX_RETRIES.
// - Backoff sequence (per failure before next attempt): 300ms → 600ms → 1200ms (capped growth).
//   For 3 retries this is 300 + 600 + 1200 = 2100ms maximum added delay.
// - Combined with 30s client timeout worst-case wall time ≈ 32.1s for a single request context.
const HTTP_MAX_RETRIES: usize = 3; // initial + 3 retries = 4 attempts total
const HTTP_INITIAL_BACKOFF_MS: u64 = 300;

// Generic retry loop helper. Attempts the operation up to (1 + HTTP_MAX_RETRIES) times.
// `should_retry` returns true if the error is transient and the loop should continue.
fn retry_loop<T, E, Op, ShouldRetry>(
    mut op: Op,
    mut should_retry: ShouldRetry,
) -> std::result::Result<T, E>
where
    Op: FnMut() -> std::result::Result<T, E>,
    ShouldRetry: FnMut(&E) -> bool,
{
    let mut backoff = HTTP_INITIAL_BACKOFF_MS;
    for attempt in 0..=HTTP_MAX_RETRIES {
        match op() {
            Ok(val) => return Ok(val),
            Err(err) => {
                if attempt == HTTP_MAX_RETRIES || !should_retry(&err) {
                    return Err(err);
                }
                std::thread::sleep(Duration::from_millis(backoff));
                backoff = (backoff.saturating_mul(2)).min(5_000);
            }
        }
    }
    unreachable!("loop exits via return on success or final failure")
}

/// Identifier for a GitHub dataset release to download.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DatasetRelease {
    /// Download the latest published release.
    #[default]
    Latest,
    /// Download a release identified by its Git tag (for example `e6c3`).
    Tag(String),
}

impl DatasetRelease {
    pub fn latest() -> Self {
        DatasetRelease::Latest
    }

    pub fn tag<T: Into<String>>(tag: T) -> Self {
        DatasetRelease::Tag(tag.into())
    }
}

impl fmt::Display for DatasetRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetRelease::Latest => write!(f, "latest"),
            DatasetRelease::Tag(tag) => write!(f, "tag {}", tag),
        }
    }
}

/// Download the latest dataset release from GitHub into `target_path`.
///
/// The downloader caches the extracted database under the OS cache directory
/// to avoid repeated downloads. Tests may override the download source via the
/// `EVEFRONTIER_DATASET_SOURCE` environment variable, which accepts either a
/// path to a `.db` file or a `.zip` archive containing the database.
pub fn download_latest_dataset(target_path: &Path) -> Result<()> {
    download_dataset_with_tag(target_path, DatasetRelease::Latest).map(|_| ())
}

/// Download the specified dataset release from GitHub into `target_path`.
///
/// When no local override is configured the downloader resolves the release
/// metadata through the GitHub API, caches the resulting asset, and copies the
/// database into the requested `target_path`.
pub fn download_dataset(target_path: &Path, release: DatasetRelease) -> Result<()> {
    download_dataset_with_tag(target_path, release).map(|_| ())
}

/// Test helper: download the latest dataset but use `source` as a local override
/// (either a `.db` file or a `.zip` archive). This avoids using the global
/// `EVEFRONTIER_DATASET_SOURCE` env var so tests can run in parallel.
pub fn download_latest_from_source(target_path: &Path, source: &Path) -> Result<()> {
    download_from_source(target_path, DatasetRelease::Latest, source).map(|_| ())
}

/// Test helper: download the specified release but use `source` as a local
/// override (either a `.db` file or a `.zip` archive). Returns the resolved
/// tag so callers can inspect it if needed.
pub fn download_from_source(
    target_path: &Path,
    release: DatasetRelease,
    source: &Path,
) -> Result<String> {
    copy_from_override(source, target_path)?;
    resolve_release_tag(&release)
}

/// Test helper: same as `download_latest_from_source` but use the provided
/// `cache_dir` instead of reading/modifying the global environment. This
/// avoids races when tests run in parallel.
pub fn download_latest_from_source_with_cache(
    target_path: &Path,
    source: &Path,
    cache_dir: &Path,
) -> Result<()> {
    download_from_source_with_cache(
        target_path,
        DatasetRelease::Latest,
        source,
        cache_dir,
        "latest",
    )
    .map(|_| ())
}

/// Test helper: same as `download_from_source` but use the provided `cache_dir`
/// instead of the global cache directory. This allows tests to avoid races when
/// multiple tests run in parallel.
///
/// The `resolved_tag` parameter explicitly specifies what tag should be written
/// to the marker file. For `DatasetRelease::Latest`, pass \"latest\" unless you
/// want to simulate a specific resolved tag. For `DatasetRelease::Tag`, this
/// should match the tag value.
pub fn download_from_source_with_cache(
    target_path: &Path,
    release: DatasetRelease,
    source: &Path,
    cache_dir: &Path,
    resolved_tag: &str,
) -> Result<String> {
    copy_from_override_with_cache(source, target_path, cache_dir)?;

    // Use the explicitly provided resolved tag instead of trying to infer it
    // from environment variables or GitHub API, which can cause race conditions
    // in tests.
    let resolved = resolved_tag.to_string();

    // Write a simple release marker adjacent to the target to match
    // `write_release_marker`'s format so tests can assert on it.
    if let Some(file_name) = target_path.file_name() {
        let mut marker_name = file_name.to_os_string();
        marker_name.push(".release");
        let marker_path = target_path.with_file_name(marker_name);
        if let Some(parent) = marker_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let requested = match &release {
            DatasetRelease::Latest => "latest",
            DatasetRelease::Tag(_) => "tag",
        };
        let contents = format!("requested={}\nresolved={}\n", requested, resolved);
        fs::write(marker_path, contents)?;
    }

    Ok(resolved)
}

pub(crate) fn download_dataset_with_tag(
    target_path: &Path,
    release: DatasetRelease,
) -> Result<String> {
    if let Some(source) = env::var_os(DATASET_SOURCE_ENV) {
        let override_path = PathBuf::from(source);
        info!(
            target = %target_path.display(),
            override = %override_path.display(),
            "using local dataset override"
        );
        copy_from_override(&override_path, target_path)?;

        // When using a local override we still want to record the resolved
        // release tag. This honors the `EVEFRONTIER_DATASET_LATEST_TAG`
        // override used by tests (or other local tooling) so the release
        // marker reflects the actual resolved tag instead of the literal
        // string "latest".
        return resolve_release_tag(&release);
    }

    let cache_dir = dataset_cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let client = build_client()?;
    let release_response = fetch_release(&client, &release)?;
    let asset =
        select_dataset_asset(&release_response).ok_or_else(|| Error::DatasetAssetMissing {
            tag: release_response.tag_name.clone(),
        })?;

    let cached_dataset = cache_dir.join(cache_file_name(&asset, &release_response, target_path));
    if !cached_dataset.exists() {
        info!(
            tag = %release_response.tag_name,
            asset = %asset.name,
            path = %cached_dataset.display(),
            release = %release,
            "caching dataset asset"
        );
        match asset.kind {
            AssetKind::Database => {
                download_database_asset(&client, &asset.download_url, &cached_dataset)?
            }
            AssetKind::Archive => {
                download_archive_asset(&client, &asset.download_url, &cached_dataset)?
            }
            AssetKind::ShipCsv => {
                // Should not be selected for the main dataset; ship CSVs will be handled
                // separately by the ship-data downloader tasks.
                unreachable!("select_dataset_asset should not return ShipCsv");
            }
        }
    } else {
        debug!(path = %cached_dataset.display(), "using cached dataset asset");
    }

    // Attempt to cache ship_data.csv alongside the dataset when present in the release.
    if let Some(ship_asset) = select_ship_asset(&release_response) {
        let cached_ship = cache_dir.join(cache_file_name(
            &ship_asset,
            &release_response,
            Path::new("ship_data.csv"),
        ));

        let need_download = if cached_ship.exists() {
            match read_checksum_sidecar(&cached_ship)? {
                Some(expected) => match compute_sha256_hex(&cached_ship) {
                    Ok(actual) if actual == expected => {
                        debug!(path = %cached_ship.display(), "using cached ship asset (checksum verified)");
                        false
                    }
                    Ok(_) => {
                        info!(path = %cached_ship.display(), "ship asset checksum mismatch; re-downloading");
                        true
                    }
                    Err(err) => {
                        warn!(%err, "failed to compute ship asset checksum; re-downloading");
                        true
                    }
                },
                None => {
                    // No checksum sidecar: compute and write one
                    match compute_sha256_hex(&cached_ship) {
                        Ok(actual) => {
                            write_checksum_sidecar(&cached_ship, &actual)?;
                            debug!(path = %cached_ship.display(), "wrote missing ship checksum sidecar");
                            false
                        }
                        Err(_) => true,
                    }
                }
            }
        } else {
            true
        };

        if need_download {
            info!(tag = %release_response.tag_name, asset = %ship_asset.name, path = %cached_ship.display(), "caching ship_data asset");
            download_ship_asset(&client, &ship_asset.download_url, &cached_ship)?;
            let checksum = compute_sha256_hex(&cached_ship)?;
            write_checksum_sidecar(&cached_ship, &checksum)?;
        }
    }

    copy_cached_to_target(&cached_dataset, target_path)?;
    Ok(release_response.tag_name)
}

pub(crate) fn resolve_release_tag(release: &DatasetRelease) -> Result<String> {
    match release {
        DatasetRelease::Latest => {
            if let Some(override_tag) = env::var_os(LATEST_TAG_OVERRIDE_ENV) {
                let tag = override_tag.to_string_lossy().trim().to_string();
                if !tag.is_empty() {
                    return Ok(tag);
                }
            }

            if env::var_os(DATASET_SOURCE_ENV).is_some() {
                return Ok("latest".to_string());
            }

            let client = build_client()?;
            let release_response = fetch_release(&client, release)?;
            Ok(release_response.tag_name)
        }
        DatasetRelease::Tag(tag) => Ok(tag.clone()),
    }
}

fn copy_from_override(source: &Path, target: &Path) -> Result<()> {
    let cache_dir = dataset_cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let cached_dataset = cache_dir.join(format!("local-{}", target_dataset_filename(target)));
    if source.is_dir() {
        // Copy DB file from directory source
        let mut found_db = false;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.eq_ignore_ascii_case("db"))
                .unwrap_or(false)
            {
                copy_file_atomic(&path, &cached_dataset)?;
                found_db = true;
                break;
            }
        }
        if !found_db {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "no .db file found in source directory",
            )));
        }

        // Also copy ship_data.csv if present
        let ship_src = source.join("ship_data.csv");
        if ship_src.exists() {
            let cached_ship = cache_dir.join(format!("local-{}", "ship_data.csv"));
            copy_file_atomic(&ship_src, &cached_ship)?;
            let checksum = compute_sha256_hex(&cached_ship)?;
            write_checksum_sidecar(&cached_ship, &checksum)?;
        }
    } else if source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        extract_archive(source, &cached_dataset)?;

        // For archives, also try to extract ship_data.csv if present
        let file = File::open(source)?;
        let mut archive = ZipArchive::new(file)?;
        for idx in 0..archive.len() {
            let mut entry = archive.by_index(idx)?;
            if !entry.is_file() {
                continue;
            }
            if let Some(path) = entry.enclosed_name() {
                let lname = path.to_string_lossy().to_ascii_lowercase();
                if lname.ends_with("ship_data.csv") {
                    let cached_ship = cache_dir.join(format!("local-{}", "ship_data.csv"));
                    let mut tmp = NamedTempFile::new_in(cache_dir.as_path())?;
                    io::copy(&mut entry, tmp.as_file_mut())?;
                    tmp.flush()?;
                    tmp.persist(&cached_ship).map_err(|err| err.error)?;
                    let checksum = compute_sha256_hex(&cached_ship)?;
                    write_checksum_sidecar(&cached_ship, &checksum)?;
                    break;
                }
            }
        }
    } else {
        copy_file_atomic(source, &cached_dataset)?;
    }

    copy_cached_to_target(&cached_dataset, target)
}

fn copy_from_override_with_cache(source: &Path, target: &Path, cache_dir: &Path) -> Result<()> {
    fs::create_dir_all(cache_dir)?;

    let cached_dataset =
        PathBuf::from(cache_dir).join(format!("local-{}", target_dataset_filename(target)));
    if source.is_dir() {
        // Copy db from directory
        let mut found_db = false;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.eq_ignore_ascii_case("db"))
                .unwrap_or(false)
            {
                copy_file_atomic(&path, &cached_dataset)?;
                found_db = true;
                break;
            }
        }
        if !found_db {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "no .db file found in source directory",
            )));
        }

        // ship_data.csv
        let ship_src = source.join("ship_data.csv");
        if ship_src.exists() {
            let cached_ship = PathBuf::from(cache_dir).join(format!("local-{}", "ship_data.csv"));
            copy_file_atomic(&ship_src, &cached_ship)?;
            let checksum = compute_sha256_hex(&cached_ship)?;
            write_checksum_sidecar(&cached_ship, &checksum)?;
        }
    } else if source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        extract_archive(source, &cached_dataset)?;

        // Extract ship_data.csv if present
        let file = File::open(source)?;
        let mut archive = ZipArchive::new(file)?;
        for idx in 0..archive.len() {
            let mut entry = archive.by_index(idx)?;
            if !entry.is_file() {
                continue;
            }
            if let Some(path) = entry.enclosed_name() {
                let lname = path.to_string_lossy().to_ascii_lowercase();
                if lname.ends_with("ship_data.csv") {
                    let cached_ship =
                        PathBuf::from(cache_dir).join(format!("local-{}", "ship_data.csv"));
                    let mut tmp = NamedTempFile::new_in(cache_dir)?;
                    io::copy(&mut entry, tmp.as_file_mut())?;
                    tmp.flush()?;
                    tmp.persist(&cached_ship).map_err(|err| err.error)?;
                    let checksum = compute_sha256_hex(&cached_ship)?;
                    write_checksum_sidecar(&cached_ship, &checksum)?;
                    break;
                }
            }
        }
    } else {
        copy_file_atomic(source, &cached_dataset)?;
    }

    copy_cached_to_target(&cached_dataset, target)
}

fn dataset_cache_dir() -> Result<PathBuf> {
    if let Some(override_dir) = env::var_os(CACHE_DIR_ENV) {
        return Ok(PathBuf::from(override_dir));
    }

    let dirs = BaseDirs::new().ok_or(Error::CacheDirsUnavailable)?;
    Ok(dirs.cache_dir().join(CACHE_DIR_NAME))
}

fn build_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(user_agent())
        .build()
        .map_err(Error::Http)
}

fn user_agent() -> String {
    format!(
        "evefrontier-lib/{version} ({repo})",
        version = env!("CARGO_PKG_VERSION"),
        repo = "https://github.com/scetrov/evefrontier-rs"
    )
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    content_type: Option<String>,
}

fn fetch_release(client: &Client, release: &DatasetRelease) -> Result<ReleaseResponse> {
    let url = match release {
        DatasetRelease::Latest => format!("{}/latest", RELEASES_API_BASE),
        DatasetRelease::Tag(tag) => format!("{}/tags/{}", RELEASES_API_BASE, tag),
    };

    let result = retry_loop(
        || {
            let response = client
                .get(&url)
                .header(ACCEPT, "application/vnd.github+json")
                .send()?;
            let checked = response.error_for_status();
            match checked {
                Ok(ok) => ok.json::<ReleaseResponse>(),
                Err(err) => Err(err),
            }
        },
        |err: &reqwest::Error| {
            // Decide if we should retry this reqwest error.
            if let Some(status) = err.status() {
                if status == StatusCode::NOT_FOUND {
                    // 404 is terminal; convert later for tagged releases.
                    return false;
                }
                return status.is_server_error();
            }
            // Network/timeout: retry.
            true
        },
    );

    match result {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            if let Some(status) = err.status() {
                if status == StatusCode::NOT_FOUND {
                    if let DatasetRelease::Tag(tag) = release {
                        return Err(Error::DatasetReleaseNotFound { tag: tag.clone() });
                    }
                }
            }
            Err(Error::Http(err))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Database,
    Archive,
    /// A CSV file containing ship data (e.g., `ship_data.csv`).
    ShipCsv,
}

#[derive(Debug)]
struct AssetInfo {
    name: String,
    download_url: String,
    kind: AssetKind,
}

fn select_dataset_asset(release: &ReleaseResponse) -> Option<AssetInfo> {
    let mut archive_candidate: Option<AssetInfo> = None;
    for asset in &release.assets {
        let kind = classify_asset(asset);
        match kind {
            Some(AssetKind::Database) => {
                return Some(AssetInfo {
                    name: asset.name.clone(),
                    download_url: asset.browser_download_url.clone(),
                    kind: AssetKind::Database,
                });
            }
            Some(AssetKind::Archive) => {
                archive_candidate = Some(AssetInfo {
                    name: asset.name.clone(),
                    download_url: asset.browser_download_url.clone(),
                    kind: AssetKind::Archive,
                });
            }
            Some(AssetKind::ShipCsv) => {
                // Ship CSVs are not dataset assets; skip and allow DB/archive selection.
                continue;
            }
            None => continue,
        }
    }

    archive_candidate
}

fn classify_asset(asset: &ReleaseAsset) -> Option<AssetKind> {
    let name = asset.name.to_ascii_lowercase();
    if name.ends_with(".db") || name.ends_with(".sqlite") {
        return Some(AssetKind::Database);
    }
    // Accept explicit ship data CSV names or names that include `ship_data`.
    if name.ends_with("ship_data.csv") || name.contains("ship_data") {
        return Some(AssetKind::ShipCsv);
    }

    if name.ends_with(".zip") {
        return Some(AssetKind::Archive);
    }

    if let Some(content_type) = &asset.content_type {
        if content_type.contains("zip") {
            return Some(AssetKind::Archive);
        }
    }

    None
}

#[allow(dead_code)]
fn select_ship_asset(release: &ReleaseResponse) -> Option<AssetInfo> {
    for asset in &release.assets {
        if let Some(AssetKind::ShipCsv) = classify_asset(asset) {
            return Some(AssetInfo {
                name: asset.name.clone(),
                download_url: asset.browser_download_url.clone(),
                kind: AssetKind::ShipCsv,
            });
        }
    }

    None
}

fn cache_file_name(asset: &AssetInfo, release: &ReleaseResponse, target: &Path) -> String {
    let tag = sanitize_component(&release.tag_name);
    match asset.kind {
        AssetKind::Database => format!("{}-{}", tag, sanitize_component(&asset.name)),
        AssetKind::Archive => format!("{}-{}", tag, target_dataset_filename(target)),
        AssetKind::ShipCsv => format!("{}-{}", tag, sanitize_component(&asset.name)),
    }
}

fn target_dataset_filename(target: &Path) -> String {
    target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "static_data.db".to_string())
}

fn sanitize_component(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

fn download_database_asset(client: &Client, url: &str, destination: &Path) -> Result<()> {
    let mut tmp = NamedTempFile::new_in(destination.parent().unwrap_or_else(|| Path::new(".")))?;
    download_to_file(client, url, tmp.as_file_mut())?;
    tmp.flush()?;
    tmp.persist(destination).map_err(|err| err.error)?;
    Ok(())
}

fn download_archive_asset(client: &Client, url: &str, destination: &Path) -> Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let mut archive_tmp = NamedTempFile::new_in(parent)?;
    download_to_file(client, url, archive_tmp.as_file_mut())?;
    archive_tmp.flush()?;
    extract_archive(archive_tmp.path(), destination)
}

fn download_ship_asset(client: &Client, url: &str, destination: &Path) -> Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)?;
    download_to_file(client, url, tmp.as_file_mut())?;
    tmp.flush()?;
    tmp.persist(destination).map_err(|err| err.error)?;
    Ok(())
}

fn compute_sha256_hex(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_checksum_sidecar(file: &Path, checksum: &str) -> Result<()> {
    let sidecar_name = format!("{}.sha256", file.file_name().unwrap().to_string_lossy());
    let sidecar_path = file.with_file_name(sidecar_name);
    fs::write(sidecar_path, checksum)?;
    Ok(())
}

fn read_checksum_sidecar(file: &Path) -> Result<Option<String>> {
    let sidecar_name = format!("{}.sha256", file.file_name().unwrap().to_string_lossy());
    let sidecar_path = file.with_file_name(sidecar_name);
    if !sidecar_path.exists() {
        return Ok(None);
    }
    let s = fs::read_to_string(sidecar_path)?;
    Ok(Some(s.trim().to_string()))
}

fn download_to_file(client: &Client, url: &str, file: &mut File) -> Result<()> {
    // We treat both HTTP server/network errors and IO streaming errors as transient.
    #[derive(Debug)]
    enum DownloadError {
        Http(reqwest::Error),
        Io(io::Error),
    }

    impl std::fmt::Display for DownloadError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DownloadError::Http(e) => write!(f, "{}", e),
                DownloadError::Io(e) => write!(f, "{}", e),
            }
        }
    }
    impl std::error::Error for DownloadError {}

    let result = retry_loop(
        || {
            // Reset file to avoid appending partial data from previous attempt.
            file.set_len(0).map_err(DownloadError::Io)?;
            file.seek(SeekFrom::Start(0)).map_err(DownloadError::Io)?;
            let response = client.get(url).send().map_err(DownloadError::Http)?;
            let mut response = response.error_for_status().map_err(DownloadError::Http)?;
            io::copy(&mut response, file).map_err(DownloadError::Io)?;
            file.flush().map_err(DownloadError::Io)?;
            Ok(())
        },
        |err: &DownloadError| match err {
            DownloadError::Http(e) => {
                if let Some(status) = e.status() {
                    if status.is_server_error() {
                        return true; // retry 5xx
                    }
                    // Client errors (4xx) are terminal.
                    return false;
                }
                // Network/timeouts: retry.
                true
            }
            DownloadError::Io(_) => true, // treat streaming IO errors as transient
        },
    );

    match result {
        Ok(()) => Ok(()),
        Err(DownloadError::Http(err)) => Err(Error::Http(err)),
        Err(DownloadError::Io(err)) => Err(Error::Io(err)),
    }
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or(Error::CacheDirsUnavailable)?
        .to_path_buf();
    fs::create_dir_all(&parent)?;

    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx)?;
        if !entry.is_file() {
            continue;
        }

        // Explicit validation: skip entries with path traversal attempts
        let safe_name = match entry.enclosed_name() {
            Some(path) => path.to_string_lossy().to_ascii_lowercase(),
            None => {
                // enclosed_name() returns None for unsafe paths like "../../../etc/passwd"
                warn!(
                    "skipping archive entry with unsafe path in {}",
                    archive_path.display()
                );
                continue;
            }
        };

        if safe_name.ends_with(".db") {
            let mut tmp = NamedTempFile::new_in(&parent)?;
            io::copy(&mut entry, tmp.as_file_mut())?;
            tmp.flush()?;
            if destination.exists() {
                fs::remove_file(destination)?;
            }
            tmp.persist(destination).map_err(|err| err.error)?;
            return Ok(());
        }
    }

    Err(Error::ArchiveMissingDatabase {
        archive: archive_path.to_path_buf(),
    })
}

fn copy_cached_to_target(cached: &Path, target: &Path) -> Result<()> {
    if cached == target {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
        copy_file_atomic(cached, target)
    } else {
        warn!(target = %target.display(), "dataset target path lacks parent directory");
        copy_file_atomic(cached, target)
    }
}

fn copy_file_atomic(source: &Path, destination: &Path) -> Result<()> {
    if source == destination {
        return Ok(());
    }
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let mut reader = File::open(source)?;
    let mut tmp = NamedTempFile::new_in(parent)?;
    io::copy(&mut reader, tmp.as_file_mut())?;
    tmp.flush()?;
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    tmp.persist(destination).map_err(|err| err.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_detects_ship_data_by_name() {
        let asset = ReleaseAsset {
            name: "ship_data.csv".to_string(),
            browser_download_url: "https://example.com/ship_data.csv".to_string(),
            content_type: Some("text/csv".to_string()),
        };

        assert_eq!(classify_asset(&asset), Some(AssetKind::ShipCsv));
    }

    #[test]
    fn classify_detects_ship_data_by_inclusion() {
        let asset = ReleaseAsset {
            name: "datasets_ship_data_v2.csv".to_string(),
            browser_download_url: "https://example.com/datasets_ship_data_v2.csv".to_string(),
            content_type: Some("text/csv".to_string()),
        };

        assert_eq!(classify_asset(&asset), Some(AssetKind::ShipCsv));
    }

    #[test]
    fn select_ship_asset_picks_ship_csv() {
        let release = ReleaseResponse {
            tag_name: "e6c3".to_string(),
            assets: vec![
                ReleaseAsset {
                    name: "static_data.db".to_string(),
                    browser_download_url: "https://example.com/static_data.db".to_string(),
                    content_type: Some("application/octet-stream".to_string()),
                },
                ReleaseAsset {
                    name: "ship_data.csv".to_string(),
                    browser_download_url: "https://example.com/ship_data.csv".to_string(),
                    content_type: Some("text/csv".to_string()),
                },
            ],
        };

        let selected = select_ship_asset(&release).expect("ship asset should be found");
        assert_eq!(selected.kind, AssetKind::ShipCsv);
        assert!(selected.name.contains("ship_data"));
    }
}
