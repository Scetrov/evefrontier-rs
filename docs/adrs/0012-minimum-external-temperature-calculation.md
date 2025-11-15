# ADR 0012: Minimum External Temperature Calculation for Solar Systems

## Status

Accepted

## Context

EVE Frontier's solar systems contain celestial bodies (planets and moons) at varying distances from their parent stars. Players need to identify systems with habitable zones or avoid extreme cold environments for operational safety and resource planning. The current implementation only tracks stellar temperature (`star_temperature`) but does not calculate the actual external temperature experienced at different orbital distances within a system.

To support route planning based on environmental temperature constraints, we need to:

1. Calculate the minimum external temperature in each solar system (representing the coldest location, typically at the outermost celestial body)
2. Allow players to filter routes to exclude systems that are too cold (below a minimum temperature threshold)
3. Provide scientifically reasonable temperature estimates based on stellar luminosity and orbital distance

### Temperature Calculation Requirements

The temperature calculation must consider:

- **Stellar luminosity** (`star_luminosity` in watts): The total energy output of the star
- **Orbital distance** (`orbitRadius` in meters for planets/moons): The distance from the star to the celestial body
- **Custom temperature model**: A parameterized curve that blends between extreme heat near the star and extreme cold in deep space
- **Stefan-Boltzmann equilibrium**: Physical calculation for blackbody radiation equilibrium

### Data Availability

The EVE Frontier `static_data.db` schema provides:

**SolarSystems table:**
- `star_luminosity` (REAL): Stellar luminosity in watts
- `star_temperature` (REAL): Stellar surface temperature in Kelvin
- Position data (`centerX`, `centerY`, `centerZ`)

**Planets table:**
- `solarSystemId` (INTEGER): Foreign key to SolarSystems
- `orbitRadius` (REAL): Orbital distance from star in meters
- `centerX`, `centerY`, `centerZ`: 3D coordinates
- `temperature` (REAL): Pre-calculated surface temperature (may be unreliable or model-specific)

**Moons table:**
- `solarSystemId` (INTEGER): Foreign key to SolarSystems
- `planetId` (INTEGER): Foreign key to Planets
- `orbitRadius` (REAL): Orbital distance from planet in meters
- Similar position and temperature fields

### Temperature Model Choice

We will implement two complementary temperature calculation methods:

1. **Custom Parameterized Model**: A smooth curve that transitions from `MaxKelvin` near the star to `MinKelvin` in deep space, controlled by parameters `K` (distance scale) and `B` (curve steepness). This model allows fine-tuning to match game balance or specific astrophysical assumptions.

2. **Stefan-Boltzmann Equilibrium**: The physically accurate blackbody equilibrium temperature:
   ```
   T = (L / (16π σ r²))^(1/4)
   ```
   Where:
   - `L` = stellar luminosity (watts)
   - `σ` = Stefan-Boltzmann constant (5.670374419 × 10⁻⁸ W⋅m⁻²⋅K⁻⁴)
   - `r` = distance from star (meters)

The custom model will be used as the primary calculation method for consistency with EVE Frontier's game mechanics, with Stefan-Boltzmann available for validation or alternative implementations.

## Decision

We will implement minimum external temperature calculation with the following design:

### 1. Temperature Calculation Module (`temperature.rs`)

Create a new module `crates/evefrontier-lib/src/temperature.rs` containing:

