use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize, PartialEq)]
struct FixtureMetadata {
    fixture: String,
    release: String,
    sha256: String,
    tables: HashMap<String, u64>,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures")
}

fn read_release_marker() -> Option<String> {
    let marker = fixtures_dir().join("static_data.db.release");
    let contents = match std::fs::read_to_string(&marker) {
        Ok(c) => c,
        Err(_e) => return None,
    };
    for line in contents.lines() {
        if let Some(value) = line.trim().strip_prefix("resolved=") {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn read_metadata() -> FixtureMetadata {
    let path = fixtures_dir().join("minimal_static_data.meta.json");
    let file = File::open(path).expect("metadata file readable");
    serde_json::from_reader(file).expect("metadata parses")
}

fn compute_sha256(path: &Path) -> String {
    let mut file = File::open(path).expect("fixture readable");
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).expect("hash read ok");
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    format!("{:x}", hasher.finalize())
}

fn count_tables(conn: &Connection, tables: &HashMap<String, u64>) -> HashMap<String, u64> {
    let mut counts = HashMap::new();
    let tx = conn.unchecked_transaction().expect("transaction");
    for table in tables.keys() {
        let mut stmt = tx
            .prepare(&format!("SELECT COUNT(*) FROM {table}"))
            .expect("prepared count");
        let value: u64 = stmt.query_row([], |row| row.get(0)).expect("row count");
        counts.insert(table.clone(), value);
    }
    tx.commit().expect("commit");
    counts
}

#[test]
fn fixture_metadata_matches_record() {
    let fixtures = fixtures_dir();
    let fixture_db = fixtures.join("minimal/static_data.db");
    let metadata = read_metadata();

    if let Some(resolved) = read_release_marker() {
        assert!(
            resolved == metadata.release || resolved == "fixture",
            "dataset release mismatch: marker='{}' metadata='{}'",
            resolved,
            metadata.release
        );
    }

    let sha = compute_sha256(&fixture_db);
    assert_eq!(metadata.sha256, sha, "fixture hash drifted");

    let conn = Connection::open(&fixture_db).expect("open fixture DB");
    let counts = count_tables(&conn, &metadata.tables);
    assert_eq!(metadata.tables, counts, "table counts differ");
}
