# Quickstart: Route & Scout Parameter Parity

**Feature**: 027-route-scout-parameter-parity  
**Phase**: 1 (Design)  
**Date**: 2026-02-03  
**Audience**: End users and automation script maintainers

## What's New

The `scout range` command now supports **all routing parameters** from the `route` command, ensuring consistent behavior across both commands. This means you can now use avoidance constraints, heat mechanics, fuel optimization, and more when scouting nearby systems.

### New Parameters Available in `scout range`

| Parameter | Purpose | Example |
|-----------|---------|---------|
| `--avoid` | Exclude hostile/dangerous systems | `--avoid "Brana" --avoid "H:2L2S"` |
| `--avoid-gates` | Find spatial-only routes (excludes all gate-connected systems) | `--avoid-gates` |
| `--avoid-critical-state` | Enable heat-aware routing | `--avoid-critical-state` |
| `--no-avoid-critical-state` | Disable temperature constraints | `--no-avoid-critical-state` |
| `--dynamic-mass` | Recalculate mass per hop as fuel burns | `--dynamic-mass` |

### Existing Parameters (No Changes)

These parameters already existed in `scout range` and continue to work identically:

- `<SYSTEM>` — Origin system name
- `-r, --radius` — Maximum spatial range in light-years
- `-n, --limit` — Maximum number of results
- `-t, --max-temp` — Maximum system temperature filter
- `--ship` — Ship name for fuel/heat projection
- `--fuel-quality` — Fuel quality rating (1-100)
- `--cargo-mass` — Cargo mass in kilograms
- `--fuel-load` — Starting fuel load
- `--sys-temp-curve` — Temperature calculation model
- `--include-ccp-systems` — Include CCP staging systems
- `--format`, `--no-logo`, `--no-footer`, etc. — Output formatting (global flags)

---

## Migration Guide

### For End Users

**Good news**: If you're using `scout range` with existing parameters, **nothing changes**. All new parameters are optional and default to the same behavior as before.

#### Before (Scout Range Without New Features)

```bash
# Basic spatial scout
evefrontier-cli scout range Nod -r 50

# With fuel projection
evefrontier-cli scout range Nod -r 80 --ship Reflex --fuel-quality 50

# With temperature filter
evefrontier-cli scout range Nod -r 100 --max-temp 8000
```

#### After (Same Commands Still Work)

```bash
# Basic spatial scout (IDENTICAL OUTPUT)
evefrontier-cli scout range Nod -r 50

# With fuel projection (IDENTICAL OUTPUT)
evefrontier-cli scout range Nod -r 80 --ship Reflex --fuel-quality 50

# With temperature filter (IDENTICAL OUTPUT)
evefrontier-cli scout range Nod -r 100 --max-temp 8000
```

**No migration required.** Existing scripts continue to work without modification.

---

## New Usage Examples

### Example 1: Avoidance Constraints

Scout systems within 50 light-years of Nod, but **avoid hostile systems** Brana and H:2L2S:

```bash
evefrontier-cli scout range Nod -r 50 --avoid "Brana" --avoid "H:2L2S"
```

**Use Case**: Planning exploration routes while avoiding known pirate-controlled systems.

---

### Example 2: Fuel Optimization

Find the 10 nearest systems to Nod, ordered by **fuel-efficient traversal** using a Reflex ship with dynamic mass recalculation:

```bash
evefrontier-cli scout range Nod -r 80 --ship Reflex --optimize fuel --dynamic-mass -n 10
```

**Output Changes**:
- Systems ordered by cumulative fuel cost (not just distance)
- Route steps show fuel consumption per hop
- Summary includes total fuel required

**Use Case**: Planning scouting expeditions with limited fuel capacity.

---

### Example 3: Heat-Aware Routing

Scout systems within 100 light-years, but **reject jumps that would overheat the ship**:

```bash
evefrontier-cli scout range Nod -r 100 --avoid-critical-state --sys-temp-curve flux
```

**Behavior**:
- Systems reachable via high-heat spatial jumps are excluded
- Gate-connected systems unaffected by heat constraints
- Uses inverse-tangent heat signature model (flux curve)

**Use Case**: Safe exploration planning for ships with low heat tolerance.

---

### Example 4: Spatial-Only Routes

Find systems **only reachable by spatial jumps** (no gate travel):

```bash
evefrontier-cli scout range Nod -r 50 --avoid-gates
```

**Behavior**:
- Gate-connected neighbors excluded from results
- Only shows systems within direct spatial jump range
- Useful for uncharted space exploration

**Use Case**: Finding hidden systems not connected to the gate network.

---

### Example 5: Algorithm Selection

