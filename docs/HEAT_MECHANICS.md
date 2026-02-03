# Heat mechanics and fuel range (summary)

This document summarizes community-sourced formulas and reasoning about ship heat and fuel influence
on jump range. It is intended as a research note and a starting point for a formal ADR or
implementation in `ship.rs` (see ADR 0015).

## Temperature Calculation Model

The system models ship temperatures using actual ambient temperatures from the EVE Frontier universe
without artificial floor clamping. This section clarifies the temperature model implementation and
corrects common misconceptions.

### Temperature Ranges and Thresholds

**Game Temperature Range:**

- Ambient temperatures range from **0.1K to 99.9K** (calculated via the inverse-tangent
  heat-signature model based on distance from star and stellar properties)
- The implementation allows temperatures down to **0.1K** (game minimum) without clamping to 30K
- A floor of `max(0.0, temp)` prevents negative values from invalid data only

### Logistic Curve Model (Validated Against Game Data)

The system uses a **logistic curve model** that has been validated against actual EVE Frontier
temperature measurements. This model achieves **~2-5% mean average error** across diverse systems.

**Formula:**

$$T(d, L) = T_{min} + \frac{T_{max} - T_{min}}{1 + (d / (k \sqrt{L}))^b}$$

Where:
- $T_{min} = 0.1$ K — game minimum temperature
- $T_{max} = 99.9$ K — game maximum temperature  
- $d$ — distance from star in light-seconds
- $L$ — stellar luminosity in watts
- $k = 3.215 \times 10^{-11}$ — calibrated distance scale factor
- $b = 1.25$ — calibrated curve steepness exponent

**Validation Results (2026-02-03):**
- Mean Absolute Error: **~1.5 K**
- Most systems: **<5% error**
- Max error: **~9 K** (very cold systems near detection limits)

### Inverse-Tangent Heat-Signature Model (Research - Not Validated)

