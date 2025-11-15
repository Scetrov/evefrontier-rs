# ADR 0012: System Temperature Calculation and Spatial Jump Constraints

## Status

Accepted (Revised 2025-11-15)

## Context

EVE Frontier's solar systems have varying stellar temperatures that affect ship operations during
spatial jumps. When making spatial jumps (warp without using stargates), ships are exposed to the
system's stellar environment and can overheat if the star temperature is too high. Gate jumps use
protective infrastructure and are unaffected by temperature.

To support realistic route planning based on environmental constraints, we need to:

1. Calculate the minimum external temperature in each solar system (representing the coldest
   location at the outermost celestial body) for informational purposes
2. Allow players to filter spatial jump routes based on maximum tolerable star temperature
3. **Gate jumps must ignore temperature constraints** - only spatial jumps are affected

### Revised Understanding (2025-11-15)

Initial implementation incorrectly included a `--min-temp` constraint. This was physically
nonsensical because:

- There is no game mechanic reason to avoid cold systems
- Ships have life support and thermal management for cold environments
- Only **hot systems prevent spatial jumps** due to thermal overload risk

The correct constraint model is:

- `--max-temp <KELVIN>`: Maximum star temperature for spatial jumps (default: unlimited)
- Constraint **only applies to EdgeKind::Spatial edges**, not EdgeKind::Gate
- Systems with `star_temperature > max_temp` cannot be reached via spatial jumps
- Gate jumps completely bypass temperature checks

### Temperature Calculation Requirements

The temperature calculation must consider:

- **Stellar luminosity** (`star_luminosity` in watts): The total energy output of the star
- **Orbital distance** (`orbitRadius` in meters for planets/moons): The distance from the star to
  the celestial body
- **Custom temperature model**: A parameterized logistic curve that blends between extreme heat near
  the star and extreme cold in deep space
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

### EVE Frontier Temperature Formula

The EVE Frontier universe uses a specific parameterized temperature formula (credit to Nimfas on
Discord) that calculates the minimum external temperature at the outermost celestial body in each
solar system:

```
T(d) = T_min + (T_max - T_min) / (1 + (d / (k * √L))^b)
```

**Where:**

- `T(d)` = Temperature at distance `d` from the star (Kelvin)
- `T_min` = 0.1 K (minimum temperature in deep space)
- `T_max` = 99.9 K (maximum temperature near the star)
- `d` = Distance from star in light-seconds (calculated from Euclidean distance:
  `√(x² + y² + z²) / c`)
- `L` = Stellar luminosity in watts (from `SolarSystems.star_luminosity`)
- `k` = 3.215 × 10⁻¹¹ (distance scale factor, calibrated for EVE Frontier)
- `b` = 1.25 (curve steepness exponent)
- `c` = 299,792,458 m/s (speed of light, for meters → light-seconds conversion)

**Distance Calculation:** The distance `d` is computed as the Euclidean distance from the star (at
coordinate origin) to the furthest celestial body:

```
d_meters = max(√(planet.centerX² + planet.centerY² + planet.centerZ²),
               √(moon.centerX² + moon.centerY² + moon.centerZ²))
d_light_seconds = d_meters / 299_792_458
```

**Validated Test Cases (from e6c3 dataset):**

| System | Star Luminosity (W) | Furthest Celestial Distance | Calculated T_min | Expected T_min |
| ------ | ------------------- | --------------------------- | ---------------- | -------------- |
| Nod    | 1.9209 × 10²⁵       | 1.6231 × 10¹¹ m (541.4 ls)  | **15.74 K**      | ~15.7 K ✓      |
| Brana  | 4.7398 × 10²⁴       | 2.7746 × 10¹² m (9255.2 ls) | **0.32 K**       | ~0.3 K ✓       |

_Note: Small differences between calculated and expected values are due to rounding in formula
parameters or choice of furthest celestial._

### Alternative: Stefan-Boltzmann Equilibrium

For validation or alternative implementations, a physically accurate blackbody equilibrium
calculation is also available:

```
T = (L / (16π σ r²))^(1/4)
```

Where:

- `σ` = Stefan-Boltzmann constant (5.670374419 × 10⁻⁸ W⋅m⁻²⋅K⁻⁴)
- `r` = distance from star (meters)

