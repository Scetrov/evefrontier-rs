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

#[test]
fn download_from_source_with_cache_copies_ship_csv_from_directory() {
    use std::fs;
    use tempfile::tempdir;

    let fixture_dir = fixture_path();
    let source = tempdir().expect("create source tempdir");
    let cache = tempdir().expect("create cache tempdir");

    // Copy fixture DB and ship_data.csv into source dir
    fs::copy(
        fixture_dir.join("minimal_static_data.db"),
        source.path().join("static_data.db"),
    )
    .expect("copy db");
    fs::copy(
        fixture_dir.join("ship_data.csv"),
        source.path().join("ship_data.csv"),
    )
    .expect("copy ship csv");

    let target = tempfile::tempdir().expect("target dir");
    let target_db = target.path().join("static_data.db");

    // Use the download_from_source_with_cache helper which should copy files into cache
    let resolved = evefrontier_lib::github::download_from_source_with_cache(
        &target_db,
        evefrontier_lib::github::DatasetRelease::Latest,
        source.path(),
        cache.path(),
        "e6c3",
    )
    .expect("download from source with cache");

    assert_eq!(resolved, "e6c3");

    // Check cached ship CSV and sidecar exist
    let cached_ship = cache.path().join("local-ship_data.csv");
    assert!(cached_ship.exists(), "cached ship_data.csv must exist");

    let sidecar = cache.path().join("local-ship_data.csv.sha256");
    assert!(sidecar.exists(), "checksum sidecar must exist");

    // checksum matches computed
    use sha2::{Digest, Sha256};
    let data = std::fs::read(&cached_ship).expect("read ship csv");
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual = format!("{:x}", hasher.finalize());
    let expected = std::fs::read_to_string(sidecar)
        .expect("read sidecar")
        .trim()
        .to_string();
    assert_eq!(actual, expected, "checksum should match sidecar");
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

#[test]
fn ship_catalog_from_sidecar_resolves_csv() {
    use std::fs;
    use tempfile::tempdir;

    let tmp = tempdir().expect("create temp dir");
    let csv = tmp.path().join("e6c4-ship_data.csv");
    let sidecar = tmp.path().join("e6c4-ship_data.csv.sha256");

    fs::write(
        &csv,
        "name,base_mass_kg,specific_heat,fuel_capacity,cargo_capacity\nReflex,1000,1.0,500,100",
    )
    .expect("write csv");
    fs::write(&sidecar, "deadbeef").expect("write sidecar");

    // Calling from_path with the sidecar should resolve to the CSV and succeed
    let catalog = evefrontier_lib::ship::ShipCatalog::from_path(&sidecar)
        .expect("should load from adjacent csv");
    assert!(catalog.get("Reflex").is_some());
}

#[test]
fn ship_catalog_from_sidecar_without_csv_errors() {
    use std::fs;
    use tempfile::tempdir;

    let tmp = tempdir().expect("create temp dir");
    let sidecar = tmp.path().join("e6c4-ship_data.csv.sha256");
    fs::write(&sidecar, "deadbeef").expect("write sidecar");

    let res = evefrontier_lib::ship::ShipCatalog::from_path(&sidecar);
    assert!(res.is_err(), "should error when only sidecar exists");
}

#[test]
fn ship_data_header_variants_parse() {
    use std::fs;
    use tempfile::tempdir;

    let tmp = tempdir().expect("create temp dir");
    let csv = tmp.path().join("variant_ship_data.csv");
    // Header matches what was in cache (Faction,ShipName,...SpecificHeat_C, FuelCapacity_units, Mass_kg)
    let content = "Faction,ShipName,Class,StructureHP,Capacity_m3,FuelCapacity_units,Mass_kg,VolumeUnpackaged_m3,VolumePackaged_m3,InertiaModifier,ShieldRecharge_s,Capacitor_GJ,SpecificHeat_C,Conductance_k,MaxTargetRange_km,MaxLockedTargets,SignatureRadius_m,ScanResolution_mm,MaxVelocity_mps,WarpSpeed_c\nKeep,Reflex,Corvette,1250,520,1750,9750000,13000,1000,0.4,240,50,2,0.875,100,5,50,500,184,1.75\n";
    fs::write(&csv, content).expect("write csv");

    let catalog = ShipCatalog::from_path(&csv).expect("variant header csv should parse");
    let reflex = catalog.get("Reflex");
    assert!(
        reflex.is_some(),
        "Reflex should be parsed from variant header CSV"
    );
}

/// Real-world-ish ship CSV sample from current cycle: ensure parser tolerates multi-column
/// header variants used by dataset providers and correctly maps fields like `Mass_kg` and
/// `FuelCapacity_units` to library fields.
#[test]
fn ship_data_realistic_cycle_parse() {
    use std::fs;
    use tempfile::tempdir;

    let tmp = tempdir().expect("create temp dir");
    let csv = tmp.path().join("realistic_ship_data.csv");

    let content = "Faction,ShipName,Class,StructureHP,Capacity_m3,FuelCapacity_units,Mass_kg,VolumeUnpackaged_m3,VolumePackaged_m3,InertiaModifier,ShieldRecharge_s,Capacitor_GJ,SpecificHeat_C,Conductance_k,MaxTargetRange_km,MaxLockedTargets,SignatureRadius_m,ScanResolution_mm,MaxVelocity_mps,WarpSpeed_c\n\
Keep,Wend,Shuttle,750,520,200,6800000,4000,500,0.46,240,32,2,1.5,100,5,30,500,134,1.5\n\
Synod,Carom,Corvette,1300,300,3000,7200000,12000,1000,0.45,240,50,8.5,0.875,100,5,50,500,270,1.5\n\
Synod,Stride,Corvette,1450,320,3200,7900000,12000,1000,0.56,240,40,8,0.875,100,5,50,500,255,1.5\n\
Keep,Reflex,Corvette,1250,520,1750,9750000,13000,1000,0.4,240,50,2,0.875,100,5,50,500,184,1.75\n\
Keep,Reiver,Corvette,1900,520,1416,10200000,12000,1000,0.35,240,50,1,0.875,100,5,50,500,435,1.5\n";

    fs::write(&csv, content).expect("write csv");

    let catalog = ShipCatalog::from_path(&csv).expect("realistic csv should parse");

    // Reflex should be present and key fields should map correctly
    let reflex = catalog
        .get("Reflex")
        .expect("Reflex should be parsed from realistic CSV");
    assert_eq!(reflex.base_mass_kg as i64, 9_750_000i64);
    assert_eq!(reflex.fuel_capacity as i64, 1_750i64);
    // Specific heat from header `SpecificHeat_C` should be parsed
    assert_eq!(reflex.specific_heat as i64, 2i64);
}
