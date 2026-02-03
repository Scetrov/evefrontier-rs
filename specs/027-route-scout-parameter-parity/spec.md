# Feature Specification: Route & Scout Parameter Parity

**Feature ID:** 027
**Branch:** 027-route-scout-parameter-parity
**Status:** Draft
**Created:** 2026-02-03
**Author:** System (based on user requirements)

## Problem Statement

The `route` and `scout` commands have divergent parameter sets despite performing conceptually similar operations. Both commands plan paths through the EVE Frontier starmap, but expose different subsets of routing constraints, ship parameters, and optimization options. This creates a confusing user experience and forces users to learn different flag sets for similar tasks.

### Current State Analysis

**Route Command Parameters:**
- Navigation: `--from`, `--to`, `--algorithm`, `--max-jump`
- Constraints: `--avoid`, `--avoid-gates`, `--max-temp`, `--avoid-critical-state`, `--no-avoid-critical-state`
- Ship/Fuel: `--ship`, `--fuel-quality`, `--cargo-mass`, `--fuel-load`, `--dynamic-mass`
- Heat: `--sys-temp-curve`
- Optimization: `--optimize`, `--max-spatial-neighbours`
- Output: `--no-temp`

**Scout Gates Parameters:**
- Navigation: `<SYSTEM>` (position argument)
- Filters: `--include-ccp-systems`
- _Missing: All ship, fuel, heat, and optimization parameters_

**Scout Range Parameters:**
- Navigation: `<SYSTEM>` (position argument), `--radius`, `--limit`
- Constraints: `--max-temp`
- Ship/Fuel: `--ship`, `--fuel-quality`, `--cargo-mass`, `--fuel-load`
- Heat: `--sys-temp-curve`
- Filters: `--include-ccp-systems`
- _Missing: `--avoid`, `--avoid-gates`, `--avoid-critical-state`, `--algorithm`, `--optimize`, `--dynamic-mass`, `--max-jump`_

### Key Inconsistencies

1. **Algorithm Selection:** Route supports BFS/Dijkstra/A*, scout range has no algorithm choice
2. **Avoidance:** Route supports `--avoid` and `--avoid-gates`, scout range has none
3. **Dynamic Mass:** Route supports `--dynamic-mass`, scout range does not
4. **Optimization:** Route supports `--optimize distance|fuel`, scout range has no equivalent
5. **Heat Constraints:** Route has `--avoid-critical-state` / `--no-avoid-critical-state`, scout range lacks this
6. **CCP System Filtering:** Scout commands have `--include-ccp-systems`, route does not

## Proposed Solution

### Goals

1. **Shared Parameter Model:** Create reusable argument structs (`CommonRouteConstraints`, `CommonShipConfig`, `CommonHeatConfig`) in `crates/evefrontier-cli/src/common.rs`
2. **Consistent Naming:** Ensure identical parameters use identical flag names and value formats across all commands
3. **Backward Compatibility:** Preserve existing CLI behavior; new parameters should be additive
4. **Library-First:** Refactor library routing APIs to accept unified constraint structures

### Parameter Mapping

| Parameter Category | Route | Scout Gates | Scout Range | Proposed Action |
|--------------------|-------|-------------|-------------|-----------------|
| **Navigation** | ✓ | ✓ | ✓ | Keep as-is (command-specific) |
| **Algorithm** | ✓ | ✗ | ✗ | Add to scout range (gates is always BFS) |
| **Max Jump** | ✓ | ✗ | ✗ | Add to scout range (validates fuel range) |
| **Avoidance** | ✓ | ✗ | ✗ | Add `--avoid` to scout range |
| **Avoid Gates** | ✓ | ✗ | ✗ | Add to scout range (spatial-only mode) |
| **Max Temp** | ✓ | ✗ | ✓ | Keep (already consistent) |
| **Ship Config** | ✓ | ✗ | ✓ | Keep scout range; add to gates if needed |
| **Fuel Quality** | ✓ | ✗ | ✓ | Keep scout range |
| **Cargo Mass** | ✓ | ✗ | ✓ | Keep scout range |
| **Fuel Load** | ✓ | ✗ | ✓ | Keep scout range |
| **Dynamic Mass** | ✓ | ✗ | ✗ | Add to scout range |
| **Heat Config** | ✓ | ✗ | ✓ | Keep scout range |
| **Avoid Critical** | ✓ | ✗ | ✗ | Add to scout range |
| **Sys Temp Curve** | ✓ | ✗ | ✓ | Keep scout range |
| **Optimization** | ✓ | ✗ | ✗ | Add to scout range |
| **Max Neighbors** | ✓ | ✗ | ✗ | Add to scout range |
| **CCP Systems** | ✗ | ✓ | ✓ | Add to route |
| **Output Flags** | ✓ | ✓ | ✓ | Keep (already global) |

