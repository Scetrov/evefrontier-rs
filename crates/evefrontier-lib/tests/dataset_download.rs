use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use evefrontier_lib::github::{download_dataset, download_latest_dataset, DatasetRelease};
use tempfile::tempdir;
use zip::write::FileOptions;
use zip::ZipWriter;

const DATASET_SOURCE_ENV: &str = "EVEFRONTIER_DATASET_SOURCE";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

fn with_dataset_override<F>(path: &Path, f: F)
where
    F: FnOnce(),
{
    std::env::set_var(DATASET_SOURCE_ENV, path);
    let guard = ScopeGuard;
    f();
    drop(guard);
}

struct ScopeGuard;

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        std::env::remove_var(DATASET_SOURCE_ENV);
    }
}

#[test]
fn download_from_local_database_override() -> evefrontier_lib::Result<()> {
    let fixture = fixture_path();
    let temp_dir = tempdir()?;
    let target = temp_dir.path().join("static_data.db");

    with_dataset_override(&fixture, || {
        download_latest_dataset(&target).expect("download succeeds");
    });

    assert!(target.exists(), "target file should exist");
    let original = fs::read(fixture)?;
    let copied = fs::read(&target)?;
    assert_eq!(original, copied, "dataset contents should match override");

    Ok(())
}

#[test]
fn download_from_local_archive_override() -> evefrontier_lib::Result<()> {
    let fixture = fixture_path();
    let temp_dir = tempdir()?;
    let archive_path = temp_dir.path().join("fixture.zip");

    create_zip_with_file(&archive_path, &fixture)?;

    let target = temp_dir.path().join("static_data.db");
    with_dataset_override(&archive_path, || {
        download_latest_dataset(&target).expect("download succeeds");
    });

    assert!(target.exists(), "target file should exist");
    let original = fs::read(fixture)?;
    let copied = fs::read(&target)?;
    assert_eq!(
        original, copied,
        "dataset contents should match extracted archive"
    );

    Ok(())
}

#[test]
fn download_specific_release_from_override() -> evefrontier_lib::Result<()> {
    let fixture = fixture_path();
    let temp_dir = tempdir()?;
    let target = temp_dir.path().join("static_data.db");

    with_dataset_override(&fixture, || {
        download_dataset(&target, DatasetRelease::tag("e6c2"))
            .expect("download succeeds with override");
    });

    assert!(target.exists(), "target file should exist");
    let original = fs::read(fixture)?;
    let copied = fs::read(&target)?;
    assert_eq!(original, copied, "dataset contents should match override");

    Ok(())
}

fn create_zip_with_file(archive_path: &Path, source: &Path) -> evefrontier_lib::Result<()> {
    let data = fs::read(source)?;
    let file = fs::File::create(archive_path)?;
    let mut writer = ZipWriter::new(file);
    writer.start_file("c3e6-static_data.db", FileOptions::default())?;
    writer.write_all(&data)?;
    writer.finish()?;
    Ok(())
}
