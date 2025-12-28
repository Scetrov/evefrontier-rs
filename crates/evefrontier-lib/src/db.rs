use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, Row};
use tracing::{debug, warn};

use crate::error::{Error, Result};

/// Numeric identifier for a solar system.
pub type SystemId = i64;

/// Conversion factor from meters to light-years.
/// 1 light-year ≈ 9.4607304725808 × 10^15 meters
const METERS_TO_LIGHT_YEARS: f64 = 1.0 / 9.4607304725808e15;

/// Cartesian coordinates for a solar system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl SystemPosition {
    /// Construct a position from coordinates, rejecting non-finite values so
    /// downstream graph builders can rely on well-formed distances.
    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Self { x, y, z })
        } else {
            None
        }
    }

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
    pub star_temperature: Option<f64>,
    pub star_luminosity: Option<f64>,
    pub min_external_temp: Option<f64>,
    pub planet_count: Option<u32>,
    pub moon_count: Option<u32>,
}

impl SystemMetadata {
    fn empty() -> Self {
        Self {
            constellation_id: None,
            constellation_name: None,
            region_id: None,
            region_name: None,
            security_status: None,
            star_temperature: None,
            star_luminosity: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
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

    /// Find system names similar to the query using fuzzy matching.
    ///
    /// Returns up to `limit` system names sorted by similarity (most similar first).
    /// Uses Jaro-Winkler similarity with a minimum threshold of 0.7.
    pub fn fuzzy_system_matches(&self, query: &str, limit: usize) -> Vec<String> {
        use strsim::jaro_winkler;

        const MIN_SIMILARITY: f64 = 0.7;

        let mut candidates: Vec<(f64, String)> = self
            .name_to_id
            .keys()
            .filter_map(|name| {
                let similarity = jaro_winkler(query, name);
                if similarity >= MIN_SIMILARITY {
                    Some((similarity, name.clone()))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        candidates
            .into_iter()
            .take(limit)
            .map(|(_, name)| name)
            .collect()
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
                    fk_column: "constellationId",
                    table: "Constellations",
                    table_id_column: "constellationId",
                    table_name_column: "name",
                }),
                region_join: Some(MetadataJoin {
                    fk_column: "regionId",
                    table: "Regions",
                    table_id_column: "regionId",
                    table_name_column: "name",
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
    load_starmap_from_connection(&connection)
}

/// Load systems and jumps from an already-opened database connection.
///
/// This is useful for loading from in-memory databases (e.g., Lambda with
/// bundled data) or when the connection is managed externally.
///
/// # Example
///
/// ```no_run
/// use rusqlite::Connection;
/// use evefrontier_lib::db::load_starmap_from_connection;
///
/// let conn = Connection::open_in_memory().unwrap();
/// // ... deserialize database bytes into conn ...
/// let starmap = load_starmap_from_connection(&conn).unwrap();
/// ```
pub fn load_starmap_from_connection(connection: &Connection) -> Result<Starmap> {
    let schema = detect_schema(connection)?;
    debug!(schema = %schema.variant, "loading starmap from connection");

    let mut systems = load_systems(connection, &schema)?;
    let adjacency = Arc::new(load_adjacency(connection, &schema, &systems)?);

    // Calculate minimum external temperatures for systems (if celestial data available)
    calculate_min_external_temps(connection, &mut systems)?;

    // Load planet and moon counts for systems (if tables exist)
    load_celestial_counts(connection, &mut systems)?;

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

    // Add star_temperature and star_luminosity if columns exist
    selects.push("s.star_temperature AS star_temperature".to_string());
    selects.push("s.star_luminosity AS star_luminosity".to_string());

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
    let mut invalid_system_ids: HashSet<SystemId> = HashSet::new();
    for row in rows {
        let (from, to): (SystemId, SystemId) = row?;
        // Skip edges referencing systems not in the dataset (may occur due to schema
        // mismatches or incomplete data exports)
        if !systems.contains_key(&from) || !systems.contains_key(&to) {
            skipped_edges += 1;
            // Collect a few examples for troubleshooting (limit to 5 to avoid excessive logging)
            if invalid_system_ids.len() < 5 {
                if !systems.contains_key(&from) {
                    invalid_system_ids.insert(from);
                }
                if !systems.contains_key(&to) {
                    invalid_system_ids.insert(to);
                }
            }
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
            invalid_system_ids = ?invalid_system_ids,
            "ignored jump edges referencing unknown systems",
        );
    }

    Ok(adjacency)
}

/// Calculate minimum external temperatures for all systems.
///
/// For each system, find the outermost celestial body (planet or moon) and
/// calculate the external temperature at that orbital distance using the
/// custom temperature model.
fn calculate_min_external_temps(
    connection: &Connection,
    systems: &mut HashMap<SystemId, System>,
) -> Result<()> {
    use crate::temperature::{
        compute_temperature_light_seconds, constants::METERS_IN_LIGHT_SECOND,
        TemperatureModelParams,
    };

    // Check if Planets and Moons tables exist
    if !table_exists(connection, "Planets")? {
        debug!("Planets table not found; skipping minimum temperature calculation");
        return Ok(());
    }

    let params = TemperatureModelParams::default();

    // Query all planets with their 3D coordinates
    let planet_query = "
        SELECT solarSystemId, centerX, centerY, centerZ
        FROM Planets
        WHERE centerX IS NOT NULL AND centerY IS NOT NULL AND centerZ IS NOT NULL
    ";

    let mut planet_stmt = connection.prepare(planet_query)?;
    let mut planet_coords: HashMap<SystemId, Vec<(f64, f64, f64)>> = HashMap::new();

    let planet_rows = planet_stmt.query_map([], |row| {
        Ok((
            row.get::<_, SystemId>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    for row in planet_rows {
        let (system_id, x, y, z) = row?;
        planet_coords.entry(system_id).or_default().push((x, y, z));
    }

    // Query moons if table exists
    let mut moon_coords: HashMap<SystemId, Vec<(f64, f64, f64)>> = HashMap::new();
    if table_exists(connection, "Moons")? {
        let moon_query = "
            SELECT solarSystemId, centerX, centerY, centerZ
            FROM Moons
            WHERE centerX IS NOT NULL AND centerY IS NOT NULL AND centerZ IS NOT NULL
        ";

        let mut moon_stmt = connection.prepare(moon_query)?;

        let moon_rows = moon_stmt.query_map([], |row| {
            Ok((
                row.get::<_, SystemId>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })?;

        for row in moon_rows {
            let (system_id, x, y, z) = row?;
            moon_coords.entry(system_id).or_default().push((x, y, z));
        }
    }

    // For each system, calculate minimum external temperature
    for (system_id, system) in systems.iter_mut() {
        let Some(luminosity) = system.metadata.star_luminosity else {
            continue; // Skip systems without luminosity data
        };

        // Zero or negative luminosity is valid for special stellar objects (e.g., black holes)
        // where temperature calculations don't apply. Skip silently.
        if luminosity <= 0.0 {
            continue;
        }

        // Find the maximum Euclidean distance from star (at origin) to any celestial
        let mut max_distance_meters = 0.0f64;

        // Check planets
        if let Some(planets) = planet_coords.get(system_id) {
            for (x, y, z) in planets {
                let dist = (x * x + y * y + z * z).sqrt();
                max_distance_meters = max_distance_meters.max(dist);
            }
        }

        // Check moons
        if let Some(moons) = moon_coords.get(system_id) {
            for (x, y, z) in moons {
                let dist = (x * x + y * y + z * z).sqrt();
                max_distance_meters = max_distance_meters.max(dist);
            }
        }

        if max_distance_meters > 0.0 {
            // Convert meters to light-seconds
            let max_distance_ls = max_distance_meters / METERS_IN_LIGHT_SECOND;

            match compute_temperature_light_seconds(max_distance_ls, luminosity, &params) {
                Ok(temp) => {
                    system.metadata.min_external_temp = Some(temp);
                }
                Err(e) => {
                    warn!(
                        system_id,
                        system_name = %system.name,
                        max_distance_meters,
                        max_distance_ls,
                        luminosity,
                        error = %e,
                        "failed to calculate minimum external temperature"
                    );
                }
            }
        }
    }

    Ok(())
}

/// Load planet and moon counts for each system.
///
/// This function queries the Planets and Moons tables (if they exist) and populates
/// the `planet_count` and `moon_count` fields in `SystemMetadata`.
fn load_celestial_counts(
    connection: &Connection,
    systems: &mut HashMap<SystemId, System>,
) -> Result<()> {
    // Check if Planets table exists
    if table_exists(connection, "Planets")? {
        let sql = "SELECT solarSystemId, COUNT(*) as cnt FROM Planets GROUP BY solarSystemId";
        let mut stmt = connection.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, SystemId>(0)?, row.get::<_, u32>(1)?))
        })?;

        for row in rows {
            let (system_id, count) = row?;
            if let Some(system) = systems.get_mut(&system_id) {
                system.metadata.planet_count = Some(count);
            }
        }
    }

    // Check if Moons table exists
    if table_exists(connection, "Moons")? {
        let sql = "SELECT solarSystemId, COUNT(*) as cnt FROM Moons GROUP BY solarSystemId";
        let mut stmt = connection.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, SystemId>(0)?, row.get::<_, u32>(1)?))
        })?;

        for row in rows {
            let (system_id, count) = row?;
            if let Some(system) = systems.get_mut(&system_id) {
                system.metadata.moon_count = Some(count);
            }
        }
    }

    Ok(())
}

fn row_to_system(row: &Row<'_>) -> rusqlite::Result<System> {
    // Use named column aliases produced by the SELECT in `load_static_systems`
    const COL_ID: &str = "system_id";
    const COL_NAME: &str = "system_name";
    const COL_CONSTELLATION_ID: &str = "constellation_id";
    const COL_CONSTELLATION_NAME: &str = "constellation_name";
    const COL_REGION_ID: &str = "region_id";
    const COL_REGION_NAME: &str = "region_name";
    const COL_SECURITY_STATUS: &str = "security_status";
    const COL_POSITION_X: &str = "position_x";
    const COL_POSITION_Y: &str = "position_y";
    const COL_POSITION_Z: &str = "position_z";
    const COL_STAR_TEMPERATURE: &str = "star_temperature";
    const COL_STAR_LUMINOSITY: &str = "star_luminosity";

    let position = match (
        row.get::<_, Option<f64>>(COL_POSITION_X)?,
        row.get::<_, Option<f64>>(COL_POSITION_Y)?,
        row.get::<_, Option<f64>>(COL_POSITION_Z)?,
    ) {
        (Some(x), Some(y), Some(z)) => {
            // Convert from meters (database storage) to light-years (routing calculations)
            SystemPosition::new(
                x * METERS_TO_LIGHT_YEARS,
                y * METERS_TO_LIGHT_YEARS,
                z * METERS_TO_LIGHT_YEARS,
            )
        }
        _ => None,
    };

    Ok(System {
        id: row.get::<_, SystemId>(COL_ID)?,
        name: row.get::<_, String>(COL_NAME)?,
        metadata: SystemMetadata {
            constellation_id: row.get::<_, Option<i64>>(COL_CONSTELLATION_ID)?,
            constellation_name: row.get::<_, Option<String>>(COL_CONSTELLATION_NAME)?,
            region_id: row.get::<_, Option<i64>>(COL_REGION_ID)?,
            region_name: row.get::<_, Option<String>>(COL_REGION_NAME)?,
            security_status: row.get::<_, Option<f64>>(COL_SECURITY_STATUS)?,
            star_temperature: row
                .get::<_, Option<f64>>(COL_STAR_TEMPERATURE)
                .ok()
                .flatten(),
            star_luminosity: row
                .get::<_, Option<f64>>(COL_STAR_LUMINOSITY)
                .ok()
                .flatten(),
            min_external_temp: None, // Calculated in a separate pass
            planet_count: None,      // Loaded in a separate pass
            moon_count: None,        // Loaded in a separate pass
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
    // Validate that table name contains only alphanumeric and underscores
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid table name: '{}'. Only alphanumeric and underscores are allowed.",
            table
        ))
        .into());
    }
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