### Architecture Changes

#### 1. Shared Argument Structs

Create `crates/evefrontier-cli/src/common_args.rs`:

```rust
#[derive(Args, Debug, Clone)]
pub struct CommonRouteConstraints {
    /// Maximum jump distance (light-years)
    #[arg(long = "max-jump")]
    pub max_jump: Option<f64>,
    
    /// Systems to avoid when building the path
    #[arg(long = "avoid")]
    pub avoid: Vec<String>,
    
    /// Avoid gates entirely (prefer spatial routes)
    #[arg(long = "avoid-gates", action = ArgAction::SetTrue)]
    pub avoid_gates: bool,
    
    /// Maximum system temperature threshold in Kelvin
    #[arg(long = "max-temp")]
    pub max_temp: Option<f64>,
}

#[derive(Args, Debug, Clone)]
pub struct CommonShipConfig {
    /// Ship name for fuel projection
    #[arg(long = "ship")]
    pub ship: Option<String>,
    
    /// Fuel quality rating (1-100)
    #[arg(long = "fuel-quality", default_value = "10")]
    pub fuel_quality: i64,
    
    /// Cargo mass in kilograms
    #[arg(long = "cargo-mass", default_value = "0")]
    pub cargo_mass: f64,
    
    /// Initial fuel load (units)
    #[arg(long = "fuel-load")]
    pub fuel_load: Option<f64>,
    
    /// Recalculate mass after each hop as fuel is consumed
    #[arg(long = "dynamic-mass", action = ArgAction::SetTrue)]
    pub dynamic_mass: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CommonHeatConfig {
    /// Heat-aware routing (rejects jumps reaching critical temperature ≥150K)
    #[arg(long = "avoid-critical-state", action = ArgAction::SetTrue)]
    pub avoid_critical_state: bool,
    
    /// Disable temperature constraints
    #[arg(long = "no-avoid-critical-state", action = ArgAction::SetTrue)]
    pub no_avoid_critical_state: bool,
    
    /// Temperature calculation model
    #[arg(long = "sys-temp-curve", value_enum, default_value_t = TemperatureCurveArg::default())]
    pub sys_temp_curve: TemperatureCurveArg,
}
```

#### 2. Updated Command Structs

**RouteCommandArgs:**
```rust
#[derive(Args, Debug, Clone)]
struct RouteCommandArgs {
    #[command(flatten)]
    endpoints: RouteEndpoints,
    
    #[command(flatten)]
    constraints: CommonRouteConstraints,
    
    #[command(flatten)]
    ship: CommonShipConfig,
    
    #[command(flatten)]
    heat: CommonHeatConfig,
    
    /// Algorithm to use when planning the route
    #[arg(long, value_enum, default_value_t = RouteAlgorithmArg::default())]
    algorithm: RouteAlgorithmArg,
    
    /// Include CCP developer/staging systems in routing
    #[arg(long, action = ArgAction::SetTrue)]
    include_ccp_systems: bool,
    
    // ... other route-specific flags
}
```

