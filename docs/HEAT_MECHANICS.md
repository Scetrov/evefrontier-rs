# Heat mechanics and fuel range (summary)

This document summarizes community-sourced formulas and reasoning about ship heat and fuel influence
on jump range. It is intended as a research note and a starting point for a formal ADR or
implementation in `ship.rs` (see ADR 0015).

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

- [Jump Calculators: Understanding Your Ships Heat and Fuel Limits](https://ef-map.com/blog/jump-calculators-heat-fuel-range)

TODO

- Add a short table of example calculations.
- Link this doc from `docs/USAGE.md` and `docs/ADR` once the formula is agreed and tests are added.

## Integration with the game Temperature Service

Finding: The game's `TemperatureSvc` (decompiled) exposes a compact set of primitives that are
useful for implementing a realistic cooling/dissipation model:

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

### Residual Heat and Cooldown

Residual heat modeling is not especially helpful in the context of this tool as it's normal practice
to need to wait for the ship to cool down after jumps which follows Newton's Law of Cooling. Because
of this we don't believe that it is valuable to include a culmulative heat model in this or any
future implementation of the tool.

It may however be useful to include cooldown times in the output of the tool to help pilots plan
their routes, however this at best will be indicative as it won't include time taken to warp between
the jump-in point and the outermost celestial object in the system.
