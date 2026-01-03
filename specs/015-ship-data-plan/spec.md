# Feature Specification: Ship Data & Fuel Calculations (ADR 0015)

**Feature Branch**: `015-ship-data-plan`  
**Created**: 2025-12-31  
**Status**: Draft  
**Input**: TODO item "Ship Data & Fuel Calculations" plus ADR 0015 (`docs/adrs/0015-fuel-cost-heat-impact-calculation.md`)

## User Scenarios & Testing

### User Story 1 - CLI user projects fuel for a route (Priority: P1)

A player uses `evefrontier-cli route --ship Reflex --fuel-quality 10 --cargo-mass 5000` to plan a trip. They need per-hop and total fuel usage with warnings if fuel exceeds capacity.

**Default Behavior**: When no `--ship` is specified, the CLI defaults to the Reflex ship with 10% fuel quality. Users can override both with explicit flags.

**Independent Test**: Running the CLI with fixture data returns route steps that include hop fuel cost, cumulative fuel, and an error when required fuel exceeds capacity unless `--no-download` is supplied.

### User Story 2 - Lambda client receives fuel projections (Priority: P1)

An API client calls the Lambda route endpoint with ship and loadout parameters. Response includes fuel projection fields and warnings in the summary.

**Independent Test**: Lambda handler returns JSON with `fuel_projection` per hop and `route_summary` containing totals and fuel warnings when capacity is insufficient.

### User Story 3 - Developer lists supported ships (Priority: P2)

A developer runs `evefrontier-cli --list-ships` (or equivalent flag) to display available ships from `ship_data.csv` with key attributes for selection.

**Independent Test**: CLI returns a tabular list of ship names and capacities sourced from cached or downloaded ship data.

### User Story 4 - Dynamic mass recalculation (Priority: P2)

A user enables `--dynamic-mass` to get more accurate projections. Fuel and heat are recalculated per hop as mass decreases.

**Independent Test**: Route output shows different totals between static and dynamic modes; tests assert dynamic mode consumes less or equal fuel than static for the same route.

### User Story 5 - Dataset bootstrap for ship data (Priority: P2)

Downloader fetches `ship_data.csv` alongside the database and caches it. Calls reuse cached data unless `--force` is specified.

**Independent Test**: `ensure_e6c3_dataset` (or downloader entrypoint) stores `ship_data.csv` in the cache directory with checksum marker; repeated runs avoid re-download when unchanged.

## Requirements

### Functional Requirements

- **FR-001**: Load `ship_data.csv` from dataset releases and cache it next to the DB (`evefrontier_datasets/`).
-- **FR-002**: Parse ship data with strict validation (fields: name, base_mass_kg, specific_heat, fuel_capacity, cargo_capacity); reject invalid rows with actionable errors. Note: per-ship heat tolerance and dissipation fields are not part of the canonical dataset and are not required.
- **FR-003**: Expose ship catalog APIs in `evefrontier-lib` (e.g., `ShipCatalog`, `ShipAttributes`, `ShipLoadout`) to retrieve ships by name and list all ships.
- **FR-004**: Implement fuel cost calculation `(total_mass_kg / 10^5) × (fuel_quality / 100) × distance_ly` supporting static and dynamic mass modes.
- **FR-005**: Implement route-level fuel projection returning per-hop and cumulative values; include remaining fuel when initial fuel load provided.
- **FR-006**: Extend `RouteStep`/`RouteSummary` to include optional `FuelProjection` fields (hop_cost, cumulative, remaining, warnings).
- **FR-007**: Extend CLI `route` subcommand with `--ship` (default: Reflex), `--fuel-quality` (default: 10%), `--cargo-mass`, `--fuel-load`, `--dynamic-mass`, and `--list-ships`; add friendly validation errors for unknown ships or invalid parameters.
- **FR-008**: Extend Lambda request/response schemas to accept ship/loadout/fuel parameters and return fuel projection data, preserving backward compatibility for callers omitting ship data.
- **FR-009**: Bundle `ship_data.csv` (and optional derived artifacts) into Lambda/container builds alongside the database and spatial index.
- **FR-010**: Provide fixtures and tests for ship data parsing and fuel calculations (static and dynamic) with known outputs; include CLI and Lambda integration tests using fixtures.

### Non-Functional Requirements

- **NFR-001**: Follow Library-First: all logic in `evefrontier-lib`; CLI/Lambda remain thin wrappers.
- **NFR-002**: Enforce TDD with unit tests for calculators and integration tests for CLI/Lambda; target coverage ≥70% in touched modules and 80% on critical paths.
- **NFR-003**: Input validation must be strict and produce actionable error messages without leaking sensitive paths (Security-First, OWASP A03).
- **NFR-004**: Maintain performance targets: fuel projection should add negligible latency (<5ms per route on fixture) and minimal memory overhead.
- **NFR-005**: Backward compatibility: existing CLI/Lambda calls without ship data continue to work with identical outputs (fuel fields omitted when ship not supplied).

### Key Entities

- **ShipAttributes**: Immutable attributes loaded from CSV.
- **ShipLoadout**: Runtime load (fuel, cargo) and derived total mass.
- **ShipCatalog**: Collection of ships with lookup by name and listing support.
- **FuelProjection**: Per-hop and total fuel usage, warnings, and remaining fuel.
- **RouteSummary/RouteStep**: Extended to optionally embed fuel projection data.

### Edge Cases

- Missing or corrupted `ship_data.csv` in cache or release asset.
- Ships with zero or extremely small mass/capacity values (validation rejects).
- Routes longer than available fuel even with full tank (warning/error path).
- Dynamic mass recalculation leading to negative fuel due to rounding (must clamp at zero).
- Backward compatibility when callers do not supply ship data (fuel projection disabled).

## Context & Prior Work

- ADR 0015 defines fuel and heat formulas plus dynamic vs static mass considerations.
- TODO item "Ship Data & Fuel Calculations" enumerates sub-tasks (ship.rs module, CLI flags, Lambda schema updates, fixtures, tests).
- Existing downloader already handles DB caching; extend to ship data asset in the same cache dir.
- Spatial index integration is complete; new fuel logic must coexist with existing routing outputs.

## CLI Defaults

- **Default Ship**: `Reflex` — automatically selected when `--ship` flag omitted
- **Default Fuel Quality**: `10%` — automatically used when `--fuel-quality` flag omitted
- **Rationale**: Reflex is EVE Frontier's starting ship; 10% is standard in-game fuel quality for T0 fuel blocks
- **Override**: Users can specify `--ship <name>` and `--fuel-quality <percent>` to use alternatives

## Acceptance Criteria

- [ ] `ship_data.csv` is downloaded/cached with checksum guard alongside the DB.
- [ ] Library exposes validated ship catalog and fuel projection APIs with unit tests (static & dynamic modes).
- [ ] `RouteStep`/`RouteSummary` include optional `FuelProjection`; outputs remain backward compatible when ship data absent.
- [ ] CLI supports ship-related flags and `--list-ships` with friendly validation and integration tests.
- [ ] Lambda request/response schemas accept ship/loadout params and return fuel projections; integration tests updated.
- [ ] Fixtures added for ship data; tests cover invalid input handling.
- [ ] Documentation updated (`USAGE.md`, `README.md`) with fuel examples and flags.
- [ ] CHANGELOG entry under Unreleased notes the new fuel projection capability and default ship/fuel settings.
