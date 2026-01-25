# CLI Contracts: Scout Range Fuel/Heat Projection

This document defines the CLI interface contracts for fuel/heat projection in scout range.

## Command Interface

### scout range (extended)

```
evefrontier-cli scout range <SYSTEM> [OPTIONS]
```

| Argument/Option | Type | Required | Default | Description |
|-----------------|------|----------|---------|-------------|
| `SYSTEM` | String | Yes | — | System name (case-sensitive with fuzzy suggestions) |
| `--limit, -n` | usize | No | 10 | Maximum results (1-100) |
| `--radius, -r` | f64 | No | None | Maximum distance in light-years |
| `--max-temp, -t` | f64 | No | None | Maximum temperature in Kelvin |
| `--include-ccp-systems` | bool | No | false | Include AD###/V-### systems |
| `--ship, -s` | String | No | None | Ship name for fuel/heat projections |
| `--fuel-quality` | f64 | No | 10.0 | Fuel quality percent (1-100) |
| `--cargo-mass` | f64 | No | 0.0 | Cargo mass in kilograms |
| `--fuel-load` | f64 | No | ship capacity | Starting fuel units |

**Global flags applied**: `--format`, `--data-dir`, `--no-logo`

**Exit codes**: Standard Rust/CLI exit codes (non-zero on error).

## Output Formats

### Basic (without ship)

Current behavior - systems sorted by distance:

```
Systems within range of Nod (10 found):
  1. Brana (18.9 ly)
  2. H:2L2S (23.1 ly)
  ...
```

### Basic (with ship)

Systems in visit order with fuel/heat:

```
Scouting route from Nod (10 systems, ship: Reflex):
  1. Brana (18.9 ly) - fuel 89, heat 12.3
  2. H:2L2S (14.2 ly) - fuel 72, heat 21.8
  ...
Total: 142 ly, 648 fuel, 58.2 heat
```

### Enhanced (with ship)

Box-drawing format with full metrics:

```
Systems in range of Nod (10 found):
  (radius 50.0 ly, limit 10, ship: Reflex)

Scouting route visiting 10 systems:
  1. ● Brana (18.9 ly)   3 Planets 10 Moons 
       │ min   5.48K, fuel  89 (rem 1661), heat  12.34
  2. ● H:2L2S (14.2 ly)   1 Planet 1 Moon 
       │ min  89.58K, fuel  72 (rem 1589), heat  21.84
  ...

───────────────────────────────────────
  Total Distance:        142 ly
  Fuel (Reflex):         648 (37% of capacity)
  Remaining:           1,102
  Total Wait:             0s
  Final Heat:          58.23
```

### JSON (with ship)

```json
{
  "system": "Nod",
  "system_id": 30000001,
  "query": {
    "limit": 10,
    "radius": 50.0,
    "max_temperature": null
  },
  "ship": {
    "name": "Reflex",
    "fuel_capacity": 1750,
    "fuel_quality": 10.0
  },
  "count": 10,
  "total_distance_ly": 142.3,
  "total_fuel": 648.0,
  "final_heat": 58.23,
  "systems": [
    {
      "name": "Brana",
      "id": 30000002,
      "distance_ly": 18.9,
      "hop_fuel": 89.0,
      "cumulative_fuel": 89.0,
      "remaining_fuel": 1661.0,
      "hop_heat": 12.34,
      "cumulative_heat": 12.34,
      "min_temp_k": 5.48,
      "planet_count": 3,
      "moon_count": 10
    }
  ]
}
```

## Behavior Contract

1. **Ship lookup**: If `--ship` specified, resolve from ship_data.csv; error with suggestions if not found.
2. **Visit order**: With ship, systems are ordered by nearest-neighbor heuristic (not by distance from origin).
3. **Distance calculation**: `distance_ly` shows hop distance from previous system (or origin for first).
4. **Fuel tracking**: Per-hop cost, cumulative, and remaining fuel calculated with dynamic mass.
5. **Heat tracking**: Per-hop heat and cumulative heat tracked across route.
6. **Warnings**: Show REFUEL when fuel insufficient, OVERHEATED/CRITICAL when heat thresholds exceeded.
7. **Cooldown**: When heat exceeds critical, calculate and display wait time.
8. **Empty results**: Return success with count=0 and empty systems list.

## Warning Format

| Condition | Warning Text |
|-----------|--------------|
| Fuel < hop cost | `⚠ REFUEL` |
| Heat ≥ 90 | `⚠ OVERHEATED` |
| Heat ≥ 150 | `🔥 CRITICAL (wait Xs)` |