Use **A\* pathfinding** for scout range planning (faster for large search spaces):

```bash
evefrontier-cli scout range Nod -r 200 --algorithm a-star --max-spatial-neighbours 100
```

**Performance Impact**:
- A\* uses heuristics to prune search space (faster than Dijkstra)
- `--max-spatial-neighbours` limits fan-out for performance
- Trade-off: May miss some optimal routes in dense clusters

**Use Case**: Large-radius scouting in dense system clusters.

---

## Parameter Consistency Across Commands

All shared parameters now behave **identically** in `route` and `scout range`:

| Parameter | Route Behavior | Scout Range Behavior | Consistent? |
|-----------|----------------|----------------------|-------------|
| `--max-jump` | Limits spatial jump distance | Limits spatial jump distance | ✅ Yes |
| `--avoid` | Excludes systems from path | Excludes systems from results | ✅ Yes |
| `--avoid-gates` | Prefers spatial routes | Shows only spatial-reachable systems | ✅ Yes |
| `--max-temp` | Filters hot systems (spatial only) | Filters hot systems (spatial only) | ✅ Yes |
| `--ship` | Enables fuel projection | Enables fuel projection | ✅ Yes |
| `--fuel-quality` | Affects fuel consumption formula | Affects fuel consumption formula | ✅ Yes |
| `--dynamic-mass` | Recalculates mass per hop | Recalculates mass per hop | ✅ Yes |
| `--optimize` | Chooses distance/fuel objective | Chooses distance/fuel objective | ✅ Yes |

---

## For Developers

### Using Shared Argument Structs

All shared parameters are now defined in `crates/evefrontier-cli/src/common_args.rs`:

```rust
use crate::common_args::{CommonRouteConstraints, CommonShipConfig, CommonHeatConfig};

#[derive(Args, Debug, Clone)]
pub struct MyNewCommand {
    #[command(flatten)]
    constraints: CommonRouteConstraints,
    
    #[command(flatten)]
    ship: CommonShipConfig,
    
    #[command(flatten)]
    heat: CommonHeatConfig,
    
    // Command-specific flags
    #[arg(long)]
    my_custom_flag: bool,
}
```

### Adding New Shared Parameters

1. **Determine the correct struct**:
   - **Routing constraints** → `CommonRouteConstraints`
   - **Ship/fuel config** → `CommonShipConfig`
   - **Heat mechanics** → `CommonHeatConfig`

2. **Add the parameter** with appropriate annotations:
   ```rust
   #[arg(long = "new-param", help_heading = "ROUTING CONSTRAINTS")]
   pub new_param: Option<String>,
   ```

3. **Update conversion logic** in command handlers to pass the new parameter to library APIs.

4. **Write tests** verifying the parameter works identically in `route` and `scout range`.

---

## Test Patterns for Parameter Parity

### Integration Test Template

```rust
#[test]
fn test_parameter_parity_avoid() {
    // Test that --avoid works identically in route and scout range
    
    let route_cmd = Command::cargo_bin("evefrontier-cli")
        .unwrap()
        .args(&["route", "--from", "Nod", "--to", "Brana", "--avoid", "H:2L2S"])
        .assert()
        .success();
    
    let scout_cmd = Command::cargo_bin("evefrontier-cli")
        .unwrap()
        .args(&["scout", "range", "Nod", "-r", "50", "--avoid", "H:2L2S"])
        .assert()
        .success();
    
    // Verify both commands exclude H:2L2S from results
    let route_output = route_cmd.get_output().stdout;
    let scout_output = scout_cmd.get_output().stdout;
    
    assert!(!String::from_utf8_lossy(&route_output).contains("H:2L2S"));
    assert!(!String::from_utf8_lossy(&scout_output).contains("H:2L2S"));
}
```

### Backward Compatibility Test Template

```rust
#[test]
fn test_backward_compat_scout_range() {
    // Verify old invocations produce identical output
    
    let old_behavior = Command::cargo_bin("evefrontier-cli")
        .unwrap()
        .args(&["scout", "range", "Nod", "-r", "50", "--format", "json"])
        .assert()
        .success();
    
    // Parse JSON output and verify schema unchanged
    let json: serde_json::Value = serde_json::from_slice(&old_behavior.get_output().stdout).unwrap();
    
    assert!(json["systems"].is_array());
    assert!(json["origin"].is_string());
    assert!(json["radius_ly"].is_number());
}
```

---

## Help Text Changes

### Before (Scout Range Help)

