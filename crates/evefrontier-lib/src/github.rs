use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::BaseDirs;
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use serde::Deserialize;
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::error::{Error, Result};

const RELEASES_API_BASE: &str =
    "https://api.github.com/repos/Scetrov/evefrontier_datasets/releases";
const CACHE_DIR_NAME: &str = "evefrontier_datasets";
const DATASET_SOURCE_ENV: &str = "EVEFRONTIER_DATASET_SOURCE";
const LATEST_TAG_OVERRIDE_ENV: &str = "EVEFRONTIER_DATASET_LATEST_TAG";

/// Identifier for a GitHub dataset release to download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatasetRelease {
    /// Download the latest published release.
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

impl Default for DatasetRelease {
    fn default() -> Self {
        DatasetRelease::Latest
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
        return Ok(match release {
            DatasetRelease::Latest => "latest".to_string(),
            DatasetRelease::Tag(tag) => tag,
        });
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
        }
    } else {
        debug!(path = %cached_dataset.display(), "using cached dataset asset");
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
    if source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        extract_archive(source, &cached_dataset)?;
    } else {
        copy_file_atomic(source, &cached_dataset)?;
    }

    copy_cached_to_target(&cached_dataset, target)
}

fn dataset_cache_dir() -> Result<PathBuf> {
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

    let response = client
        .get(&url)
        .header(ACCEPT, "application/vnd.github+json")
        .send()?;

    if response.status() == StatusCode::NOT_FOUND {
        if let DatasetRelease::Tag(tag) = release {
            return Err(Error::DatasetReleaseNotFound { tag: tag.clone() });
        }
    }

    let response = response.error_for_status()?;
    Ok(response.json::<ReleaseResponse>()?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Database,
    Archive,
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

fn cache_file_name(asset: &AssetInfo, release: &ReleaseResponse, target: &Path) -> String {
    let tag = sanitize_component(&release.tag_name);
    match asset.kind {
        AssetKind::Database => format!("{}-{}", tag, sanitize_component(&asset.name)),
        AssetKind::Archive => format!("{}-{}", tag, target_dataset_filename(target)),
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

fn download_to_file(client: &Client, url: &str, file: &mut File) -> Result<()> {
    let mut response = client.get(url).send()?.error_for_status()?;
    io::copy(&mut response, file)?;
    Ok(())
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

        let name = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();

        if name.ends_with(".db") {
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
