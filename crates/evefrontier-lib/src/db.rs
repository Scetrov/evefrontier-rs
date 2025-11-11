use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, Row};
use tracing::{debug, warn};

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

impl fmt::Display for SchemaVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SchemaVariant::StaticData => "static_data",
            SchemaVariant::LegacyMap => "legacy_map",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SchemaDefinition {
    variant: SchemaVariant,
    systems_table: &'static str,
    system_id_column: &'static str,
    system_name_column: &'static str,
    jumps_table: &'static str,
    jump_from_column: &'static str,
    jump_to_column: &'static str,
}

impl SchemaVariant {
    fn definition(self) -> SchemaDefinition {
        match self {
            SchemaVariant::StaticData => SchemaDefinition {
                variant: SchemaVariant::StaticData,
                systems_table: "SolarSystems",
                system_id_column: "solarSystemId",
                system_name_column: "name",
                jumps_table: "Jumps",
                jump_from_column: "fromSystemId",
                jump_to_column: "toSystemId",
            },
            SchemaVariant::LegacyMap => SchemaDefinition {
                variant: SchemaVariant::LegacyMap,
                systems_table: "mapSolarSystems",
                system_id_column: "solarSystemID",
                system_name_column: "solarSystemName",
                jumps_table: "mapSolarSystemJumps",
                jump_from_column: "fromSolarSystemID",
                jump_to_column: "toSolarSystemID",
            },
        }
    }
}

/// Load systems and jumps from a dataset into memory.
///
/// The loader performs runtime schema detection so both the current
/// `SolarSystems`/`Jumps` tables and the legacy
/// `mapSolarSystems`/`mapSolarSystemJumps` layout are supported. It also
/// verifies that referenced jump endpoints exist in the dataset to avoid
/// propagating corrupt edges into the in-memory graph.
pub fn load_starmap(db_path: &Path) -> Result<Starmap> {
    let connection = Connection::open(db_path)?;
    let schema = detect_schema(&connection)?;
    debug!(schema = %schema.variant, path = %db_path.display(), "loading starmap");

    let systems = load_systems(&connection, &schema)?;
    let adjacency = Arc::new(load_adjacency(&connection, &schema, &systems)?);

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

fn detect_schema(connection: &Connection) -> Result<SchemaDefinition> {
    for variant in [SchemaVariant::StaticData, SchemaVariant::LegacyMap] {
        if schema_matches(connection, variant)? {
            return Ok(variant.definition());
        }
    }

    Err(Error::UnsupportedSchema)
}

fn load_systems(
    connection: &Connection,
    schema: &SchemaDefinition,
) -> Result<HashMap<SystemId, System>> {
    let sql = format!(
        "SELECT {id}, {name} FROM {table}",
        id = schema.system_id_column,
        name = schema.system_name_column,
        table = schema.systems_table
    );

    let mut stmt = connection.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row_to_system(row, 0, 1))?;

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
    schema: &SchemaDefinition,
    systems: &HashMap<SystemId, System>,
) -> Result<HashMap<SystemId, Vec<SystemId>>> {
    let sql = format!(
        "SELECT {from}, {to} FROM {table}",
        from = schema.jump_from_column,
        to = schema.jump_to_column,
        table = schema.jumps_table
    );

    let mut stmt = connection.prepare(&sql)?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut adjacency: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
    let mut skipped_edges = 0usize;
    for row in rows {
        let (from, to): (SystemId, SystemId) = row?;
        if !systems.contains_key(&from) || !systems.contains_key(&to) {
            skipped_edges += 1;
            continue;
        }
        adjacency.entry(from).or_default().push(to);
        adjacency.entry(to).or_default().push(from);
    }

    for neighbours in adjacency.values_mut() {
        neighbours.sort_unstable();
        neighbours.dedup();
    }

    if skipped_edges > 0 {
        warn!(
            skipped_edges,
            "ignored jump edges referencing unknown systems",
        );
    }

    Ok(adjacency)
}

fn row_to_system(row: &Row<'_>, id_col: usize, name_col: usize) -> rusqlite::Result<System> {
    Ok(System {
        id: row.get(id_col)?,
        name: row.get(name_col)?,
    })
}

fn schema_matches(connection: &Connection, variant: SchemaVariant) -> Result<bool> {
    let definition = variant.definition();
    if !table_exists(connection, definition.systems_table)?
        || !table_exists(connection, definition.jumps_table)?
    {
        return Ok(false);
    }

    if !table_has_columns(
        connection,
        definition.systems_table,
        &[definition.system_id_column, definition.system_name_column],
    )? {
        return Ok(false);
    }

    if !table_has_columns(
        connection,
        definition.jumps_table,
        &[definition.jump_from_column, definition.jump_to_column],
    )? {
        return Ok(false);
    }

    Ok(true)
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool> {
    let mut stmt = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")?;
    let mut rows = stmt.query([table])?;
    Ok(rows.next()?.is_some())
}

fn table_has_columns(connection: &Connection, table: &str, required: &[&str]) -> Result<bool> {
    let pragma = format!("PRAGMA table_info('{table}')");
    let mut stmt = connection.prepare(&pragma)?;
    let mut rows = stmt.query([])?;

    let mut columns = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        columns.push(name);
    }

    Ok(required.iter().all(|required| {
        columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(required))
    }))
}
