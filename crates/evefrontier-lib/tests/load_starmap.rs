use std::path::PathBuf;

use evefrontier_lib::{build_graph, find_route, load_starmap, Error, Result};
use rusqlite::Connection;
use tempfile::NamedTempFile;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

#[test]
fn load_fixture_and_find_route() -> Result<()> {
    let path = fixture_path();
    let starmap = load_starmap(&path)?;

    assert_eq!(starmap.systems.len(), 8, "fixture should have 8 systems");
    // Note: E1J-M5G has no gates, so it won't have adjacency entries in gate-only graph
    assert!(
        starmap.adjacency.len() >= 7,
        "systems with gates should have adjacency"
    );

    let start = starmap.system_id_by_name("Nod").expect("start exists");
    let goal = starmap.system_id_by_name("Brana").expect("goal exists");

    let start_system = starmap
        .systems
        .get(&start)
        .expect("system metadata available");
    assert!(start_system.metadata.constellation_id.is_some());
    assert!(start_system.metadata.constellation_name.is_some());
    assert!(start_system.metadata.region_id.is_some());
    assert!(start_system.metadata.region_name.is_some());
    assert!(start_system.metadata.security_status.is_none());
    assert!(start_system.position.is_some());

    let graph = build_graph(&starmap);
    let route = find_route(&graph, start, goal).expect("route should exist");

    assert_eq!(route.first().copied(), Some(start));
    assert_eq!(route.last().copied(), Some(goal));
    assert!(route.len() >= 2);

    Ok(())
}

#[test]
fn load_legacy_schema() -> Result<()> {
    let file = NamedTempFile::new()?;
    let conn = Connection::open(file.path())?;
    conn.execute_batch(
        r#"
        CREATE TABLE mapSolarSystems (
            solarSystemID INTEGER PRIMARY KEY,
            solarSystemName TEXT NOT NULL
        );
        CREATE TABLE mapSolarSystemJumps (
            fromSolarSystemID INTEGER NOT NULL,
            toSolarSystemID INTEGER NOT NULL
        );
        INSERT INTO mapSolarSystems (solarSystemID, solarSystemName) VALUES
            (1, 'Alpha'),
            (2, 'Beta'),
            (3, 'Gamma');
        INSERT INTO mapSolarSystemJumps (fromSolarSystemID, toSolarSystemID) VALUES
            (1, 2),
            (2, 3);
        "#,
    )?;
    drop(conn);

    let starmap = load_starmap(file.path())?;
    assert_eq!(starmap.systems.len(), 3);
    assert_eq!(starmap.adjacency.len(), 3);

    for system in starmap.systems.values() {
        assert!(system.metadata.constellation_id.is_none());
        assert!(system.metadata.region_id.is_none());
        assert!(system.metadata.security_status.is_none());
        assert!(system.position.is_none());
    }

    Ok(())
}

#[test]
fn rejects_schema_with_missing_columns() {
    let file = NamedTempFile::new().expect("tempfile");
    let conn = Connection::open(file.path()).expect("open temp db");
    conn.execute_batch(
        r#"
        CREATE TABLE SolarSystems (
            solarSystemId INTEGER PRIMARY KEY
        );
        CREATE TABLE Jumps (
            fromSystemId INTEGER NOT NULL,
            toSystemId INTEGER NOT NULL
        );
        "#,
    )
    .expect("create schema");
    drop(conn);

    let err = load_starmap(file.path()).expect_err("should reject schema");
    assert!(matches!(err, Error::UnsupportedSchema));
}
