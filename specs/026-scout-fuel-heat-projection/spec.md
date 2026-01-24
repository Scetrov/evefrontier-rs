# Feature Specification: Scout Fuel and Heat Projection

**Spec ID**: 026  
**Created**: 2026-01-24  
**Status**: Draft  
**Depends On**: 025 (Scout CLI Subcommand)

## Overview

Enhance the `scout range` subcommand to calculate fuel and heat projections for visiting all
discovered systems in a Hamiltonian path (visiting each system exactly once). This transforms scout
from a "show what's nearby" tool into a "plan a scouting route" tool.

## Problem Statement

The current `scout range` implementation shows systems within a spatial radius but lacks:

1. **Fuel projections**: Users cannot see how much fuel is needed to visit each system
2. **Heat projections**: Users cannot see heat accumulation across the scouting run
3. **Route ordering**: Systems are shown by distance from origin, not in an optimal visiting order
4. **Cumulative tracking**: No running totals for fuel consumed or heat generated

Without this information, pilots must manually calculate fuel requirements or run separate `route`
commands for each destination, which is cumbersome for exploration planning.

## Requirements

### Functional Requirements

1. **FR-1**: Add `--ship <NAME>` option to `scout range` to enable fuel/heat calculations
2. **FR-2**: Add `--fuel-quality <PERCENT>` option (default: 10%)
3. **FR-3**: Add `--cargo-mass <KG>` option for fuel calculation accuracy
4. **FR-4**: Add `--fuel-load <UNITS>` option to specify starting fuel
5. **FR-5**: When ship is specified, calculate optimal visiting order using nearest-neighbor heuristic
6. **FR-6**: Show per-hop fuel cost, cumulative fuel consumed, and remaining fuel
7. **FR-7**: Show per-hop heat generated and cumulative heat
8. **FR-8**: Show cooldown time when heat exceeds critical threshold
9. **FR-9**: Display REFUEL warning when fuel would be insufficient for a hop
10. **FR-10**: Display OVERHEATED warning when heat exceeds ship tolerance

### Non-Functional Requirements

1. **NFR-1**: Nearest-neighbor heuristic must complete in <100ms for up to 100 systems
2. **NFR-2**: Reuse existing fuel/heat calculation code from `evefrontier-lib`
3. **NFR-3**: Output format must match `route` command's enhanced format for consistency

## Behavioral Notes

### Scout Gates (FR-0)

Gate travel does not consume fuel (per game mechanics), so `scout gates` will **not** include
fuel/heat projections even when `--ship` is specified. This is intentional.

### Hamiltonian Path Ordering

When `--ship` is specified, the output order changes from "sorted by distance from origin" to
"optimal visiting order". The algorithm:

1. Start at origin system
2. Find nearest unvisited system (nearest-neighbor heuristic)
3. Move to that system, calculate fuel/heat for the hop
4. Repeat until all systems visited
5. Optionally return to origin (if `--return-to-origin` flag added later)

This is a greedy approximation, not a globally optimal solution (TSP is NP-hard), but provides
reasonable results for typical scouting scenarios (5-20 systems).

### Fuel/Heat Calculation

Reuse existing library functions:
- `calculate_jump_fuel_cost(distance_ly, total_mass_kg, fuel_quality)`
- `calculate_jump_heat(distance_ly, specific_heat, base_heat)`

Mass decreases as fuel is consumed (dynamic mass mode).

## User Interface

### Command Syntax

```bash
# Without ship - current behavior (sorted by distance, no fuel/heat)
evefrontier-cli scout range "Nod" --limit 10

# With ship - Hamiltonian path with fuel/heat projections
evefrontier-cli scout range "Nod" --limit 10 --ship Reflex
evefrontier-cli scout range "Nod" --limit 10 --ship Reflex --fuel-quality 15
evefrontier-cli scout range "Nod" --radius 50 --ship Reflex --cargo-mass 5000
```

### Output Example (Enhanced Format with Ship)

```
Systems in range of A3V-125 (10 found):
  (radius 50.0 ly, limit 10, ship: Reflex)

Route from A3V-125 visiting 10 systems:
  1. ● I4C-2T4 (17.5 ly)   3 Planets 10 Moons 
       │ min   5.48K, fuel  89 (rem 1661), heat  12.34
  2. ● O1J-P35 (12.3 ly)   1 Planet 1 Moon 
       │ min  89.58K, fuel  63 (rem 1598), heat  21.05
  3. ● EHN-655 (8.7 ly)   7 Planets 
       │ min   0.26K, fuel  44 (rem 1554), heat  26.89
  ...
 10. ● O8D-RB5 (15.2 ly)   4 Planets 19 Moons 
       │ min   5.06K, fuel  77 (rem 1102), heat  58.23

───────────────────────────────────────
  Total Distance:        142 ly
  Fuel (Reflex):         648 (37% of capacity)
  Remaining:           1,102
  Total Wait:             0s
  Final Heat:          58.23
```

### JSON Output

```json
{
  "system": "A3V-125",
  "system_id": 30000191,
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
  "total_fuel": 648,
  "final_heat": 58.23,
  "systems": [
    {
      "name": "I4C-2T4",
      "id": 30000192,
      "distance_ly": 17.5,
      "hop_fuel": 89,
      "cumulative_fuel": 89,
      "remaining_fuel": 1661,
      "hop_heat": 12.34,
      "cumulative_heat": 12.34,
      "min_temp_k": 5.48,
      "planet_count": 3,
      "moon_count": 10
    }
  ]
}
```

## Success Criteria

1. `scout range --ship Reflex` shows fuel/heat projections in visiting order
2. Output format matches route command's enhanced style
3. REFUEL/OVERHEATED warnings display when thresholds exceeded
4. Nearest-neighbor algorithm produces reasonable visiting order
5. JSON output includes all fuel/heat fields
6. Tests cover fuel calculation accuracy and edge cases

## Out of Scope

1. Globally optimal TSP solution (NP-hard; nearest-neighbor heuristic is sufficient)
2. Return-to-origin option (could be added in future iteration)
3. Fuel/heat for `scout gates` (gate travel is free)
4. Multi-hop route planning to reach distant systems (use `route` command instead)

## Implementation Notes

### Algorithm Pseudocode

```
function plan_scout_route(origin, systems, ship):
    current = origin
    unvisited = systems.copy()
    route = []
    fuel_remaining = ship.fuel_capacity
    heat_current = 30.0  # baseline
    
    while unvisited not empty:
        # Find nearest unvisited system
        nearest = min(unvisited, key=distance(current, s))
        
        # Calculate hop metrics
        hop_distance = distance(current, nearest)
        hop_fuel = calculate_fuel(hop_distance, ship.mass, fuel_quality)
        hop_heat = calculate_heat(hop_distance, ship.specific_heat)
        
        # Update state
        fuel_remaining -= hop_fuel
        heat_current += hop_heat
        
        # Add to route with projections
        route.append({
            system: nearest,
            distance: hop_distance,
            hop_fuel: hop_fuel,
            remaining_fuel: fuel_remaining,
            hop_heat: hop_heat,
            cumulative_heat: heat_current
        })
        
        # Move to nearest
        current = nearest
        unvisited.remove(nearest)
    
    return route
```

### Files to Modify

1. `crates/evefrontier-cli/src/main.rs` - Add ship args to `ScoutRangeArgs`
2. `crates/evefrontier-cli/src/commands/scout.rs` - Implement route planning logic
3. `crates/evefrontier-cli/src/output_helpers.rs` - Add fuel/heat formatting for scout
4. `crates/evefrontier-lib/src/ship.rs` - Expose fuel/heat calculation functions if needed
