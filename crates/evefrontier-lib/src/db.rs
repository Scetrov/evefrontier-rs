use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, Row};
use tracing::{debug, warn};

use crate::error::{Error, Result};

/// Numeric identifier for a solar system.
pub type SystemId = i64;

/// Cartesian coordinates for a solar system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl SystemPosition {
    /// Calculate the Euclidean distance to another position.
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Additional metadata tracked for each system.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemMetadata {
    pub constellation_id: Option<i64>,
    pub constellation_name: Option<String>,
    pub region_id: Option<i64>,
    pub region_name: Option<String>,
    pub security_status: Option<f64>,
}

impl SystemMetadata {
    fn empty() -> Self {
        Self {
            constellation_id: None,
            constellation_name: None,
            region_id: None,
            region_name: None,
            security_status: None,
        }
    }
}

/// Representation of a solar system with optional metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct System {
    pub id: SystemId,
    pub name: String,
    pub metadata: SystemMetadata,
    pub position: Option<SystemPosition>,
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
struct MetadataJoin {
    fk_column: &'static str,
    table: &'static str,
    table_id_column: &'static str,
    table_name_column: &'static str,
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
    constellation_join: Option<MetadataJoin>,
    region_join: Option<MetadataJoin>,
    security_column: Option<&'static str>,
    position_columns: Option<PositionColumns>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositionColumns {
    x: &'static str,
    y: &'static str,
    z: &'static str,
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
                constellation_join: Some(MetadataJoin {
                    fk_column: "constellationID",
                    table: "Constellations",
                    table_id_column: "constellationID",
                    table_name_column: "constellationName",
                }),
                region_join: Some(MetadataJoin {
                    fk_column: "regionID",
                    table: "Regions",
                    table_id_column: "regionID",
                    table_name_column: "regionName",
                }),
                security_column: Some("security"),
                position_columns: Some(PositionColumns {
                    x: "centerX",
                    y: "centerY",
                    z: "centerZ",
                }),
            },
            SchemaVariant::LegacyMap => SchemaDefinition {
                variant: SchemaVariant::LegacyMap,
                systems_table: "mapSolarSystems",
                system_id_column: "solarSystemID",
                system_name_column: "solarSystemName",
                jumps_table: "mapSolarSystemJumps",
                jump_from_column: "fromSolarSystemID",
                jump_to_column: "toSolarSystemID",
                constellation_join: None,
                region_join: None,
                security_column: None,
                position_columns: None,
            },
        }
    }
}

/// Load systems and jumps from a dataset into memory.
///
/// The loader performs runtime schema detection so both the current
/// `SolarSystems`/`Jumps` tables and the legacy
/// `mapSolarSystems`/`mapSolarSystemJumps` layout are supported. When metadata
/// tables are available, systems are annotated with their region,
/// constellation, and security status. The loader also verifies that
/// referenced jump endpoints exist in the dataset to avoid propagating corrupt
/// edges into the in-memory graph.
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
    if let Some(schema) = detect_static_schema(connection)? {
        return Ok(schema);
    }
    if let Some(schema) = detect_legacy_schema(connection)? {
        return Ok(schema);
    }

    Err(Error::UnsupportedSchema)
}

fn load_systems(
    connection: &Connection,
    schema: &SchemaDefinition,
) -> Result<HashMap<SystemId, System>> {
    match schema.variant {
        SchemaVariant::StaticData => load_static_systems(connection, schema),
        SchemaVariant::LegacyMap => load_legacy_systems(connection, schema),
    }
}

fn load_static_systems(
    connection: &Connection,
    schema: &SchemaDefinition,
) -> Result<HashMap<SystemId, System>> {
    let mut selects = vec![
        format!("s.{id} AS system_id", id = schema.system_id_column),
        format!("s.{name} AS system_name", name = schema.system_name_column),
    ];
    let mut joins = Vec::new();

    if let Some(join) = schema.constellation_join {
        selects.push(format!("s.{fk} AS constellation_id", fk = join.fk_column));
        selects.push(format!(
            "c.{name} AS constellation_name",
            name = join.table_name_column
        ));
        joins.push(format!(
            "LEFT JOIN {table} c ON c.{id} = s.{fk}",
            table = join.table,
            id = join.table_id_column,
            fk = join.fk_column
        ));
    } else {
        selects.push("NULL AS constellation_id".to_string());
        selects.push("NULL AS constellation_name".to_string());
    }

    if let Some(join) = schema.region_join {
        selects.push(format!("s.{fk} AS region_id", fk = join.fk_column));
        selects.push(format!(
            "r.{name} AS region_name",
            name = join.table_name_column
        ));
        joins.push(format!(
            "LEFT JOIN {table} r ON r.{id} = s.{fk}",
            table = join.table,
            id = join.table_id_column,
            fk = join.fk_column
        ));
    } else {
        selects.push("NULL AS region_id".to_string());
        selects.push("NULL AS region_name".to_string());
    }

    if let Some(column) = schema.security_column {
        selects.push(format!("s.{column} AS security_status", column = column));
    } else {
        selects.push("NULL AS security_status".to_string());
    }

    if let Some(columns) = schema.position_columns {
        selects.push(format!("s.{x} AS position_x", x = columns.x));
        selects.push(format!("s.{y} AS position_y", y = columns.y));
        selects.push(format!("s.{z} AS position_z", z = columns.z));
    } else {
        selects.push("NULL AS position_x".to_string());
        selects.push("NULL AS position_y".to_string());
        selects.push("NULL AS position_z".to_string());
    }

    let mut sql = format!(
        "SELECT {selects} FROM {table} s",
        selects = selects.join(", "),
        table = schema.systems_table
    );

    for join in joins {
        sql.push(' ');
        sql.push_str(&join);
    }

    let mut stmt = connection.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_system)?;

    let mut systems = HashMap::new();
    for entry in rows {
        let system = entry?;
        systems.insert(system.id, system);
    }
    Ok(systems)
}

