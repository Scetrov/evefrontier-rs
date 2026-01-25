# Quickstart: Scout Fuel and Heat Projection

**Spec**: 026 | **Date**: 2026-01-24

## What This Feature Does

Adds fuel and heat projection to `scout range` when a ship is specified. Instead of showing systems
sorted by distance, it plans a scouting route using nearest-neighbor ordering and calculates:

- Per-hop fuel cost and remaining fuel
- Per-hop heat and cumulative heat
- Cooldown times when overheated
- REFUEL/OVERHEATED warnings

## Quick Examples

### Basic Scout (no ship - existing behavior)

```bash
evefrontier-cli scout range "Nod" --limit 5
```

Output shows 5 nearest systems sorted by distance from Nod.

### Scout with Fuel/Heat Projections

```bash
evefrontier-cli scout range "Nod" --limit 10 --ship Reflex
```

Output shows 10 systems in optimal visiting order with fuel/heat for each hop.

### Scout with Custom Fuel Quality

```bash
evefrontier-cli scout range "Nod" --limit 10 --ship Reflex --fuel-quality 15
```

Higher fuel quality = more efficient jumps (lower fuel consumption).

### Scout with Cargo Mass

```bash
evefrontier-cli scout range "Nod" --radius 50 --ship Reflex --cargo-mass 5000
```

Adds 5000 kg cargo to mass calculations, increasing fuel/heat per hop.

### JSON Output for Automation

```bash
evefrontier-cli scout range "Nod" --limit 10 --ship Reflex --format json
```

Returns structured JSON with all fuel/heat fields for programmatic use.

## Understanding the Output

### Enhanced Format

```
Systems in range of Nod (10 found):
  (radius 50.0 ly, limit 10, ship: Reflex)

Scouting route visiting 10 systems:
  1. ● Brana (18.9 ly)   3 Planets 10 Moons 
       │ min   5.48K, fuel  89 (rem 1661), heat  12.34
  2. ● H:2L2S (14.2 ly)   1 Planet 1 Moon 
       │ min  89.58K, fuel  72 (rem 1589), heat  21.84 ⚠ OVERHEATED
  ...

───────────────────────────────────────
  Total Distance:        142 ly
  Fuel (Reflex):         648 (37% of capacity)
  Remaining:           1,102
  Total Wait:             0s
  Final Heat:          58.23
```

**Key fields**:
- `(18.9 ly)` - Distance from previous system (not from origin)
- `fuel 89` - Fuel units consumed for this hop
- `rem 1661` - Fuel remaining after this hop
- `heat 12.34` - Cumulative heat at this point
- `⚠ OVERHEATED` - Warning when heat ≥ 90

### Warnings

| Warning | Meaning |
|---------|---------|
| `⚠ REFUEL` | Not enough fuel for next hop |
| `⚠ OVERHEATED` | Heat ≥ 90 units |
| `🔥 CRITICAL (wait 30s)` | Heat ≥ 150 units; shows cooldown time |

## Algorithm

The nearest-neighbor heuristic:
1. Start at origin system
2. Find the closest unvisited system
3. Calculate fuel/heat for the hop
4. Move there and repeat until all systems visited

This is a greedy approximation, not globally optimal, but works well for 5-20 systems.

## Integration with Route Command

The fuel/heat calculations use the same library functions as `route`:
- `calculate_jump_fuel_cost()` - per-hop fuel
- `calculate_jump_heat()` - per-hop heat

This ensures consistency between scouting and route planning.

## Limitations

- No return-to-origin option (may be added later)
- Greedy algorithm may not find optimal path for complex clusters
- Gate neighbors (`scout gates`) do not support fuel/heat (gates are free)
