use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use evefrontier_lib::github::DatasetRelease;
use tempfile::tempdir;
use zip::write::FileOptions;
use zip::ZipWriter;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

fn with_cache_dir<F>(f: F)
where
    F: FnOnce(&Path),
{
    let dir = tempdir().expect("create temp cache");
    let cache_path = dir.path().to_path_buf();
    // pass cache path into the closure and keep `dir` alive until it finishes
    f(&cache_path);
}

#[test]
fn download_from_local_database_override() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let fixture = fixture_path();
        let fixture_copy = temp_dir.path().join("fixture.db");
        fs::copy(&fixture, &fixture_copy).unwrap();
        let target = temp_dir.path().join("static_data.db");

        // Use the direct helper to avoid global env var mutation.
        evefrontier_lib::github::download_latest_from_source_with_cache(
            &target,
            &fixture_copy,
            cache,
        )
        .expect("download succeeds");

        assert!(target.exists(), "target file should exist");
        let original = fs::read(fixture).unwrap();
        let copied = fs::read(&target).unwrap();
        assert_eq!(original, copied, "dataset contents should match override");
    });
    Ok(())
}

#[test]
fn download_from_local_archive_override() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let fixture = fixture_path();
        let fixture_copy = temp_dir.path().join("fixture.db");
        fs::copy(&fixture, &fixture_copy).unwrap();
        let archive_path = temp_dir.path().join("fixture.zip");

        create_zip_with_file(&archive_path, &fixture_copy).unwrap();

        let target = temp_dir.path().join("static_data.db");
        evefrontier_lib::github::download_latest_from_source_with_cache(
            &target,
            &archive_path,
            cache,
        )
        .expect("download succeeds");

        assert!(target.exists(), "target file should exist");
        let original = fs::read(fixture).unwrap();
        let copied = fs::read(&target).unwrap();
        assert_eq!(
            original, copied,
            "dataset contents should match extracted archive"
        );
    });

    Ok(())
}

#[test]
fn download_specific_release_from_override() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let fixture = fixture_path();
        let fixture_copy = temp_dir.path().join("fixture.db");
        fs::copy(&fixture, &fixture_copy).unwrap();
        let target = temp_dir.path().join("static_data.db");

        evefrontier_lib::github::download_from_source_with_cache(
            &target,
            DatasetRelease::tag("e6c2"),
            &fixture_copy,
            cache,
            "e6c2",
        )
        .expect("download succeeds with override");

        assert!(target.exists(), "target file should exist");
        let original = fs::read(fixture).unwrap();
        let copied = fs::read(&target).unwrap();
        assert_eq!(original, copied, "dataset contents should match override");
    });

    Ok(())
}

#[test]
fn ensure_dataset_redownloads_when_tag_changes() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let dataset_path = temp_dir.path().join("static_data.db");

        let source_one = temp_dir.path().join("source-one.db");
        fs::write(&source_one, b"first").unwrap();
        let source_two = temp_dir.path().join("source-two.db");
        fs::write(&source_two, b"second").unwrap();

        evefrontier_lib::github::download_from_source_with_cache(
            &dataset_path,
            DatasetRelease::tag("e6c3"),
            &source_one,
            cache,
            "e6c3",
        )
        .expect("initial download succeeds");

        assert_eq!(fs::read(&dataset_path).unwrap(), b"first");
        let marker_path = dataset_path.with_file_name("static_data.db.release");
        let marker = fs::read_to_string(&marker_path).unwrap();
        assert!(marker.contains("requested=tag"));
        assert!(marker.contains("resolved=e6c3"));

        evefrontier_lib::github::download_from_source_with_cache(
            &dataset_path,
            DatasetRelease::tag("e6c2"),
            &source_two,
            cache,
            "e6c2",
        )
        .expect("tag change triggers re-download");

        assert_eq!(fs::read(&dataset_path).unwrap(), b"second");
        let marker = fs::read_to_string(&marker_path).unwrap();
        assert!(marker.contains("requested=tag"));
        assert!(marker.contains("resolved=e6c2"));
    });

    Ok(())
}

#[test]
fn ensure_dataset_redownloads_when_switching_back_to_latest() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let dataset_path = temp_dir.path().join("static_data.db");

        let source_one = temp_dir.path().join("source-one.db");
        fs::write(&source_one, b"first").unwrap();
        let source_two = temp_dir.path().join("source-two.db");
        fs::write(&source_two, b"second").unwrap();

        evefrontier_lib::github::download_from_source_with_cache(
            &dataset_path,
            DatasetRelease::tag("e6c2"),
            &source_one,
            cache,
            "e6c2",
        )
        .expect("initial tagged download succeeds");

        assert_eq!(fs::read(&dataset_path).unwrap(), b"first");
        let marker_path = dataset_path.with_file_name("static_data.db.release");
        let marker = fs::read_to_string(&marker_path).unwrap();
        assert!(marker.contains("requested=tag"));
        assert!(marker.contains("resolved=e6c2"));

        evefrontier_lib::github::download_latest_from_source_with_cache(
            &dataset_path,
            &source_two,
            cache,
        )
        .expect("latest request refreshes dataset");

        assert_eq!(fs::read(&dataset_path).unwrap(), b"second");
        let marker = fs::read_to_string(&marker_path).unwrap();
        assert!(marker.contains("requested=latest"));
        assert!(marker.contains("resolved=latest"));
    });

    Ok(())
}

#[test]
fn ensure_dataset_redownloads_when_latest_release_changes() -> evefrontier_lib::Result<()> {
    with_cache_dir(|cache| {
        let temp_dir = tempdir().unwrap();
        let dataset_path = temp_dir.path().join("static_data.db");
        fs::write(&dataset_path, b"cached").unwrap();

        let marker_path = dataset_path.with_file_name("static_data.db.release");
        fs::write(&marker_path, "requested=latest\nresolved=e6c2\n").unwrap();

        let source_path = temp_dir.path().join("source-new.db");
        fs::write(&source_path, b"fresh").unwrap();

        // Simulate a new latest release by explicitly passing e6c3 as resolved tag
        evefrontier_lib::github::download_from_source_with_cache(
            &dataset_path,
            DatasetRelease::Latest,
            &source_path,
            cache,
            "e6c3",
        )
        .expect("latest change triggers re-download");

        assert_eq!(fs::read(&dataset_path).unwrap(), b"fresh");
        let marker = fs::read_to_string(&marker_path).unwrap();
        assert!(marker.contains("requested=latest"));
        assert!(marker.contains("resolved=e6c3"));
    });

    Ok(())
}

fn create_zip_with_file(archive_path: &Path, source: &Path) -> evefrontier_lib::Result<()> {
    let data = fs::read(source)?;
    let file = fs::File::create(archive_path)?;
    let mut writer = ZipWriter::new(file);
    writer.start_file("e6c3-static_data.db", FileOptions::<()>::default())?;
    writer.write_all(&data)?;
    writer.finish()?;
    Ok(())
}