```rust
/// Configuration parameters for the custom temperature model
pub struct TemperatureModelParams {
    pub k: f64,              // Distance scale factor (controls transition point)
    pub b: f64,              // Curve steepness exponent
    pub min_kelvin: f64,     // Minimum temperature (deep space)
    pub max_kelvin: f64,     // Maximum temperature (star surface)
    pub kelvin_offset: f64,  // Optional offset for calibration
    pub kelvin_scale: f64,   // Optional scale for calibration
    pub map_to_kelvin: bool, // Whether to apply offset/scale
}

impl Default for TemperatureModelParams {
    fn default() -> Self {
        Self {
            k: 1.0,
            b: 4.0,
            min_kelvin: 2.7,      // Cosmic microwave background
            max_kelvin: 5778.0,   // Solar surface temperature
            kelvin_offset: 0.0,
            kelvin_scale: 1.0,
            map_to_kelvin: false,
        }
    }
}

/// Physical constants for temperature calculations
pub mod constants {
    /// Stefan-Boltzmann constant (W⋅m⁻²⋅K⁻⁴)
    pub const STEFAN_BOLTZMANN_SIGMA: f64 = 5.670374419e-8;
    
    /// Meters in one light-second
    pub const METERS_IN_LIGHT_SECOND: f64 = 299_792_458.0;
    
    /// Meters in one astronomical unit
    pub const METERS_IN_AU: f64 = 1.495978707e11;
}

/// Calculate external temperature using the custom parameterized model
pub fn compute_temperature_light_seconds(
    distance_light_seconds: f64,
    luminosity_watts: f64,
    params: &TemperatureModelParams,
) -> Result<f64>;

/// Calculate external temperature in meters (convenience wrapper)
pub fn compute_temperature_meters(
    distance_meters: f64,
    luminosity_watts: f64,
    params: &TemperatureModelParams,
) -> Result<f64>;

/// Calculate Stefan-Boltzmann equilibrium temperature
/// For a fast-rotating, zero-albedo sphere
pub fn compute_stefan_boltzmann_kelvin(
    distance_meters: f64,
    luminosity_watts: f64,
) -> Result<f64>;
```

### 2. System Minimum Temperature Calculation

During `load_starmap`, for each solar system:

1. Query all planets and moons in the system
2. Calculate the total orbital distance from the star for each celestial body:
   - For planets: `orbit_from_star = planet.orbitRadius`
   - For moons: `orbit_from_star = planet.orbitRadius + moon.orbitRadius` (approximation; assumes moon orbit around planet)
3. Identify the celestial body with the maximum orbital distance (furthest from star)
4. Calculate external temperature at that distance using `compute_temperature_meters`
5. Store result in `SystemMetadata.min_external_temp`

**Edge cases:**
- Systems with no planets/moons: Set `min_external_temp = None`
- Systems with missing `star_luminosity`: Set `min_external_temp = None`
- Invalid luminosity or distance (negative/zero): Set `min_external_temp = None` and log warning

### 3. Schema Extension

Update `SystemMetadata` in `db.rs`:

```rust
pub struct SystemMetadata {
    pub constellation_id: Option<i64>,
    pub constellation_name: Option<String>,
    pub region_id: Option<i64>,
    pub region_name: Option<String>,
    pub security_status: Option<f64>,
    pub star_temperature: Option<f64>,      // Stellar surface temperature
    pub min_external_temp: Option<f64>,     // Minimum external temperature in system
}
```

### 4. Routing Constraint

Update `PathConstraints` in `path.rs`:

```rust
pub struct PathConstraints {
    pub max_jump: Option<f64>,
    pub avoid_gates: bool,
    pub avoided_systems: HashSet<SystemId>,
    pub max_temperature: Option<f64>,      // Maximum stellar temperature (existing)
    pub min_temperature: Option<f64>,      // NEW: Minimum external temperature threshold
}
```

Filter logic in `allows()`:
- If `min_temperature` is set and system has `min_external_temp < min_temperature`, exclude the system
- Systems with `min_external_temp = None` are treated as unknown and **allowed** (fail-open for safety)

### 5. CLI Integration

Add to `RouteOptions` in `main.rs`:

```rust
#[derive(Debug, Clone, clap::Args)]
struct RouteOptions {
    // ... existing fields ...
    
    /// Exclude systems with minimum external temperature below this threshold (Kelvin)
    /// 
    /// The minimum external temperature is calculated at the outermost celestial
    /// body in each system. Use this to avoid extremely cold systems.
    /// 
    /// Example values:
    ///   --min-temp 200   (exclude systems colder than 200K)
    ///   --min-temp 273.15 (exclude systems below water freezing point)
    #[arg(long, value_name = "KELVIN")]
    min_temp: Option<f64>,
}
```

Map to `RoutingRequest`:
```rust
min_temperature: self.options.min_temp,
```

### 6. Output Format

When a route is filtered by `--min-temp`, provide helpful error messages:

```
Error: No route found from 'Nod' to 'D:2NAS'

Route constraints:
  Minimum external temperature: 200.00 K
  
System 'Brana' (excluded): minimum temperature 45.2 K is below threshold

Suggestions:
  - Relax the --min-temp constraint
  - Choose a destination in a warmer system
```

## Consequences

### Positive

1. **Scientifically grounded**: Uses established thermodynamic principles (Stefan-Boltzmann) with game-tuned parameters
2. **Operationally useful**: Players can avoid frozen wastelands or plan for cryogenic operations
3. **Extensible**: Temperature model parameters can be adjusted for balance or accuracy
4. **Fail-safe**: Unknown systems (no celestial data) are allowed, preventing over-filtering
5. **Backward compatible**: Existing routes without `--min-temp` are unaffected

### Negative

1. **Computational overhead**: Requires joining SolarSystems, Planets, and Moons tables during `load_starmap`
2. **Data dependency**: Requires complete and accurate `star_luminosity` and `orbitRadius` data in the dataset
3. **Approximation for moons**: Moon orbital distance calculation (`planet.orbitRadius + moon.orbitRadius`) is a simplification that may be inaccurate for eccentric orbits
4. **Increased memory footprint**: Each `System` now stores an additional `Option<f64>` for `min_external_temp`
5. **Maintenance burden**: Temperature model parameters may require tuning as game mechanics evolve

### Risks and Mitigations

**Risk: Inaccurate temperature calculations due to bad source data**
- *Mitigation*: Validate luminosity and distance values; reject negative/zero/infinite values; log warnings for suspicious data

**Risk: Performance degradation from additional database queries**
- *Mitigation*: Use batch queries with JOINs; consider caching results; profile load time before/after

**Risk: Confusion between `star_temperature` and `min_external_temp`**
- *Mitigation*: Clear documentation and naming; provide examples in `USAGE.md`; include units in CLI help text

## Implementation Plan

1. Create `temperature.rs` module with calculation functions and tests
2. Extend `db.rs` to load `star_luminosity` from SolarSystems table
3. Implement celestial body query and minimum temperature calculation during `load_starmap`
4. Update `SystemMetadata` and `PathConstraints` structures
5. Add filtering logic to pathfinding algorithms
6. Add `--min-temp` CLI option and wire it through to routing
7. Write unit tests for temperature calculations (known star + orbit → expected temp)
8. Write integration tests for route filtering with `--min-temp`
9. Update `USAGE.md` and `README.md` with examples

## References

- **Stefan-Boltzmann Law**: [Wikipedia - Stefan–Boltzmann law](https://en.wikipedia.org/wiki/Stefan%E2%80%93Boltzmann_law)
- **Planetary equilibrium temperature**: [Wikipedia - Planetary equilibrium temperature](https://en.wikipedia.org/wiki/Planetary_equilibrium_temperature)
- EVE Frontier `static_data.db` schema (observed from fixture database)
- Original C# temperature calculation code (provided by user)

## Notes

- This ADR supersedes the placeholder `temperature: None` in the current implementation
- The custom temperature model parameters (`k`, `b`, `min_kelvin`, `max_kelvin`) should be documented with recommended values or made configurable if needed for different dataset versions
- Future enhancement: Consider stellar spectral class and albedo for more accurate calculations
- Future enhancement: Pre-calculate minimum temperatures in a spatial index artifact (see ADR 0009) to avoid runtime overhead