fn load_legacy_systems(
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
    let rows = stmt.query_map([], |row| {
        Ok(System {
            id: row.get(0)?,
            name: row.get(1)?,
            metadata: SystemMetadata::empty(),
            position: None,
        })
    })?;

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

fn row_to_system(row: &Row<'_>) -> rusqlite::Result<System> {
    let position = match (
        row.get::<_, Option<f64>>(7)?,
        row.get::<_, Option<f64>>(8)?,
        row.get::<_, Option<f64>>(9)?,
    ) {
        (Some(x), Some(y), Some(z)) => Some(SystemPosition { x, y, z }),
        _ => None,
    };

    Ok(System {
        id: row.get(0)?,
        name: row.get(1)?,
        metadata: SystemMetadata {
            constellation_id: row.get(2)?,
            constellation_name: row.get(3)?,
            region_id: row.get(4)?,
            region_name: row.get(5)?,
            security_status: row.get(6)?,
        },
        position,
    })
}

fn detect_static_schema(connection: &Connection) -> Result<Option<SchemaDefinition>> {
    let mut schema = SchemaVariant::StaticData.definition();

    if !table_exists(connection, schema.systems_table)?
        || !table_exists(connection, schema.jumps_table)?
    {
        return Ok(None);
    }

    if !table_has_columns(
        connection,
        schema.systems_table,
        &[schema.system_id_column, schema.system_name_column],
    )? {
        return Ok(None);
    }

    if !table_has_columns(
        connection,
        schema.jumps_table,
        &[schema.jump_from_column, schema.jump_to_column],
    )? {
        return Ok(None);
    }

    if let Some(join) = schema.constellation_join {
        if !table_has_columns(connection, schema.systems_table, &[join.fk_column])?
            || !table_exists(connection, join.table)?
            || !table_has_columns(
                connection,
                join.table,
                &[join.table_id_column, join.table_name_column],
            )?
        {
            schema.constellation_join = None;
        }
    }

    if let Some(join) = schema.region_join {
        if !table_has_columns(connection, schema.systems_table, &[join.fk_column])?
            || !table_exists(connection, join.table)?
            || !table_has_columns(
                connection,
                join.table,
                &[join.table_id_column, join.table_name_column],
            )?
        {
            schema.region_join = None;
        }
    }

    if let Some(column) = schema.security_column {
        if !table_has_columns(connection, schema.systems_table, &[column])? {
            schema.security_column = None;
        }
    }

    let position_candidates = [
        PositionColumns {
            x: "centerX",
            y: "centerY",
            z: "centerZ",
        },
        PositionColumns {
            x: "x",
            y: "y",
            z: "z",
        },
    ];

    schema.position_columns = None;
    for columns in position_candidates {
        if table_has_columns(
            connection,
            schema.systems_table,
            &[columns.x, columns.y, columns.z],
        )? {
            schema.position_columns = Some(columns);
            break;
        }
    }

    Ok(Some(schema))
}

fn detect_legacy_schema(connection: &Connection) -> Result<Option<SchemaDefinition>> {
    let schema = SchemaVariant::LegacyMap.definition();

    if !table_exists(connection, schema.systems_table)?
        || !table_exists(connection, schema.jumps_table)?
    {
        return Ok(None);
    }

    if !table_has_columns(
        connection,
        schema.systems_table,
        &[schema.system_id_column, schema.system_name_column],
    )? {
        return Ok(None);
    }

    if !table_has_columns(
        connection,
        schema.jumps_table,
        &[schema.jump_from_column, schema.jump_to_column],
    )? {
        return Ok(None);
    }

    Ok(Some(schema))
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
