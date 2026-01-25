# Research: Scout Fuel and Heat Projection

**Spec**: 026 | **Date**: 2026-01-24

## Research Tasks

### 1. Nearest-Neighbor Heuristic for Hamiltonian Path

**Decision**: Use greedy nearest-neighbor algorithm starting from origin system.

**Rationale**: 
- TSP is NP-hard; exact solutions are impractical for interactive CLI
- Nearest-neighbor is O(n²) and completes in <1ms for 100 systems
- Produces reasonable results for typical scouting scenarios (compact clusters)
- Same approach used in real-world logistics and game pathfinding

**Alternatives Considered**:
- 2-opt improvement: Rejected (adds complexity; marginal benefit for 5-20 systems)
- Christofides algorithm: Rejected (overkill for CLI tool; requires MST implementation)
- Branch-and-bound: Rejected (exponential time for n>15)

**Implementation**:
```rust
fn nearest_neighbor_order(origin: i64, systems: &[RangeNeighbor]) -> Vec<RangeNeighbor> {
    let mut current = origin;
    let mut unvisited: Vec<_> = systems.to_vec();
    let mut ordered = Vec::with_capacity(systems.len());
    
    while !unvisited.is_empty() {
        // Find nearest to current position
        let (idx, _) = unvisited.iter().enumerate()
            .min_by(|(_, a), (_, b)| {
                distance(current, a.id).partial_cmp(&distance(current, b.id)).unwrap()
            })
            .unwrap();
        
        let next = unvisited.remove(idx);
        current = next.id;
        ordered.push(next);
    }
    ordered
}
```

### 2. Existing Fuel/Heat Calculation Functions

**Decision**: Reuse `calculate_jump_fuel_cost` and `calculate_jump_heat` from `evefrontier-lib/src/ship.rs`.

**Rationale**:
- Functions already validated and tested (ADR 0015)
- Consistent with route command's projections
- Avoids code duplication

**Key Functions**:

| Function | Signature | Formula |
|----------|-----------|---------|
| `calculate_jump_fuel_cost` | `(total_mass_kg, distance_ly, &FuelConfig) -> Result<f64>` | `(mass / 100_000) × (quality / 100) × distance` |
| `calculate_jump_heat` | `(total_mass_kg, distance_ly, hull_mass_kg, calibration) -> Result<f64>` | `(3 × mass × distance) / (calibration × hull)` |

**Dynamic Mass**: When `fuel_config.dynamic_mass = true`, mass decreases as fuel is consumed.

### 3. Integration with Existing Ship Catalog

**Decision**: Use `ShipCatalog::load()` to resolve ship name to `ShipAttributes`.

**Rationale**:
- Already implemented for route command
- Handles fuzzy matching and validation
- Provides all required attributes (mass, fuel_capacity, specific_heat)

**Usage Pattern**:
```rust
let catalog = ShipCatalog::load(&data_dir)?;
let ship = catalog.get(&args.ship)?;
let loadout = ShipLoadout::new(ship.clone())
    .with_fuel_load(args.fuel_load.unwrap_or(ship.fuel_capacity))
    .with_cargo_mass(args.cargo_mass.unwrap_or(0.0));
```

### 4. Output Format Alignment with Route Command

**Decision**: Mirror route command's enhanced format for fuel/heat columns.

**Rationale**:
- NFR-3 requires consistency
- Users familiar with route output will recognize the format
- Reduces cognitive load

**Format Reference** (from output_helpers.rs):
```
  1. ● SystemName (17.5 ly)   3 Planets 10 Moons 
       │ min   5.48K, fuel  89 (rem 1661), heat  12.34
```

### 5. Warning Thresholds

**Decision**: Reuse existing threshold constants from ship.rs.

| Threshold | Value | Warning |
|-----------|-------|---------|
| `HEAT_OVERHEATED` | 90.0 | `⚠ OVERHEATED` |
| `HEAT_CRITICAL` | 150.0 | `🔥 CRITICAL` |
| Fuel < hop cost | - | `⚠ REFUEL` |

### 6. Edge Cases

**Decision**: Handle gracefully with clear error messages.

| Case | Behavior |
|------|----------|
| Unknown ship name | Error with fuzzy suggestions |
| 0 systems in range | Return empty result (no route) |
| 1 system in range | Single-hop route (trivial) |
| Insufficient fuel mid-route | Show REFUEL warning, continue planning |
| Heat exceeds critical | Show CRITICAL warning, calculate cooldown |

## Summary

All NEEDS CLARIFICATION items resolved. Implementation can proceed with:
1. Add ship CLI options to `ScoutRangeArgs`
2. Implement nearest-neighbor ordering in `scout.rs`
3. Add fuel/heat projection to output structs
4. Extend formatters for fuel/heat columns
5. Write tests covering algorithm and edge cases