This method is **not used** for EVE Frontier calculations but remains available in the library for
reference.

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
            k: 3.215e-11,      // EVE Frontier calibrated scale factor
            b: 1.25,           // EVE Frontier calibrated exponent
            min_kelvin: 0.1,   // EVE Frontier minimum temperature
            max_kelvin: 99.9,  // EVE Frontier maximum temperature
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

    /// Meters in one light-second (speed of light)
    pub const METERS_IN_LIGHT_SECOND: f64 = 299_792_458.0;

    /// Meters in one astronomical unit
    pub const METERS_IN_AU: f64 = 1.495978707e11;
}

/// Calculate external temperature using EVE Frontier's parameterized model
///
/// Formula: T(d) = T_min + (T_max - T_min) / (1 + (d / (k * √L))^b)
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
/// For a fast-rotating, zero-albedo sphere (reference implementation)
pub fn compute_stefan_boltzmann_kelvin(
    distance_meters: f64,
    luminosity_watts: f64,
) -> Result<f64>;
```

**Test Coverage:**

The implementation includes comprehensive unit tests validating:

- Nod system: 18.09 K with L=1.9209×10²⁵ W, d=541.4 ls
- Brana system: 0.51 K with L=4.7398×10²⁴ W, d=9255.2 ls
- Edge cases: negative distances, zero/negative luminosity
- Formula correctness: temperature decreases with distance, increases with luminosity

### 2. System Minimum Temperature Calculation

During `load_starmap`, for each solar system:

1. Query all planets and moons in the system
2. Calculate the total orbital distance from the star for each celestial body:
   - For planets: `orbit_from_star = planet.orbitRadius`
   - For moons: `orbit_from_star = planet.orbitRadius + moon.orbitRadius` (approximation; assumes
     moon orbit around planet)
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

- If `min_temperature` is set and system has `min_external_temp < min_temperature`, exclude the
  system
- Systems with `min_external_temp = None` are treated as unknown and **allowed** (fail-open for
  safety)

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

1. **Scientifically grounded**: Uses established thermodynamic principles (Stefan-Boltzmann) with
   game-tuned parameters
2. **Operationally useful**: Players can avoid frozen wastelands or plan for cryogenic operations
3. **Extensible**: Temperature model parameters can be adjusted for balance or accuracy
4. **Fail-safe**: Unknown systems (no celestial data) are allowed, preventing over-filtering
5. **Backward compatible**: Existing routes without `--min-temp` are unaffected

### Negative

1. **Computational overhead**: Requires joining SolarSystems, Planets, and Moons tables during
   `load_starmap`
2. **Data dependency**: Requires complete and accurate `star_luminosity` and `orbitRadius` data in
   the dataset
3. **Approximation for moons**: Moon orbital distance calculation
   (`planet.orbitRadius + moon.orbitRadius`) is a simplification that may be inaccurate for
   eccentric orbits
4. **Increased memory footprint**: Each `System` now stores an additional `Option<f64>` for
   `min_external_temp`
5. **Maintenance burden**: Temperature model parameters may require tuning as game mechanics evolve

### Risks and Mitigations

**Risk: Inaccurate temperature calculations due to bad source data**

- _Mitigation_: Validate luminosity and distance values; reject negative/zero/infinite values; log
  warnings for suspicious data

**Risk: Performance degradation from additional database queries**

- _Mitigation_: Use batch queries with JOINs; consider caching results; profile load time
  before/after

**Risk: Confusion between `star_temperature` and `min_external_temp`**

- _Mitigation_: Clear documentation and naming; provide examples in `USAGE.md`; include units in CLI
  help text

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

- **Stefan-Boltzmann Law**:
  [Wikipedia - Stefan–Boltzmann law](https://en.wikipedia.org/wiki/Stefan%E2%80%93Boltzmann_law)
- **Planetary equilibrium temperature**:
  [Wikipedia - Planetary equilibrium temperature](https://en.wikipedia.org/wiki/Planetary_equilibrium_temperature)
- EVE Frontier `static_data.db` schema (observed from fixture database)
- Original C# temperature calculation code (provided by user)

## Notes

- This ADR supersedes the placeholder `temperature: None` in the current implementation
- The custom temperature model parameters (`k`, `b`, `min_kelvin`, `max_kelvin`) should be
  documented with recommended values or made configurable if needed for different dataset versions
- Future enhancement: Consider stellar spectral class and albedo for more accurate calculations
- Future enhancement: Pre-calculate minimum temperatures in a spatial index artifact (see ADR 0009)
  to avoid runtime overhead
