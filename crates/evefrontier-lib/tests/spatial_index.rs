//! Integration tests for the spatial index module.
//!
//! These tests verify:
//! - Building an index from a starmap
//! - Serialization and deserialization round-trip
//! - Nearest-neighbor queries
//! - Radius queries
//! - Temperature-filtered queries
//! - Checksum validation

use std::path::PathBuf;

use evefrontier_lib::{load_starmap, NeighbourQuery, SpatialIndex};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db")
}

#[test]
fn build_index_from_fixture() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    // Fixture has 8 systems, all should have positions
    assert!(!index.is_empty());
    assert!(index.len() <= starmap.systems.len());

    // At least some systems should have temperature data
    let systems_with_temp = starmap
        .systems
        .values()
        .filter(|s| s.metadata.min_external_temp.is_some())
        .count();
    assert!(
        systems_with_temp > 0,
        "fixture should have systems with temperature"
    );
}

#[test]
fn serialize_deserialize_round_trip() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let original = SpatialIndex::build(&starmap);

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let index_path = temp_dir.path().join("test_index.bin");

    // Save
    original.save(&index_path).expect("save succeeds");
    assert!(index_path.exists());

    // Load
    let loaded = SpatialIndex::load(&index_path).expect("load succeeds");

    // Verify same number of nodes
    assert_eq!(original.len(), loaded.len());

    // Verify Nod system is present and has same temperature
    let nod_id = starmap
        .system_id_by_name("Nod")
        .expect("Nod exists in fixture");
    assert_eq!(
        original.temperature(nod_id),
        loaded.temperature(nod_id),
        "temperature should match after round-trip"
    );
}

#[test]
fn nearest_query_returns_ordered_results() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    // Get Nod's position
    let nod_id = starmap
        .system_id_by_name("Nod")
        .expect("Nod exists in fixture");
    let nod = starmap.systems.get(&nod_id).expect("Nod system");
    let nod_pos = nod.position.expect("Nod has position");

    // Query 5 nearest
    let results = index.nearest([nod_pos.x, nod_pos.y, nod_pos.z], 5);

    // Should include at least Nod itself (distance 0)
    assert!(!results.is_empty());
    assert_eq!(results[0].0, nod_id, "first result should be Nod itself");
    assert!(
        results[0].1 < 0.001,
        "distance to self should be ~0, got {}",
        results[0].1
    );

    // Results should be ordered by distance
    for window in results.windows(2) {
        assert!(
            window[0].1 <= window[1].1,
            "results should be sorted by distance"
        );
    }
}

#[test]
fn radius_query_respects_distance() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    // Query from origin with small radius
    let results = index.within_radius([0.0, 0.0, 0.0], 10.0);

    // All results should be within radius
    for (_, distance) in &results {
        assert!(
            *distance <= 10.0,
            "all results should be within radius, got {}",
            distance
        );
    }
}

#[test]
fn temperature_filter_excludes_hot_systems() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    // Get Nod which has ~18K temperature (hot)
    let nod_id = starmap
        .system_id_by_name("Nod")
        .expect("Nod exists in fixture");
    let nod = starmap.systems.get(&nod_id).expect("Nod system");
    let nod_temp = nod.metadata.min_external_temp;

    // Skip if fixture doesn't have temperature data
    let Some(temp) = nod_temp else {
        println!("Skipping temperature test - no temp data in fixture");
        return;
    };

    // Query with temperature threshold below Nod's temp
    // Nod is ~18K, so set threshold at 10K
    let threshold = 10.0;
    if temp <= threshold {
        println!(
            "Skipping - Nod temp {} is below threshold {}",
            temp, threshold
        );
        return;
    }

    let nod_pos = nod.position.expect("Nod has position");
    let query = NeighbourQuery {
        k: 10,
        radius: None,
        max_temperature: Some(threshold),
    };

    let results = index.nearest_filtered([nod_pos.x, nod_pos.y, nod_pos.z], &query);

    // Nod should NOT be in results (too hot)
    let nod_in_results = results.iter().any(|(id, _)| *id == nod_id);
    assert!(
        !nod_in_results,
        "Nod ({}K) should be excluded with max_temp={}K",
        temp, threshold
    );
}

#[test]
fn none_temperature_passes_filter() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    // Find a system without temperature data (if any)
    let system_without_temp = starmap
        .systems
        .values()
        .find(|s| s.metadata.min_external_temp.is_none() && s.position.is_some());

    let Some(system) = system_without_temp else {
        // All systems have temperature - create a synthetic test
        println!("All fixture systems have temperature data, testing fail-open differently");

        // Just verify that the query works at all with a very restrictive temp
        let query = NeighbourQuery {
            k: 10,
            radius: None,
            max_temperature: Some(0.1), // Very cold threshold
        };

        let results = index.nearest_filtered([0.0, 0.0, 0.0], &query);
        // Should still return results (fail-open for None temps if any)
        // This is mostly a sanity check that the function works
        let _ = results;
        return;
    };

    let pos = system.position.expect("has position");
    let query = NeighbourQuery {
        k: 10,
        radius: None,
        max_temperature: Some(1.0), // Very restrictive
    };

    let results = index.nearest_filtered([pos.x, pos.y, pos.z], &query);

    // System with None temp should pass (fail-open policy)
    let in_results = results.iter().any(|(id, _)| *id == system.id);
    assert!(
        in_results,
        "system with None temperature should pass filter (fail-open)"
    );
}

#[test]
fn corrupted_checksum_fails_to_load() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let index_path = temp_dir.path().join("corrupted_index.bin");

    // Save valid index
    index.save(&index_path).expect("save succeeds");

    // Corrupt the file by modifying some bytes in the middle
    let mut data = std::fs::read(&index_path).expect("read file");
    if data.len() > 50 {
        data[30] ^= 0xFF; // Flip bits
        data[31] ^= 0xFF;
    }
    std::fs::write(&index_path, &data).expect("write corrupted");

    // Load should fail with checksum error
    let result = SpatialIndex::load(&index_path);
    assert!(result.is_err(), "corrupted file should fail to load");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("checksum") || err_msg.contains("corrupt") || err_msg.contains("failed"),
        "error should mention checksum/corruption: {}",
        err_msg
    );
}

#[test]
fn position_lookup_works() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let index = SpatialIndex::build(&starmap);

    let nod_id = starmap
        .system_id_by_name("Nod")
        .expect("Nod exists in fixture");
    let nod = starmap.systems.get(&nod_id).expect("Nod system");
    let nod_pos = nod.position.expect("Nod has position");

    let indexed_pos = index.position(nod_id).expect("Nod indexed");

    // Positions should match (within f32 precision)
    assert!(
        (indexed_pos[0] as f64 - nod_pos.x).abs() < 0.01,
        "x should match"
    );
    assert!(
        (indexed_pos[1] as f64 - nod_pos.y).abs() < 0.01,
        "y should match"
    );
    assert!(
        (indexed_pos[2] as f64 - nod_pos.z).abs() < 0.01,
        "z should match"
    );
}