```
Find systems within spatial range of a system

USAGE:
    evefrontier-cli scout range [OPTIONS] <SYSTEM>

ARGUMENTS:
    <SYSTEM>    System name to query

OPTIONS:
    -r, --radius <RADIUS>        Maximum distance in light-years
    -n, --limit <LIMIT>          Maximum number of results [default: 10]
    -t, --max-temp <MAX_TEMP>    Maximum star temperature in Kelvin
    --ship <SHIP>                Ship name for fuel/heat projections
    --fuel-quality <QUALITY>     Fuel quality rating [default: 10.0]
    --include-ccp-systems        Include CCP systems
```

### After (Scout Range Help with Organized Sections)

```
Find systems within spatial range of a system

USAGE:
    evefrontier-cli scout range [OPTIONS] <SYSTEM>

ARGUMENTS:
    <SYSTEM>    System name to query

NAVIGATION:
    -r, --radius <RADIUS>         Maximum distance in light-years
    -n, --limit <LIMIT>           Maximum number of results [default: 10]

ROUTING CONSTRAINTS:
    --max-jump <MAX_JUMP>         Maximum jump distance (light-years)
    --avoid <AVOID>               Systems to avoid when building paths
    --avoid-gates                 Avoid gates entirely (spatial routes only)
    --max-temp <MAX_TEMP>         Maximum system temperature threshold in Kelvin

SHIP & FUEL:
    --ship <SHIP>                 Ship name for fuel projection
    --fuel-quality <QUALITY>      Fuel quality rating (1-100) [default: 10]
    --cargo-mass <MASS>           Cargo mass in kilograms [default: 0]
    --fuel-load <LOAD>            Initial fuel load in units
    --dynamic-mass                Recalculate mass after each hop

HEAT MECHANICS:
    --avoid-critical-state        Reject jumps reaching critical temperature (≥150K)
    --no-avoid-critical-state     Disable temperature constraints
    --sys-temp-curve <CURVE>      Temperature model: flux or logistic [default: flux]

OPTIMIZATION:
    --algorithm <ALGORITHM>       Algorithm: bfs, dijkstra, a-star [default: dijkstra]
    --optimize <OBJECTIVE>        Optimize for distance or fuel
    --max-spatial-neighbours <N>  Max spatial neighbours to consider [default: 250]

FILTERS:
    --include-ccp-systems         Include CCP developer/staging systems (AD###, V-###)
```

**Improvement**: Parameters grouped by purpose; easier to scan and understand.

---

## Troubleshooting

### Q: I'm getting "unknown argument" errors after upgrading

**A**: Make sure you've updated to the latest version:
```bash
evefrontier-cli --version
```

If you're using an old binary, rebuild:
```bash
cargo install --path crates/evefrontier-cli --force
```

---

### Q: My old scripts are failing with new parameters

**A**: This should **never** happen if you're using existing parameters. All new parameters are optional with backward-compatible defaults. If you see failures, please file a bug report.

**Debug steps**:
1. Run with `--help` to verify parameter names
2. Check for typos in flag names (e.g., `--max-temp` not `--maxtemp`)
3. Verify you're passing valid values (e.g., `--fuel-quality` must be 1-100)

---

### Q: Which parameters should I use for [specific use case]?

**Exploration Planning**:
- Use `--avoid` to exclude dangerous systems
- Use `--max-temp` to avoid hot stars
- Use `--algorithm a-star` for large search spaces

**Fuel-Efficient Scouting**:
- Use `--ship` + `--optimize fuel`
- Add `--dynamic-mass` for accurate long-range projections
- Use `--fuel-load` to simulate partial fuel tanks

**Heat-Aware Routing**:
- Use `--avoid-critical-state` to prevent overheating
- Use `--sys-temp-curve flux` (default; physically accurate)
- Use `--ship` to get ship-specific heat tolerance

**Spatial-Only Exploration**:
- Use `--avoid-gates` to find uncharted systems
- Use `--max-jump` to limit warp drive range
- Use `--radius` to define search area

---

## Related Documentation

- **Full CLI Reference**: `docs/USAGE.md`
- **Fuel Mechanics**: `docs/HEAT_MECHANICS.md` (heat model formulas)
- **Ship Data**: `data/ship_data.csv` (available ships and stats)
- **ADRs**: `docs/adrs/` (architectural decisions)

---

## Summary

### Key Takeaways
- ✅ **No migration required** — existing scripts continue to work
- ✅ **New features are opt-in** — use new parameters only when needed
- ✅ **Parameter consistency** — identical behavior in `route` and `scout range`
- ✅ **Better help text** — organized by purpose for easier discovery

### Next Steps
1. Explore new parameters with `--help`
2. Try examples from this guide
3. Update automation scripts to use new features (optional)
4. Report bugs or suggestions via GitHub issues
