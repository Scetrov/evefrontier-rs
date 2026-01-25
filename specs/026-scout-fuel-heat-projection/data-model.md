# Data Model: Scout Fuel and Heat Projection

**Spec**: 026 | **Date**: 2026-01-24

## Entities

### ScoutRangeArgs (Extended)

CLI arguments for `scout range` command, extended with ship options.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `system` | String | Yes | - | Origin system name |
| `limit` | usize | No | 10 | Max results (1-100) |
| `radius` | Option<f64> | No | None | Max distance in ly |
| `max_temp` | Option<f64> | No | None | Max temperature in K |
| `include_ccp_systems` | bool | No | false | Include AD###/V-### systems |
| `ship` | Option<String> | No | None | Ship name for fuel/heat projections |
| `fuel_quality` | Option<f64> | No | 10.0 | Fuel quality percent (1-100) |
| `cargo_mass` | Option<f64> | No | 0.0 | Cargo mass in kg |
| `fuel_load` | Option<f64> | No | ship.fuel_capacity | Starting fuel units |

### ScoutRangeResult (Extended)

Result struct for range queries, extended for fuel/heat when ship specified.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `system` | String | Yes | Origin system name |
| `system_id` | i64 | Yes | Origin system ID |
| `query` | RangeQueryParams | Yes | Query parameters |
| `ship` | Option<ShipInfo> | No | Ship info when specified |
| `count` | usize | Yes | Number of systems found |
| `total_distance_ly` | Option<f64> | No | Total route distance (when ship) |
| `total_fuel` | Option<f64> | No | Total fuel consumed (when ship) |
| `final_heat` | Option<f64> | No | Final cumulative heat (when ship) |
| `systems` | Vec<RangeNeighbor> | Yes | Systems in visit order |

### RangeNeighbor (Extended)

Individual system in range results, extended for fuel/heat projections.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | System name |
| `id` | i64 | Yes | System ID |
| `distance_ly` | f64 | Yes | Distance from previous hop (or origin) |
| `min_temp_k` | Option<f64> | No | Min external temperature |
| `planet_count` | Option<u32> | No | Number of planets |
| `moon_count` | Option<u32> | No | Number of moons |
| `hop_fuel` | Option<f64> | No | Fuel for this hop (when ship) |
| `cumulative_fuel` | Option<f64> | No | Total fuel consumed so far |
| `remaining_fuel` | Option<f64> | No | Fuel remaining after hop |
| `hop_heat` | Option<f64> | No | Heat generated this hop |
| `cumulative_heat` | Option<f64> | No | Total heat accumulated |
| `cooldown_seconds` | Option<f64> | No | Wait time if overheated |
| `fuel_warning` | Option<String> | No | REFUEL warning if insufficient |
| `heat_warning` | Option<String> | No | OVERHEATED/CRITICAL warning |

### ShipInfo

Ship information echoed in response.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Ship name |
| `fuel_capacity` | f64 | Yes | Max fuel capacity |
| `fuel_quality` | f64 | Yes | Fuel quality used |

### RangeQueryParams (Extended)

Query parameters echoed in response.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `limit` | usize | Yes | Max results requested |
| `radius` | Option<f64> | No | Max distance filter |
| `max_temperature` | Option<f64> | No | Max temperature filter |

## State Transitions

### Without Ship (Current Behavior)

```
Query → Find systems in range → Sort by distance → Return
```

### With Ship (New Behavior)

```
Query → Find systems in range → Nearest-neighbor order → Calculate fuel/heat per hop → Return
```

## Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| `ship` | Must exist in ship_data.csv | "Unknown ship: {name}. Did you mean: ..." |
| `fuel_quality` | 1.0 ≤ value ≤ 100.0 | "Fuel quality must be between 1 and 100" |
| `cargo_mass` | ≥ 0.0 | "Cargo mass cannot be negative" |
| `fuel_load` | ≥ 0.0, ≤ fuel_capacity | "Fuel load exceeds ship capacity" |
| `limit` | 1 ≤ value ≤ 100 | "Limit must be between 1 and 100" |

## Relationships

```
ScoutRangeArgs ─────┬──→ ShipCatalog (optional lookup)
                    │
                    └──→ Starmap (system lookup)
                           │
                           ↓
                    SpatialIndex (range query)
                           │
                           ↓
                    ScoutRangeResult
                           │
                           ├──→ RangeNeighbor[] (ordered by visit)
                           │
                           └──→ ShipInfo (when ship specified)
```
