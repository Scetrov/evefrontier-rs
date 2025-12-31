# Data Model — Ship Data & Fuel Calculations

## Entities

### ShipAttributes
- **Fields**: `name:String`, `base_mass_kg:f64`, `specific_heat:f64`, `fuel_capacity:f64`, `cargo_capacity:f64`, `max_heat_tolerance:f64`, `heat_dissipation_rate:f64`
- **Validation**: All numeric fields > 0; names unique (case-insensitive); reject NaN/inf; optional trimming on name.
- **Source**: Parsed from `ship_data.csv` (cached alongside DB).

### ShipLoadout
- **Fields**: `ship:ShipAttributes`, `fuel_load:f64`, `cargo_mass_kg:f64`, `fuel_quality:f64`, `dynamic_mass:bool`
- **Derived**: `total_mass_kg = base_mass_kg + fuel_load(kg) + cargo_mass_kg`
- **Validation**: `0 <= fuel_load <= fuel_capacity`; `cargo_mass_kg >= 0`; `fuel_quality` in [1,100]; optional max total mass check.

### ShipCatalog
- **Fields**: `ships:Vec<ShipAttributes>` with index by lowercased name; metadata `source_path`, `checksum`.
- **Operations**: `list()`, `get(name)`, `validate_unique_names()`, `from_csv(path)`.
- **Relations**: Supplies `ShipAttributes` to loadouts and calculators.

### FuelProjection
- **Fields**: `hop_cost:f64`, `cumulative:f64`, `remaining:Option<f64>`, `warning:Option<String>`
- **Validation**: `hop_cost >= 0`, `cumulative >= 0`, `remaining` never negative (clamp at zero), warnings emitted when fuel exceeds capacity or below zero.

### RouteStep (extension)
- **Existing**: System hop metadata, distance, temperature.
- **New**: `fuel:Option<FuelProjection>` when ship/loadout provided.

### RouteSummary (extension)
- **Existing**: Total distance, hops, formatting metadata.
- **New**: `fuel:Option<{ total:f64, remaining:Option<f64>, ship_name:Option<String>, warnings:Vec<String> }>`.

## Relationships

- `ShipCatalog` supplies `ShipAttributes` → `ShipLoadout` uses attributes to compute mass → `FuelProjection` calculated per `RouteStep` using `Route` distances.
- `RouteSummary` aggregates all `FuelProjection` values and warnings.

## State & Transitions

- **Load catalog**: `ship_data.csv` downloaded/cached → parsed into `ShipCatalog` (state: validated).
- **Select ship**: CLI/Lambda picks ship by name → constructs `ShipLoadout` after validating fuel/cargo inputs (state: ready).
- **Compute route**: Routing already produces steps; fuel calculator traverses steps, computing static or dynamic mass per hop → attaches `FuelProjection` to each step (state: enriched route).
- **Output**: CLI/Lambda formatters serialize fuel data; if ship not supplied, fuel fields remain `None`/absent (state: backward-compatible route).

## Validation Rules Summary

- Reject CSV rows with missing/negative numeric fields or duplicate names.
- Fuel quality must be 1–100; default 10 when unspecified (configurable flag).
- Dynamic mode recalculates mass per hop: `mass_next = mass_current - hop_cost` (clamp at zero); static mode keeps mass constant.
- Emit warnings when required fuel > capacity or remaining < 0.

## Open Questions (tracked in research)

- None (performance and scale clarified in Phase 0).