This section documents the inverse-tangent model developed by community researcher
[Ergod](https://awar.dev/). **Note: This model has NOT been validated against actual EVE Frontier
temperatures and shows 45-99% errors compared to in-game measurements.**

**Reference:**
[No more Traps, Inverse-Tangent Heat-Signature Model](https://thoughtfolio.xyz/No+more+Traps%2C+Inverse-Tangent+Heat-Signature+Model)
(2026-01-29, updated 2026-01-31)

**Status:** ✅ **VALIDATED** - This is the default temperature calculation method. Achieves ~1.2K MAE
(< 2% error) against actual in-game measurements. The flux-based formulation uses radiative
intensity (L/d²) rather than total luminosity, making it physically meaningful.

#### Final Formula (Flux-Based)

The external heat signature (ambient temperature) at distance $D$ from the star is:

$$T = \frac{200}{\pi} \cdot \arctan\left(\sqrt{\frac{L}{D^2 \times 10^{21}}}\right)$$

Where:

- $T$ — temperature in Kelvin
- $L$ — stellar luminosity in watts
- $D$ — distance from the star in light-seconds
- $10^{21}$ — intensity normalization constant (fitted from game data)
- $\frac{200}{\pi} \approx 63.66$ — radians-to-gradians conversion factor

#### Physical Interpretation

This formula uses **radiative flux** (intensity) rather than total luminosity:

- **Flux** = $\frac{L}{D^2}$ follows the inverse-square law for radiation
- The square root $\sqrt{\text{flux}}$ represents the "signature magnitude"
- The $\arctan$ function provides saturation at close distances (prevents infinite temperature)
- The $10^{21}$ normalization scales flux to a dimensionless argument for arctan
- The $\frac{200}{\pi}$ factor converts radians to gradians (game's temperature unit scale)

**Note:** This is the formula actually used by heatsense.pages.dev and achieves the claimed
<1% MAE. The Ergod publication's various formula revisions were attempts to describe this
underlying flux-based relationship.

#### Model Performance (Against EVE Frontier Game Data)

| Model                          | Mean Absolute Error | Max Error  | Status              |
| ------------------------------ | ------------------- | ---------- | ------------------- |
| **Flux-Based Inverse-Tangent** | **~1.2 K (< 2%)**   | **~9 K**   | ✅ **Validated (Default)** |
| Logistic Curve                 | ~1.2 K (< 2%)       | ~9 K       | ✅ Validated        |
| Exponential Decay              | ~5%                 | >10%       | ⚠️ Legacy (unused)  |

**Note:** The flux-based inverse-tangent formula achieves 9/10 systems with <5% error and matches
the performance of heatsense.pages.dev. This is the production-validated formula derived from
Ergod's research but using radiative flux (L/d²) rather than normalized luminosity (L/L☉).
Both validated models are available; inverse-tangent is default for its physical interpretability.

#### Implementation Notes

The arctan form provides a saturation feature that better models the slower temperature variation
near the star—a regime where exponential decay models struggled due to limited data.

```rust
/// Calculate ambient temperature at distance from star using inverse-tangent model.
///
/// # Arguments
/// * `distance_ls` - Distance from star in light-seconds
/// * `luminosity_ratio` - Stellar luminosity / Solar luminosity (L/L☉)
///
/// # Returns
/// Temperature in Kelvin (range approximately 0.1K to 99.9K)
fn calculate_ambient_temperature(distance_ls: f64, luminosity_ratio: f64) -> f64 {
    const A: f64 = 100.0;  // Scale factor [0,1] -> [0,100] K
    const K: f64 = 200.0 / std::f64::consts::PI;  // ~63.66 LS

    let lambda = K * luminosity_ratio;
    A * (2.0 / std::f64::consts::PI) * (lambda / distance_ls).atan()
}
```

#### Acknowledgements

This research was conducted by [awar.dev](https://awar.dev/) with data contributions from:

- [anteris90](https://github.com/anteris90) — initial data gathering
- [Diabolacal](https://github.com/Diabolacal) — game database validation

This work is shared publicly as part of the awar.dev Research Disclosure Protocol.
**Canonical Thresholds (from CCP's TemperatureThreshold class):**

```python
class TemperatureThreshold:
    NOMINAL = 30.0      # Safe operation: 0-30K
    OVERHEATED = 90.0   # Warning state: 90-150K
    CRITICAL = 150.0    # Dangerous state: ≥150K
```

**Important:** These thresholds are used for **status classification only**, not as temperature
floors. A ship at 2.76K ambient is perfectly valid and will remain at 2.76K, not be artificially
raised to 30K.

### Instantaneous Temperature Calculation

**Formula:**

```
T_instantaneous = T_ambient + ΔT_jump
```

Where:

- `T_ambient` = minimum external temperature of the origin system (0.1K-99.9K range)
- `ΔT_jump` = heat generated by the jump: `energy / (mass × specific_heat)`
- `energy` = jump heat energy from `calculate_jump_heat()`

**Status Labels (OVERHEATED/CRITICAL):**

Labels reflect the **instantaneous temperature during the jump** (i.e. temperature generated by the
jump) plus the minimal temperature of the origin system, not artificial floor values:

- If `T_instantaneous < 90K` → no warning
- If `90K ≤ T_instantaneous < 150K` → "OVERHEATED"
- If `T_instantaneous ≥ 150K` → "CRITICAL"

**Example (Bug Fix):**

```
Origin: IDR-KR4 (ambient 2.76K)
Jump: 39 light-years
Ship: Reflex
Jump heat: 132.57K

OLD (BUGGY): T_instantaneous = max(30.0, 2.76) + 132.57 = 162.57K → CRITICAL ❌
NEW (FIXED): T_instantaneous = 2.76 + 132.57 = 135.33K → OVERHEATED ✅
```

The old code incorrectly clamped starting temperature to `HEAT_NOMINAL` (30K), inflating the total
temperature and causing incorrect CRITICAL labels.

### Cooling Model

**Target Temperature:**

Ships cool from overheated states toward the minimum ambient temperature of the current solar system
as the target.

**Cooling Formula (Newton's Law of Cooling):**

```
T(t) = T_env + (T_0 - T_env) × e^(-kt)

Where:
  t = -1/k × ln((T_target - T_env) / (T_start - T_env))
  k = (BASE_COOLING_POWER × zone_factor) / (mass × specific_heat)
```

**Residual Temperature After Cooling:**

```
T_residual = max(T_target, T_ambient + COOLING_EPSILON)
```

Where `COOLING_EPSILON = 0.01K` prevents numerical issues in the cooling formula.

**Examples:**

```
# Cold System (5K ambient)
Start: 90K → Target: 30K → Residual: max(30K, 5.01K) = 30K ✓

# Hot System (40K ambient)
Start: 90K → Target: 30K → Residual: max(30K, 40.01K) = 40.01K ✓
```

In hot systems, ships cannot cool below ambient temperature, so residual reflects the physical
constraint.

### Zone Factor and Cooling Rate

The `zone_factor` scales cooling rate inversely with ambient temperature:

- **Cold systems (low T_ambient)** → high zone_factor → high k → **fast cooling**
- **Hot systems (high T_ambient)** → low zone_factor → low k → **slow cooling**

**Cooling Constant:**

```rust
k = (BASE_COOLING_POWER × zone_factor) / (total_mass_kg × specific_heat)

BASE_COOLING_POWER = 1e6  // W/K units
```

This ensures realistic behavior where ships cool more efficiently in cold systems (e.g., frostline
zones) than in hot systems (inner zones near stars).

### Gate Hops and Heat Reset

**Gate transitions reset heat to 0**, so cooling calculations do not apply:

- Gate hops have no cooling time displayed
- Next hop after a gate starts from ambient temperature of the new system
- Heat projections show `heat 0.00` for gate arrivals

**Example:**

```
 GATE  ● O43-CT4 (gate, 19ly)   1 Planet 4 Moons
       │ min  50.39K, fuel   0 (rem  885), heat 0.00
```

### Heat-Aware Routing Default Behavior

**--avoid-critical-state Flag:**

- **Defaults to TRUE** for all routes (heat-aware routing enabled by default)
- Uses **Reflex as default ship** when `--ship` is not specified
- Rejects spatial jumps where `T_instantaneous ≥ HEAT_CRITICAL` (150K)
- Disable with `--no-avoid-critical-state` for gate-only networks or high-risk planning

**Use Cases for Disabling:**

- Gate-only networks (temperature irrelevant since gates reset heat)
- Intentional high-risk route planning where critical temperatures are acceptable
- Performance optimization for batch processing (skips heat calculations)

**CLI Examples:**

```bash
# Default: heat-aware routing with Reflex
evefrontier-cli route --from "Nod" --to "Brana"

# Explicit ship specification
evefrontier-cli route --from "Nod" --to "Brana" --ship "Vanguard"

# Disable temperature constraints
evefrontier-cli route --from "Nod" --to "Brana" --no-avoid-critical-state
```

**Lambda API:**

```json
{
  "from": "Nod",
  "to": "Brana",
  "avoid_critical_state": true, // Default: true
  "ship": null // Defaults to "Reflex" when avoid_critical_state=true
}
```

### Temperature Model Summary Table

| Scenario                   | T_ambient | ΔT_jump | T_instantaneous | Label      | Residual After Cooling |
| -------------------------- | --------- | ------- | --------------- | ---------- | ---------------------- |
| Cold system, small jump    | 2.76K     | 50K     | 52.76K          | (none)     | 30.0K                  |
| Cold system, medium jump   | 2.76K     | 100K    | 102.76K         | OVERHEATED | 30.0K                  |
| Cold system, large jump    | 2.76K     | 132.57K | 135.33K         | OVERHEATED | 30.0K                  |
| Cold system, critical jump | 2.76K     | 150K    | 152.76K         | CRITICAL   | 30.0K                  |
| Hot system, medium jump    | 40K       | 60K     | 100K            | OVERHEATED | 40.01K                 |
| Very cold system           | 0.1K      | 89K     | 89.1K           | (none)     | 30.0K                  |
| Gate hop                   | any       | 0       | 0.0             | (none)     | 0.0                    |

### Implementation Notes

**Files Modified (Bug Fixes):**

- `crates/evefrontier-lib/src/ship/heat.rs:294` — removed `HEAT_NOMINAL.max()` floor
- `crates/evefrontier-lib/src/ship/heat.rs:324` — fixed residual to use `COOLING_EPSILON`
- `crates/evefrontier-lib/src/path.rs` — changed `avoid_critical_state` default to `true`

**Test Coverage:**

- Unit tests: `crates/evefrontier-lib/tests/heat_threshold_regression.rs`
- Integration tests: `crates/evefrontier-lib/tests/routing.rs`

---

Summary formula (informational):

```
range = fuel_volume × fuel_quality / (FUEL_CONSTANT × ship_mass)

// Where:
// FUEL_CONSTANT = 0.0000001 (1e-7)  # example constant from community sources
// fuel_volume = liters (fuel_quantity × 0.28 m³/unit)
// fuel_quality = 0.1 to 0.9 (10% to 90% purity)
```

Notes:

- This is a community-derived formula and should be validated before use in production.
- Keep the document concise — implementation choices (units, normalization) must be documented in
  the ship data ADR before this formula is used for route fuel/heat calculations.

Display and dataset notes:

- The canonical game CSV does **not** include per-ship `max_heat_tolerance` or
  `heat_dissipation_rate`, so the implementation does not rely on per-ship tolerances.
- The system uses canonical absolute heat thresholds (NOMINAL / OVERHEATED / CRITICAL) to classify
  cumulative route heat and drive warnings: `HEAT_OVERHEATED` and `HEAT_CRITICAL`.

References

- [No more Traps, Inverse-Tangent Heat-Signature Model](https://thoughtfolio.xyz/No+more+Traps%2C+Inverse-Tangent+Heat-Signature+Model)
  — awar.dev's community research achieving 0.15% MAE for ambient temperature calculation
- [All to Avoid Heat Traps, Exponential Heat-Signature Decay Model](https://thoughtfolio.xyz/All+to+Avoid+Heat+Traps%2C+Exponential+Heat-Signature+Decay+Model)
  — Predecessor exponential decay model research
- [Jump Calculators: Understanding Your Ships Heat and Fuel Limits](https://ef-map.com/blog/jump-calculators-heat-fuel-range)

TODO

- Add a short table of example calculations.
- Link this doc from `docs/USAGE.md` and `docs/ADR` once the formula is agreed and tests are added.

## Newton's Law of Cooling Implementation

The system models ship cooling using Newton's Law of Cooling, providing a more realistic exponential
decay where cooling slows down as the ship approaches the ambient temperature.

**Key Equations**:

- **Solution**: $T(t) = T_{env} + (T_0 - T_{env}) e^{-kt}$
- **Cooldown Time**:
  $t = -\frac{1}{k} \ln\left(\frac{T_{threshold} - T_{env}}{T_{start} - T_{env}}\right)$

**Implementation Details**:

- **Cooling Constant (k)**: `k = BASE_COOLING_POWER * zone_factor / (total_mass_kg * specific_heat)`
- **Wait Time**: The time required to reach the jump-ready state (30.0 K) is calculated per hop.
- **Start Temperature (T_start)**: Assumed to be T_ready + delta_T_jump, where T_ready is max(30.0K,
  T_ambient_origin).
- **Ambient Floor**: Ships cannot cool below the system's ambient temperature (T_env).

## Integration with the game Temperature Service (Historical Notes)

_Note: The following sections describe research into the game's TemperatureSvc and suggested models
that informed the current implementation._

Finding: The game's TemperatureSvc (decompiled) exposes a compact set of primitives that are useful
for implementing a realistic cooling/dissipation model:

- Current and base temperatures via `current_temperature()` and `base_temperature()`
- A `temperature_time_scale()` value used by the game to scale time-based temperature changes
- Threshold callbacks and predictions (`get_next_state_change()`, `get_current_state()`) for
  `TemperatureThreshold` → `TemperatureState` transitions (nominal → overheated → critical)
- Zone detection (`get_current_temperature_zone()`), derived from ship position and
  `get_temperature_zone(distance_from_sun, system_data)` and signalled via `zone_changed_signal()`
- Active signals for state/zone changes (`state_change_signal`, `zone_changed_signal`) and an
  attribute update handler (`OnAttributes`) that triggers recalculation.

Practical implications:

- We can compute a location-aware cooling efficiency by combining a **base dissipation rate** with a
  **zone factor** (e.g., cold/frostline → high cooling, inner/hot zones → low cooling).
- For higher fidelity, use the game-provided `current_temperature`/`base_temperature` and the
  `temperature_time_scale()` to convert the game's natural temperature decay/growth into an
  effective dissipation per second.
- Use the `get_next_state_change()` and `state_change_signal` to invalidate cached wait-time
  calculations when system or ship state changes (prevents stale recommendations).

Suggested cooling/dissipation models

1. Zone-based (conservative, opt-in)
   - `dissipation_per_sec = BASE_DISSIPATION * zone_factor(zone)`
   - `zone_factor` may be: FROSTLINE=1.0, OUTER=0.8, NOMINAL=0.5, INNER=0.2, HOT=0.0
   - Pros: simple, predictable, easy to test
   - Cons: less realistic than a game-driven model

2. Game-driven (recommended for realism)
   - Use `current_temperature()`, `base_temperature()` and `temperature_time_scale()` to compute a
     per-second cooling rate consistent with how the game models temperature evolution. Translate
     the game's next-state predictions into wait-time estimates.
   - Pros: faithful to in-game behavior, responsive to system state
   - Cons: requires careful unit validation and more tests

Using dissipation to compute `HeatProjection` fields

- Required reduction = (cumulative + hop_heat) - target_threshold (HEAT_OVERHEATED or HEAT_CRITICAL)
- wait_seconds = required_reduction / dissipation_per_sec (if dissipation_per_sec > 0)
- cooled_cumulative_heat = max(0.0, (cumulative + hop_heat) - dissipation_per_sec × wait_seconds)
- If dissipation_per_sec == 0 and required_reduction > 0, mark `can_proceed = false` (infeasible)

Tests & validation

- Unit tests for zone-based factors: assert that wait times and cooled heat are smaller for icy
  zones than for hot zones with identical inputs.
- Integration tests mocking `TemperatureSvc` (or substituting deterministic values) to ensure
  `wait_time_seconds` and `cooled_cumulative_heat` are consistent with `temperature_time_scale()`
  and `get_next_state_change()` predictions.
- Contract tests: ensure JSON output (`HeatProjection`) includes `wait_time_seconds` and
  `cooled_cumulative_heat` when waiting is recommended and `can_proceed=false` when infeasible.

Notes & caveats

- The decompiled code is a guide — verify numeric semantics (units, scaling) with canonical game
  data before relying on precise numeric thresholds or rates.
- Keep dissipation models configurable and opt-in initially (feature flag or CLI option) and
  document default behavior in `docs/USAGE.md`.

Next steps

- Decide whether to implement a zone-based prototype (fast) or the game-driven model (realistic).
- Add tests (unit + integration) and examples in `docs/USAGE.md` and this file.
- Implement the chosen model and add tests and docs in a follow-up (tracked in tasks.md).

### Conservative avoidance: `--avoid-critical-state`

We implemented a conservative, opt-in avoidance check to help pilots avoid single jumps that would
instantly push a ship's drive into the **CRITICAL** heat band. The behavior is intentionally simple
and deterministic so it can be applied at planning time without requiring expensive stateful
searches or lookahead logic.

- **What it does:** When the CLI is invoked with `--avoid-critical-state` (requires `--ship`), the
  planner computes the hop-specific heat for each spatial edge using:
  1. `calculate_jump_heat(total_mass_kg, distance_ly, hull_mass_kg, calibration_constant)` to
     compute an energy-like quantity.
  2. Convert to a temperature delta using the ship's specific heat:
     `delta_T = energy / (mass * specific_heat)`.
  3. The instantaneous temperature experienced during the jump is `ambient_temperature + delta_T`.
  4. If that instantaneous temperature is >= `HEAT_CRITICAL` (150.0 K), the spatial edge is rejected
     and not considered by the pathfinder.

- **Important temperature clarification:** The `ambient_temperature` used in this check is the
  **minimum external temperature** (`min_external_temp`) calculated using the EVE Frontier logistic
  curve formula (range 0.1K-99.9K). This represents the black body temperature at the coldest
  habitable zone (typically at the furthest planet/moon from the star). It is **NOT** the stellar
  surface temperature (`star_temperature` in the database), which is in the thousands of Kelvin and
  represents the photosphere temperature of the star itself. The confusion between these two values
  was a critical bug that caused all routes to be rejected when first implemented.

- **Why conservative:** This check is per-hop and _does not_ model residual cumulative heat carried
  between hops. It therefore errs on the side of caution: any jump that would by itself be
  immediately dangerous is avoided. This is a practical, low-complexity mitigation that fits current
  routing infrastructure and avoids introducing non-deterministic or expensive stateful pathfinding
  into the MVP. Tests covering both static and `dynamic_mass` cases are included in
  `crates/evefrontier-lib/tests/route_dynamic_heat.rs` and
  `crates/evefrontier-lib/tests/routing_critical.rs`.

**Dynamic mass behavior:** Currently the instantaneous avoidance check uses the provided
`ShipLoadout` to compute mass (static at planning time). `HeatConfig::dynamic_mass` affects how
route _projections_ are calculated (fuel consumption across hops) but does not relax the avoidance
decision for the current MVP. The justification: modeling per-hop mass changes for avoidance
requires carrying per-node state in the search, which is a planned follow-up (see `Future work`
below).

- **CLI usage example:**

```bash
# Avoid any single jump that would reach CRITICAL temperature; requires --ship
evefrontier-cli route --from "Nod" --to "Brana" --avoid-critical-state --ship "Reflex"
```

- **Notes & tests:**
  - If `--avoid-critical-state` is used without `--ship` the CLI errors with a helpful message to
    avoid ambiguous behavior.
  - Unit & integration tests added: `routing_critical.rs`, `route_dynamic_heat.rs`, and
    `crates/evefrontier-cli/tests/route_avoid_critical.rs` (covers both error and success cases).

### Residual Heat and Cooldown (Non-Cumulative Model)

The system uses a non-cumulative thermal model. We assume that pilots wait for their ships to return
to a "jump-ready" state—either the nominal temperature (30.0 K) or the system's ambient temperature,
whichever is higher—before initiating the next jump.

Consequently, each hop's cooldown calculation starts from this baseline:
$T_{start} = \max(30.0, T_{ambient\_origin}) + \Delta T_{jump}$.

We do not track heat build-up across multiple hops, as residual heat dissipates quickly enough that
carrying it into the next system's jump calculation would introduce unnecessary complexity without
significantly improving the accuracy of the cooldown estimates provided by the tool.

It may, however, be useful to include cooldown time estimates in the output to help pilots plan
their routes; these will be indicative because they won't include time spent warping between the
jump-in point and the outermost celestial body in the destination system.

### Test Data (In-Game Measurements, 2026-02-03)

| System  | Distance (LS) | Luminosity (W)                   | Measured (K) | Logistic (K) | Error (K) | Error (%) |
| ------- | ------------- | -------------------------------- | ------------ | ------------ | --------- | --------- |
| A9R-PQ4 | 1168          | 117079286879216367304704.00      | 0.6          | 0.39         | 0.21      | 34.7%     |
| O86-215 | 3078          | 210611439263697778669256704.00   | 0.0          | 8.72         | 8.72      | —         |
| E37-N15 | 2735          | 46609614223656827010154496.00    | 5.1          | 4.19         | 0.91      | 17.9%     |
| UVG-MV3 | 6283          | 2399495653136391448790302720.00  | 15.7         | 15.13        | 0.57      | 3.6%      |
| U98-VK4 | 20850         | 95197165897008878019531505664.00 | 28.4         | 28.37        | 0.03      | 0.1%      |
| EKF-2N4 | 1297          | 3482357323030063141093376.00     | 2.9          | 2.20         | 0.70      | 24.3%     |
| ER7-MN4 | 165           | 5945032820976178790662144.00     | 28.3         | 28.33        | 0.03      | 0.1%      |
| E6S-8S4 | 3290          | 526627990081004722192384.00      | 0.4          | 0.31         | 0.10      | 23.8%     |
| O43-CT4 | 532           | 269067787727477819523989504.00   | 49.9         | 49.73        | 0.17      | 0.3%      |
| E17-S05 | 17            | 705658793434329632473088.00      | 63.3         | 64.05        | 0.75      | 1.2%      |

**Note:** O86-215 shows 0.0K measured temperature, which may indicate sensor limitations at extreme
distances or a measurement error. The high error for very cold systems (A9R-PQ4, EKF-2N4, E6S-8S4)
is expected as these approach the game's minimum temperature threshold.