**ScoutRangeArgs:**
```rust
#[derive(Args, Debug, Clone)]
pub struct ScoutRangeArgs {
    pub system: String,
    
    #[command(flatten)]
    constraints: CommonRouteConstraints,
    
    #[command(flatten)]
    ship: CommonShipConfig,
    
    #[command(flatten)]
    heat: CommonHeatConfig,
    
    /// Algorithm for route planning mode
    #[arg(long, value_enum, default_value_t = RouteAlgorithmArg::Dijkstra)]
    algorithm: RouteAlgorithmArg,
    
    /// Maximum number of results to return (1-100)
    #[arg(long, short = 'n', default_value = "10")]
    limit: usize,
    
    /// Maximum distance in light-years from origin
    #[arg(long, short = 'r')]
    radius: Option<f64>,
    
    /// Include CCP developer/staging systems in results
    #[arg(long, action = ArgAction::SetTrue)]
    include_ccp_systems: bool,
}
```

#### 3. Library API Adjustments

No breaking changes required - library already uses `RouteRequest` and `RouteConstraints` which support all parameters. CLI refactoring is isolated to argument parsing layer.

## User Stories

1. **As a route planner**, I want to use `--avoid` in `scout range` to exclude hostile systems from my scouting radius
2. **As a fuel-conscious pilot**, I want to use `--dynamic-mass` in `scout range` to get accurate fuel projections for multi-hop scouting trips
3. **As a heat-sensitive navigator**, I want `--avoid-critical-state` in `scout range` to prevent recommending dangerous high-heat hops
4. **As a spatial explorer**, I want to use `--avoid-gates` in `scout range` to find systems only reachable by spatial jumps
5. **As a route optimizer**, I want `--optimize fuel` in `scout range` to minimize fuel consumption when visiting multiple systems
6. **As a data analyst**, I want consistent parameter names across all commands so scripts and documentation remain simple

## Success Criteria

1. All shared parameters use identical flag names across `route` and `scout` commands
2. `scout range` supports all applicable routing constraints from `route`
3. All tests pass with backward-compatible behavior
4. Documentation (`docs/USAGE.md`, CLI `--help`) reflects unified parameter model
5. No breaking changes to existing command behavior (additive only)

## Non-Goals

- Changing the fundamental behavior of `route` vs `scout` commands
- Modifying JSON output schemas (backward compatibility required)
- Adding GUI/web interface support (out of scope)

## Dependencies

- Existing routing library APIs (`evefrontier-lib::plan_route`)
- Existing ship/fuel systems (already shared)
- Existing heat mechanics (already shared)

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking CLI backward compatibility | High | Use `#[command(flatten)]` to preserve existing args; add comprehensive tests |
| Confusing help text with too many options | Medium | Group related flags in help output; improve documentation |
| Increased test maintenance burden | Medium | Share test fixtures between route/scout test suites |
| Performance regression from additional validation | Low | Constraint validation already exists in library layer |

## Open Questions

1. Should `scout gates` also support ship/fuel parameters for multi-gate traversal planning?
2. Should we add a `--no-ccp-systems` flag to make CCP filtering consistent across all commands?
3. Should `--limit` in scout range be renamed to `--max-results` for clarity?

## Acceptance Tests

```bash
# Test 1: Scout range with avoidance
evefrontier-cli scout range Nod -r 50 --avoid "Brana" --avoid "H:2L2S"

# Test 2: Scout range with fuel optimization
evefrontier-cli scout range Nod -r 80 --ship Reflex --optimize fuel --dynamic-mass

# Test 3: Scout range with heat constraints
evefrontier-cli scout range Nod -r 100 --avoid-critical-state --sys-temp-curve flux

# Test 4: Route with CCP system filtering
evefrontier-cli route --from Nod --to Brana --include-ccp-systems

# Test 5: Parameter consistency check
evefrontier-cli route --help | grep -E "(--max-jump|--avoid|--ship)"
evefrontier-cli scout range --help | grep -E "(--max-jump|--avoid|--ship)"
# Should show identical parameter descriptions
```
