# Feature Specification: Heat Mechanics Integration

**Branch**: `020-heat` | **Date**: 2026-01-02 | **Status**: Planning

## Summary

Integrate heat mechanics into EVE Frontier route planning by displaying heat generation,
accumulation, and tolerance status in route output. Heat calculations will use the
community-validated formula from `HEAT_MECHANICS.md` and ship-specific attributes from
`ship_data.csv`.

## Background

EVE Frontier ships generate heat during spatial jumps based on their mass, jump distance, and hull
characteristics. Exceeding a ship's heat tolerance can result in damage or operational penalties.
Currently, route planning shows fuel consumption but lacks heat impact information, leaving players
unable to assess thermal risk during route planning.

Heat mechanics are documented in `docs/HEAT_MECHANICS.md` with the formula:

```text
heat_gen = (3 × current_total_mass_kg × distance_ly) / (C × hull_mass_only_kg)
```

Where:

- `current_total_mass_kg` = hull + fuel + cargo mass (dynamically reduced as fuel is consumed)
- `distance_ly` = jump distance in light-years
- `hull_mass_only_kg` = ship base hull mass (empty, no fuel/cargo)
-- `C` = calibration constant (fixed at `1e-7` for stable, reproducible projections)
- `3` = proportionality constant for heat generation rate

Heat accumulates across the entire route and is evaluated against canonical absolute thresholds
(`HEAT_OVERHEATED`, `HEAT_CRITICAL`) rather than per-ship tolerances (the canonical dataset does
not provide per-ship `max_heat_tolerance`).

## Requirements

### Functional Requirements

**FR1: Heat Calculation**

- Calculate per-hop heat generation using the formula from `HEAT_MECHANICS.md`
- Support both static mass mode (constant mass) and dynamic mass mode (mass decreases as fuel
  consumed)
- Accumulate heat across all route hops
- Gate transitions generate zero heat

**FR2: Status Line Display**

- Add heat information to route step status lines in all output formats (plain, rich, in-game note)
- Show per-hop heat generation and cumulative heat at each step
- Display format: `Heat: +XX.XX (cumulative: YYY.YY / tolerance: ZZZ.ZZ)`
- Include heat warnings when cumulative heat exceeds thresholds

**FR3: Heat Warnings**

- Warn when cumulative route heat >= `HEAT_OVERHEATED` (overheated)
- Error/infeasible when cumulative route heat >= `HEAT_CRITICAL` (critical)
- Display warnings in route summary footer and include `can_proceed=false` when a hop is infeasible

**FR4: Heat Summary**

- Add `HeatSummary` struct with `total`, `warnings` fields (warnings driven by canonical HEAT_* thresholds)
- Include heat summary in `RouteSummary` when ship data is provided
- JSON output includes heat data for programmatic consumption

**FR5: CLI Integration**

- Heat projections automatically enabled when `--ship` flag is provided to `route` command
- No additional CLI flags required (uses existing `--dynamic-mass` flag for mass mode)
- Heat data included in all output formats (JSON, plain, rich, in-game note)

### Non-Functional Requirements

**NFR1: Performance**

- Heat calculations add < 5% overhead to route planning
- No additional I/O required (uses existing ship data loaded for fuel calculations)

**NFR2: Accuracy**

- Heat formula validated against community sources and in-game observations
- Test coverage includes known heat generation scenarios with expected values

**NFR3: Maintainability**

- Heat calculation logic isolated in `ship.rs` module
- Clear separation between heat generation (per-hop) and accumulation (route-level)

## Success Criteria

1. Route output includes heat information when ship data is provided
2. Heat warnings correctly identify routes exceeding thermal tolerance
3. Heat calculations match community-validated formula
4. All existing tests pass; new heat tests achieve 100% coverage
5. ADR 0015 updated with heat mechanics design decisions

## Out of Scope

- Heat dissipation over time (deferred; requires time-based routing)
- Heat-based route optimization (finding lowest-heat routes)
- Visual heat mapping in web-based explorer (deferred to ADR 0016)
- Ship damage mechanics from heat exposure (game mechanics, not routing)

## References

- `docs/HEAT_MECHANICS.md` — Community-sourced heat formula and examples
- `docs/adrs/0015-fuel-cost-heat-impact-calculation.md` — ADR documenting fuel/heat design
-- `data/ship_data.csv` — Ship attributes (canonical CSV does **not** include per-ship `max_heat_tolerance` or `heat_dissipation_rate`)
- `crates/evefrontier-lib/src/ship.rs` — Ship data module with fuel calculations
- `crates/evefrontier-lib/src/output.rs` — Route output formatting
