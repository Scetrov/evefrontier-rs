use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, Row};
use tracing::debug;

use crate::error::{Error, Result};

/// Numeric identifier for a solar system.
pub type SystemId = i64;

/// Minimal representation of a solar system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub id: SystemId,
    pub name: String,
}

/// In-memory representation of the starmap graph.
#[derive(Debug, Clone, Default)]
pub struct Starmap {
    pub systems: HashMap<SystemId, System>,
    pub name_to_id: HashMap<String, SystemId>,
    pub adjacency: Arc<HashMap<SystemId, Vec<SystemId>>>,
}

impl Starmap {
    /// Lookup a system identifier by its case-sensitive name.
    pub fn system_id_by_name(&self, name: &str) -> Option<SystemId> {
        self.name_to_id.get(name).copied()
    }

    /// Lookup a system name by identifier.
    pub fn system_name(&self, id: SystemId) -> Option<&str> {
        self.systems.get(&id).map(|sys| sys.name.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaVariant {
    StaticData,
    LegacyMap,
}

/// Load systems and jumps from a dataset into memory.
pub fn load_starmap(db_path: &Path) -> Result<Starmap> {
    let connection = Connection::open(db_path)?;
    let schema = detect_schema(&connection)?;
    debug!(?schema, path = %db_path.display(), "loading starmap");

    let systems = load_systems(&connection, schema)?;
    let adjacency = Arc::new(load_adjacency(&connection, schema)?);

    let mut name_to_id = HashMap::new();
    for system in systems.values() {
        name_to_id.insert(system.name.clone(), system.id);
    }

    Ok(Starmap {
        systems,
        name_to_id,
        adjacency,
    })
}

fn detect_schema(connection: &Connection) -> Result<SchemaVariant> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('SolarSystems', 'mapSolarSystems')",
    )?;
    let mut rows = stmt.query([])?;

    let mut found = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        found.push(name);
    }

    if found.iter().any(|t| t == "SolarSystems") {
        Ok(SchemaVariant::StaticData)
    } else if found.iter().any(|t| t == "mapSolarSystems") {
        Ok(SchemaVariant::LegacyMap)
    } else {
        Err(Error::UnsupportedSchema)
    }
}

fn load_systems(
    connection: &Connection,
    schema: SchemaVariant,
) -> Result<HashMap<SystemId, System>> {
    let (sql, id_col, name_col) = match schema {
        SchemaVariant::StaticData => (
            "SELECT solarSystemId, name FROM SolarSystems",
            0usize,
            1usize,
        ),
        SchemaVariant::LegacyMap => (
            "SELECT solarSystemID, solarSystemName FROM mapSolarSystems",
            0usize,
            1usize,
        ),
    };

    let mut stmt = connection.prepare(sql)?;
    let rows = stmt.query_map([], |row| row_to_system(row, id_col, name_col))?;

    let mut systems = HashMap::new();
    for entry in rows {
        let system = entry?;
        systems.insert(system.id, system);
    }
    Ok(systems)
}

/// Load jump connections into adjacency lists.
///
/// The loader inserts edges in both directions for every row, assuming that
/// jumps are bidirectional gate connections. One-way travel (such as wormholes)
/// is not currently modeled and would require schema changes or additional
/// metadata to represent directionality accurately.
fn load_adjacency(
    connection: &Connection,
    schema: SchemaVariant,
) -> Result<HashMap<SystemId, Vec<SystemId>>> {
    let sql = match schema {
        SchemaVariant::StaticData => "SELECT fromSystemId, toSystemId FROM Jumps",
        SchemaVariant::LegacyMap => {
            "SELECT fromSolarSystemID, toSolarSystemID FROM mapSolarSystemJumps"
        }
    };

    let mut stmt = connection.prepare(sql)?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut adjacency: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
    for row in rows {
        let (from, to): (SystemId, SystemId) = row?;
        adjacency.entry(from).or_default().push(to);
        adjacency.entry(to).or_default().push(from);
    }

    for neighbours in adjacency.values_mut() {
        neighbours.sort_unstable();
        neighbours.dedup();
    }

    Ok(adjacency)
}

fn row_to_system(row: &Row<'_>, id_col: usize, name_col: usize) -> rusqlite::Result<System> {
    Ok(System {
        id: row.get(id_col)?,
        name: row.get(name_col)?,
    })
}
