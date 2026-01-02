# ADR 0015: Fuel Cost and Heat Impact Calculations for Route Planning

## Status

Approved

## Context

EVE Frontier ships have varying physical characteristics that affect their operational costs during
spatial travel. When planning routes, players need to understand:

1. **Fuel consumption** — How much fuel each jump will cost based on ship mass, fuel quality, and
   distance
2. **Thermal impact** — How the cumulative heat exposure during spatial jumps affects the ship based
   on its specific heat capacity

Currently, the CLI and Lambda route outputs include environmental data (system temperature, planets,
moons) but lack ship-specific operational projections. To provide actionable route planning, we need
to integrate ship attribute data and calculate per-hop and total-route fuel costs and heat impact.

### Data Source

The `evefrontier_datasets` repository (https://github.com/Scetrov/evefrontier_datasets) includes a
`ship_data.csv` file in each release (starting from e6c4) containing ship attributes:

| Field                    | Type   | Description                                           |
| ------------------------ | ------ | ----------------------------------------------------- |
| `name`                   | String | Ship name (e.g., "Reflex", "Forager")                 |
| `base_mass_kg`           | f64    | Ship hull mass in kilograms (empty, no fuel/cargo)    |
| `specific_heat`          | f64    | Specific heat capacity (J/kg·K)                       |
| `fuel_capacity`          | f64    | Maximum fuel tank capacity (units)                    |
| `cargo_capacity`         | f64    | Maximum cargo hold capacity (m³)                      |
| `max_heat_tolerance`     | *removed*    | Per-ship tolerance is not provided by the canonical game CSV; heat warnings use canonical thresholds (HEAT_OVERHEATED/HEAT_CRITICAL) |
| `heat_dissipation_rate`  | *removed*    | Per-ship dissipation is not provided by the canonical game CSV; dissipation may be modelled using environment and future research |

### Mass Calculation

The fuel formula uses **total operational mass**, not just hull mass. Total mass is:

```text
total_mass_kg = base_mass_kg + fuel_mass_kg + cargo_mass_kg
```

Where:

- `base_mass_kg` = Ship hull mass (from CSV)
- `fuel_mass_kg` = Current fuel load × fuel density (assumed 1 kg per fuel unit)
- `cargo_mass_kg` = Current cargo mass (user-specified or assumed empty)

**Important:** Fuel consumption affects mass dynamically. As fuel is consumed, the ship becomes
lighter, reducing fuel cost for subsequent jumps. This creates a feedback loop where:

1. First jump: Uses mass with full fuel tank
2. Subsequent jumps: Uses reduced mass (fuel consumed)
3. Final jumps: Most fuel-efficient due to lowest mass

For route planning, we offer two calculation modes:

- **Static mode** (default): Assumes constant mass (simplest, slightly overestimates fuel)
- **Dynamic mode** (`--dynamic-mass`): Recalculates mass after each hop (more accurate)

### Fuel Consumption Formula

The fuel cost for a spatial jump follows a linear model derived from in-game mechanics:

```text
fuel_cost = (total_mass_kg / 10^5) × (fuel_quality / 100) × distance_ly
```

Where:

- `total_mass_kg` = Total operational mass (hull + fuel + cargo) in kilograms
- `fuel_quality` = Fuel quality rating (1-100, typically 10 for standard fuel)
- `distance_ly` = Jump distance in light-years

**Example calculation (Reflex ship, static mode):**

Assuming: base_mass = 10,000,000 kg, fuel_load = 1,750 units (1,750 kg), cargo = 633,006 kg
→ total_mass = 12,383,006 kg

| Distance (ly) | Total Mass (kg) | Fuel Quality | Fuel Cost |
| ------------- | --------------- | ------------ | --------- |
| 18.95         | 12,383,006      | 10           | 23.47     |
| 38.26         | 12,383,006      | 10           | 47.38     |
| 23.09         | 12,383,006      | 10           | 28.59     |

**Example calculation (Reflex ship, dynamic mode):**

Starting: total_mass = 12,383,006 kg, fuel_load = 1,750 units

| Hop | Distance (ly) | Mass Before (kg) | Fuel Cost | Fuel After | Mass After (kg) |
| --- | ------------- | ---------------- | --------- | ---------- | --------------- |
| 1   | 18.95         | 12,383,006       | 23.47     | 1,726.53   | 12,359,536      |
| 2   | 38.26         | 12,359,536       | 47.29     | 1,679.24   | 12,312,246      |
| 3   | 23.09         | 12,312,246       | 28.43     | 1,650.81   | 12,283,816      |

_Note: Dynamic mode total = 99.19 units vs static mode total = 99.44 units (0.25% savings)_

### Heat Impact Formula

Heat generation during spatial jumps depends on the ship's mass and the jump distance. The heat
generation model accounts for both current operational mass (including fuel and cargo) and the
baseline hull mass:

```text
heat_gen = (3 × current_total_mass_kg × distance_ly) / (C × hull_mass_only_kg)
```

Where:

- `current_total_mass_kg` = Total operational mass (hull + fuel + cargo) in kilograms
- `distance_ly` = Jump distance in light-years
- `hull_mass_only_kg` = Ship hull mass (empty, no fuel or cargo) in kilograms
- `C` = Calibration constant (default: 1.0, tuned for EVE Frontier game balance)
- `3` = Proportionality constant calibrated for heat generation rate

**Rationale:**

The formula captures the physics of spatial jump heat generation:

1. **Mass dependency (linear)**: Heavier ships generate more heat during jumps
2. **Distance dependency (linear)**: Longer jumps expose ships to thermal stress longer
3. **Hull mass normalization**: The ratio accounts for efficiency — lighter-hulled ships generate
   less heat relative to their operational mass
4. **Dynamic loading effects**: As fuel is consumed during route, mass decreases, reducing heat
   generation for subsequent jumps (when using dynamic mass mode)

**Example calculation (Reflex ship, static mode):**

Assuming: hull_mass = 10,000,000 kg, total_mass = 12,383,006 kg, C = 1.0

| Distance (ly) | Total Mass (kg) | Heat Generated |
| ------------- | --------------- | -------------- |
| 18.95         | 12,383,006      | 88.96          |
| 38.26         | 12,383,006      | 179.73         |
| 23.09         | 12,383,006      | 108.39         |

**Dynamic mode impact:**

In dynamic mass mode, heat generation decreases with each hop as fuel is consumed:

| Hop | Distance (ly) | Mass (kg)   | Heat Gen | Total Heat |
| --- | ------------- | ----------- | -------- | ---------- |
| 1   | 18.95         | 12,383,006  | 88.96    | 88.96      |
| 2   | 38.26         | 12,359,536  | 89.53    | 178.49     |
| 3   | 23.09         | 12,312,246  | 88.75    | 267.24     |

_Note: Heat accumulates across the entire route; total route heat differs in dynamic vs static modes_

### Heat Implementation Details

The library implements heat projections with the following behaviour:

- **Function**: `calculate_jump_heat(total_mass_kg, distance_ly, hull_mass_kg, calibration)` — a pure function returning per-hop heat as f64.
- **Calibration**: default `calibration_constant = 1.0` (configurable via `HeatConfig`).
- **Mass modes**: mirrors fuel (`static` default, `dynamic_mass` recalculates mass after each hop).
- **Types**: `HeatProjection` (per-step) and `HeatSummary` (route-level) are serialisable and included in CLI and Lambda responses.
- **Warnings**: cumulative heat > 75% yields a warning, > 100% yields an error-level warning.


**Heat accumulation and dissipation:**

Ships accumulate heat during spatial jumps. The exact dissipation rate and maximum heat tolerance
are ship-specific attributes defined in the ship data CSV. Heat that exceeds a ship's tolerance
may result in damage or operational penalties.

### Use Cases

1. **Route fuel estimation** — Show total fuel required for a route and per-hop breakdown
2. **Fuel capacity warnings** — Alert when total route fuel exceeds ship capacity
3. **Refueling stop suggestions** — Identify systems where refueling may be required
4. **Route heat generation** — Show cumulative heat generated by the route and per-hop breakdown
5. **Heat tolerance warnings** — Alert when route heat exceeds ship's heat tolerance
6. **Heat dissipation planning** — Suggest waypoints for heat dissipation during long routes
7. **Thermal risk assessment** — Highlight high-heat routes that may damage the ship
8. **Ship comparison** — Compare fuel efficiency, heat generation, and thermal resilience across ships
9. **Route optimization** — Select fuel/heat-efficient routes based on ship characteristics

## Decision

We will implement ship attribute loading and fuel/heat calculations with the following design:

### 1. Ship Data Module (`ship.rs`)

Create a new module `crates/evefrontier-lib/src/ship.rs` containing:

```rust
use serde::{Deserialize, Serialize};

/// Mass of one fuel unit in kilograms
pub const FUEL_MASS_PER_UNIT_KG: f64 = 1.0;

/// Ship attributes loaded from ship_data.csv
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipAttributes {
    /// Ship name (e.g., "Reflex")
    pub name: String,
    /// Ship hull mass in kilograms (empty, no fuel/cargo)
    pub base_mass_kg: f64,
    /// Specific heat capacity (J/kg·K) — used for heat tolerance calculations
    pub specific_heat: f64,
    /// Maximum fuel tank capacity (units)
    pub fuel_capacity: f64,
    /// Maximum cargo hold capacity (m³)
    pub cargo_capacity: f64,
    // NOTE: Per-ship tolerance and dissipation fields are not present in the canonical
    // dataset and are intentionally omitted from the `ShipAttributes` model. Heat warnings
    // are produced using canonical thresholds (HEAT_OVERHEATED, HEAT_CRITICAL) described in
    // the research documentation.
}

/// Current ship loadout for mass calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipLoadout {
    /// Current fuel load (units)
    pub fuel_load: f64,
    /// Current cargo mass (kg)
    pub cargo_mass_kg: f64,
}

impl ShipLoadout {
    /// Create a loadout with full fuel and empty cargo
    pub fn full_fuel(ship: &ShipAttributes) -> Self {
        Self {
            fuel_load: ship.fuel_capacity,
            cargo_mass_kg: 0.0,
        }
    }
    
    /// Calculate total mass including hull, fuel, and cargo
    pub fn total_mass_kg(&self, ship: &ShipAttributes) -> f64 {
        ship.base_mass_kg 
            + (self.fuel_load * FUEL_MASS_PER_UNIT_KG)
            + self.cargo_mass_kg
    }
}

/// Fuel quality configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuelConfig {
    /// Fuel quality rating (1-100)
    pub quality: f64,
    /// Whether to recalculate mass after each hop
    pub dynamic_mass: bool,
}

impl Default for FuelConfig {
    fn default() -> Self {
        Self { 
            quality: 10.0,      // Standard fuel quality
            dynamic_mass: false, // Static mass by default
        }
    }
}

/// Calculate fuel cost for a single spatial jump
///
/// Formula: (total_mass_kg / 10^5) × (fuel_quality / 100) × distance_ly
pub fn calculate_jump_fuel_cost(
    total_mass_kg: f64,
    distance_ly: f64,
    fuel_config: &FuelConfig,
) -> f64 {
    let mass_factor = total_mass_kg / 100_000.0;
    let quality_factor = fuel_config.quality / 100.0;
    mass_factor * quality_factor * distance_ly
}

/// Calculate fuel costs for an entire route
///
/// In dynamic mode, mass is recalculated after each hop as fuel is consumed.
/// Returns a vector of (hop_cost, cumulative_cost, remaining_fuel) tuples.
pub fn calculate_route_fuel(
    ship: &ShipAttributes,
    loadout: &ShipLoadout,
    distances_ly: &[f64],
    fuel_config: &FuelConfig,
) -> Vec<(f64, f64, f64)> {
    let mut results = Vec::with_capacity(distances_ly.len());
    let mut cumulative = 0.0;
    let mut current_loadout = *loadout;
    
    for &distance in distances_ly {
        let mass = current_loadout.total_mass_kg(ship);
        let hop_cost = calculate_jump_fuel_cost(mass, distance, fuel_config);
        cumulative += hop_cost;
        
        if fuel_config.dynamic_mass {
            current_loadout.fuel_load -= hop_cost;
        }
        
        let remaining = current_loadout.fuel_load - if fuel_config.dynamic_mass { 0.0 } else { cumulative };
        results.push((hop_cost, cumulative, remaining.max(0.0)));
    }
    
    results
}

/// Ship catalog loaded from CSV
#[derive(Debug, Clone, Default)]
pub struct ShipCatalog {
    ships: std::collections::HashMap<String, ShipAttributes>,
}

impl ShipCatalog {
    /// Load ship catalog from CSV content
    pub fn from_csv(csv_content: &str) -> Result<Self, crate::error::Error>;
    
    /// Get ship by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&ShipAttributes>;
    
    /// List all available ship names
    pub fn ship_names(&self) -> Vec<&str>;
}
```

### 2. CSV Download Extension

Extend the GitHub downloader (`github.rs`) to support downloading the `ship_data.csv` asset:

```rust
/// Download ship data CSV from the dataset release
pub fn download_ship_data(target_path: &Path, release: &DatasetRelease) -> Result<()>;

/// Ensure ship data is available, downloading if necessary
pub fn ensure_ship_data(cache_dir: Option<&Path>) -> Result<PathBuf>;
```

**Caching behavior:**

- Ship data is cached alongside the database in the `evefrontier_datasets/` cache directory
- Cache key includes the release tag (e.g., `e6c4/ship_data.csv`)
- CSV is validated after download (header check, row count)

### 3. Fuel Projection in Route Output

Extend `RouteStep` and `RouteSummary` to include fuel projections:

```rust
/// Fuel projection for a single route step
#[derive(Debug, Clone, Serialize)]
pub struct FuelProjection {
    /// Fuel cost for this hop (units)
    pub hop_cost: f64,
    /// Cumulative fuel consumed from start (units)
    pub cumulative: f64,
    /// Remaining fuel capacity after this hop (units)
    pub remaining: f64,
    /// Warning if fuel would be exhausted
    pub warning: Option<String>,
}

/// Extended route step with fuel data
pub struct RouteStep {
    // ... existing fields ...
    
    /// Fuel projection for this step (if ship specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelProjection>,
}

/// Extended route summary with fuel totals
pub struct RouteSummary {
    // ... existing fields ...
    
    /// Total fuel required for the route
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_fuel: Option<f64>,
    
    /// Ship used for fuel calculations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_name: Option<String>,
    
    /// Warning if route exceeds fuel capacity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_warning: Option<String>,
}
```

### 4. Heat Impact in Route Output

Heat generation calculations provide per-hop and cumulative heat projections based on the formula:

```rust
/// Heat generation for a single route step
#[derive(Debug, Clone, Serialize)]
pub struct HeatProjection {
    /// Heat generated during this hop (units)
    pub hop_heat: f64,
    /// Cumulative heat generated from start (units)
    pub cumulative_heat: f64,
    /// Current ship heat level (units)
    pub current_heat: f64,
    /// Maximum heat tolerance for this ship (units)
    pub max_heat: f64,
    /// Warning if heat exceeds safe operating range
    pub warning: Option<String>,
}

/// Extended route step with heat data
pub struct RouteStep {
    // ... existing fields ...
    
    /// Heat generation for this step (if ship specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat: Option<HeatProjection>,
}

/// Extended route summary with heat totals
pub struct RouteSummary {
    // ... existing fields ...
    
    /// Total heat generated for the route
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_heat: Option<f64>,
    
    /// Heat dissipation rate (per time unit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_dissipation_rate: Option<f64>,
    
    /// Warning if route generates excessive heat
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_warning: Option<String>,
}
```

**Heat generation formula implementation:**

```rust
/// Calculate heat generation for a single spatial jump
pub fn calculate_jump_heat(
    current_total_mass_kg: f64,
    hull_mass_only_kg: f64,
    distance_ly: f64,
    calibration_constant: f64, // C (default: 1.0)
) -> f64 {
    (3.0 * current_total_mass_kg * distance_ly) / (calibration_constant * hull_mass_only_kg)
}

/// Calculate heat generation for an entire route
pub fn calculate_route_heat(
    ship: &ShipAttributes,
    loadout: &ShipLoadout,
    distances_ly: &[f64],
    dynamic_mass: bool,
    calibration_constant: f64,
) -> Vec<(f64, f64)> {
    // Returns vector of (hop_heat, cumulative_heat) tuples
    let mut results = Vec::with_capacity(distances_ly.len());
    let mut cumulative = 0.0;
    let mut current_loadout = *loadout;
    
    for &distance in distances_ly {
        let current_mass = current_loadout.total_mass_kg(ship);
        let hop_heat = calculate_jump_heat(
            current_mass,
            ship.base_mass_kg,
            distance,
            calibration_constant,
        );
        cumulative += hop_heat;
        
        if dynamic_mass {
            // Assume heat impacts fuel burn or mass changes
            // (This may require additional mechanics definition)
            current_loadout.fuel_load -= (hop_heat * 0.1); // Example: heat affects fuel load
        }
        
        results.push((hop_heat, cumulative));
    }
    
    results
}
```

### 5. CLI Integration

Add ship selection options to the CLI:

```rust
#[derive(Debug, Clone, clap::Args)]
struct RouteOptions {
    // ... existing fields ...

    /// Ship name for fuel calculations (e.g., "Reflex")
    ///
    /// When specified, the route output will include per-hop fuel costs
    /// and total fuel required. Use --list-ships to see available ships.
    #[arg(long, value_name = "SHIP")]
    ship: Option<String>,

    /// Fuel quality rating (1-100, default: 10)
    ///
    /// Higher quality fuel is more efficient. Standard fuel is quality 10.
    #[arg(long, value_name = "QUALITY", default_value = "10")]
    fuel_quality: f64,

    /// Cargo mass in kilograms (default: 0)
    ///
    /// Include cargo weight in fuel calculations. Total ship mass =
    /// hull mass + fuel mass + cargo mass.
    #[arg(long, value_name = "KG", default_value = "0")]
    cargo_mass: f64,

    /// Initial fuel load as percentage of capacity (default: 100)
    ///
    /// Start route with partial fuel tank. Use 100 for full tank.
    #[arg(long, value_name = "PERCENT", default_value = "100")]
    fuel_load: f64,

    /// Recalculate mass after each hop as fuel is consumed
    ///
    /// Enables more accurate fuel estimation by accounting for the
    /// decreasing ship mass as fuel is burned. Slightly reduces total
    /// fuel consumption estimates.
    #[arg(long)]
    dynamic_mass: bool,

    /// List available ships and exit
    #[arg(long)]
    list_ships: bool,
}
```

### 6. Status Line Enhancement

Update the enhanced output format to include fuel information in the status line:

**Current format:**

```
  3: Brana (30123456789; min 0.32K) [4 planets, 12 moons]
      ↳ 38.26 ly via jump
```

**Enhanced format with fuel:**

```
  3: Brana (30123456789; min 0.32K) [4 planets, 12 moons]
      ↳ 38.26 ly via jump | fuel: 47.38 (remaining: 1,655.24)
```

**JSON output extension:**

```json
{
  "steps": [
    {
      "index": 3,
      "id": 30123456789,
      "name": "Brana",
      "distance": 38.26,
      "method": "jump",
      "min_external_temp": 0.32,
      "planet_count": 4,
      "moon_count": 12,
      "fuel": {
        "hop_cost": 47.38,
        "cumulative": 99.44,
        "remaining": 1655.24
      }
    }
  ],
  "total_fuel": 99.44,
  "ship_name": "Reflex",
  "fuel_warning": null
}
```

### 7. Lambda API Extension

Extend Lambda request/response schemas to support ship-specific calculations:

**Request extension:**

```json
{
  "from": "Nod",
  "to": "Brana",
  "ship": "Reflex",
  "fuel_quality": 10
}
```

**Response extension:**

```json
{
  "route": { /* ... existing fields ... */ },
  "fuel_summary": {
    "ship": "Reflex",
    "total_cost": 99.44,
    "capacity": 1750.0,
    "remaining": 1650.56,
    "warnings": []
  }
}
```

## Rationale

### Why integrate ship data?

1. **Actionable information** — Distances alone aren't sufficient for route planning; fuel costs
   determine feasibility
2. **Player experience** — Knowing fuel requirements before departure prevents stranding
3. **Route optimization** — Enables future features like fuel-efficient route algorithms

### Why use CSV instead of database?

1. **Simplicity** — Ship data is small (~10-50 entries), CSV is easy to maintain
2. **Separation** — Ship stats may change more frequently than static system data
3. **Extensibility** — Additional ship attributes can be added without schema migrations

### Why fuel-only in first iteration?

1. **Validation** — Fuel formula is derived from confirmed in-game behavior
2. **Heat complexity** — Heat mechanics require more research and may vary by game version
3. **Incremental delivery** — Fuel projection provides immediate value

### Alternatives Considered

1. **Ship data in SQLite** — Rejected due to added complexity for small dataset
2. **Hardcoded ship data** — Rejected due to maintenance burden and version skew
3. **Heat impact in first iteration** — Deferred pending mechanic validation
4. **Per-hop refueling suggestions** — Deferred to future enhancement

## Consequences

### Positive

1. **Practical utility** — Players can plan routes knowing exact fuel requirements
2. **Warning system** — Prevents embarking on routes that exceed fuel capacity
3. **Ship comparison** — Enables informed ship selection for different routes
4. **Extensible foundation** — Ship catalog can expand to include more attributes
5. **Backward compatible** — Ship parameter is optional; existing behavior unchanged

### Negative

1. **Additional download** — Ship data CSV requires separate download/cache management
2. **Memory overhead** — Ship catalog adds ~1KB per ship to runtime memory
3. **API complexity** — Lambda endpoints gain additional optional parameters
4. **Output verbosity** — Enhanced format becomes more complex with fuel data
5. **Formula dependency** — Fuel formula may need updates if game mechanics change

### Risks and Mitigations

**Risk: Fuel formula inaccuracy**

- _Mitigation_: Document formula source; provide `--fuel-quality` override; validate against
  community data

**Risk: Ship data staleness**

- _Mitigation_: Include release tag in cache key; warn if ship data older than database

**Risk: CSV parsing errors**

- _Mitigation_: Validate CSV structure; provide clear error messages; fall back gracefully

**Risk: User confusion between fuel cost and distance**

- _Mitigation_: Clear labeling in output; include units; document in `USAGE.md`

## Implementation Plan

1. **Phase 1: Ship Data Foundation**
   - Create `ship.rs` module with `ShipAttributes` and `ShipCatalog`
   - Implement CSV parsing with validation
   - Add unit tests for parsing and fuel calculation
   - Extend downloader to fetch `ship_data.csv`

2. **Phase 2: CLI Integration**
   - Add `--ship` and `--fuel-quality` options to `route` subcommand
   - Add `--list-ships` convenience option
   - Update output formatters to include fuel projection
   - Add integration tests with fixture ship data

3. **Phase 3: Lambda Integration**
   - Extend request/response schemas
   - Bundle ship data CSV with Lambda deployment
   - Add fuel projection to route responses
   - Update API documentation

4. **Phase 4: Documentation & Polish**
   - Update `USAGE.md` with fuel projection examples
   - Update `README.md` with ship data information
   - Add ship data to test fixtures
   - Performance testing with full ship catalog

5. **Future: Heat Impact (Separate ADR)**
   - Research and validate heat mechanics
   - Design thermal projection model
   - Implement and test heat calculations
   - Document in separate ADR

## References

- EVE Frontier fuel consumption formula (community-derived)
- `evefrontier_datasets` repository: https://github.com/Scetrov/evefrontier_datasets
- ADR 0012: System Temperature Calculation (for thermal context)
- ADR 0005: CLI Design (for output format patterns)

## Test Cases

### Fuel Calculation Validation

Using Reflex ship attributes (mass_kg = 12,383,006, fuel_quality = 10):

| Test Case           | Distance (ly) | Expected Fuel Cost |
| ------------------- | ------------- | ------------------ |
| Short jump          | 18.95         | 23.47              |
| Medium jump         | 38.26         | 47.38              |
| Long jump           | 23.09         | 28.59              |
| Zero distance       | 0.0           | 0.0                |
| Gate (no fuel)      | N/A           | 0.0                |

### Edge Cases

- Ship not found → Error with suggestion (fuzzy match)
- Missing ship data CSV → Warning, calculations skipped
- Fuel quality out of range → Clamp to 1-100 with warning
- Route with only gates → Total fuel = 0.0

---

_Author: Generated based on user requirements_  
_Date: 2025-12-30_
