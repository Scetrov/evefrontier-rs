use std::path::PathBuf;

use evefrontier_lib::ship::ShipCatalog;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures")
}

/// Test that ship_data.csv can be located and loaded from the fixtures directory.
/// This test validates that the ship data fixture is present and properly formatted.
#[test]
fn ship_data_fixture_exists_and_loads() {
    let ship_data_path = fixture_path().join("ship_data.csv");
    assert!(
        ship_data_path.exists(),
        "ship_data.csv fixture must exist at {:?}",
        ship_data_path
    );

    let catalog =
        ShipCatalog::from_path(&ship_data_path).expect("ship_data.csv should load successfully");
    assert!(
        !catalog.ship_names().is_empty(),
        "ship_data.csv should contain at least one ship"
    );
}

/// Test that ship_data.csv fixture contains the expected default ship (Reflex).
#[test]
fn ship_data_fixture_contains_reflex() {
    let ship_data_path = fixture_path().join("ship_data.csv");
    let catalog = ShipCatalog::from_path(&ship_data_path).expect("fixture should load");
    let reflex = catalog.get("Reflex");
    assert!(reflex.is_some(), "Reflex ship must be present in fixture");
}

/// Test that ship_data.csv can be parsed with various ship entries.
#[test]
fn ship_data_fixture_parse_multiple_ships() {
    let ship_data_path = fixture_path().join("ship_data.csv");
    let catalog = ShipCatalog::from_path(&ship_data_path).expect("fixture should load");

    // Verify at least a few ships exist
    let reflex = catalog.get("Reflex");
    assert!(reflex.is_some(), "Reflex should be available");

    // Verify ship attributes are valid
    if let Some(ship) = reflex {
        assert!(ship.base_mass_kg > 0.0, "Ship mass must be positive");
        assert!(
            ship.fuel_capacity > 0.0,
            "Ship fuel capacity must be positive"
        );
        assert!(
            ship.cargo_capacity >= 0.0,
            "Ship cargo capacity must be non-negative"
        );
    }
}

/// Specification: Ship data downloader should be idempotent.
///
/// When called multiple times with the same target path:
/// - First call downloads/caches the ship_data.csv
/// - Subsequent calls skip download and use cached version
/// - All calls return successfully
///
/// Note: Actual GitHub downloader integration is deferred to a future PR.
/// For now, this test documents the expected behavior specification.
#[test]
#[ignore = "Future: Requires GitHub downloader enhancement for ship_data.csv"]
fn ship_data_downloader_is_idempotent() {
    // let mut temp_dir = tempfile::tempdir().expect("create temp dir");
    // let ship_data_path = temp_dir.path().join("ship_data.csv");
    //
    // // First download
    // download_latest_ship_data(&ship_data_path)
    //     .expect("first download should succeed");
    // assert!(ship_data_path.exists(), "ship_data should be written");
    //
    // let first_metadata = std::fs::metadata(&ship_data_path).expect("stat file");
    // std::thread::sleep(std::time::Duration::from_millis(100));
    //
    // // Second download (should use cache)
    // download_latest_ship_data(&ship_data_path)
    //     .expect("second download should succeed");
    //
    // let second_metadata = std::fs::metadata(&ship_data_path).expect("stat file");
    // assert_eq!(
    //     first_metadata.modified(),
    //     second_metadata.modified(),
    //     "cached file should not be re-downloaded"
    // );
}

/// Specification: Ship data downloader should extract from archives.
///
/// When a .zip release contains ship_data.csv:
/// - The downloader should extract the CSV from the archive
/// - Atomic write to temporary file, then rename to target
/// - Checksum validation should ensure integrity
///
/// Note: This behavior is planned but not yet implemented.
#[test]
#[ignore = "Future: Requires GitHub downloader enhancement for archive extraction"]
fn ship_data_extracts_from_zip_archive() {
    // Future implementation spec:
    // - Create a test archive with ship_data.csv
    // - Call downloader with archive as override source
    // - Verify CSV is extracted
    // - Verify atomic write semantics
    // - Verify checksum marker (version 2 format)
}

/// Specification: Ship data should include checksum marker.
///
/// The downloader should write metadata alongside ship_data.csv:
/// - Version identifier
/// - Source hash (SHA-256 of CSV content)
/// - Release tag that provided the data
/// - Timestamp of download
///
/// This enables cache invalidation and freshness verification.
#[test]
#[ignore = "Future: Requires checksum marker implementation"]
fn ship_data_includes_freshness_metadata() {
    // Future implementation spec:
    // - After download, check for ship_data.csv.meta file
    // - Verify metadata contains: version, hash, tag, timestamp
    // - Use metadata to determine if re-download is needed
}
