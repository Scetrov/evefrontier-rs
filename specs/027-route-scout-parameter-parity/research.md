# Research: Route & Scout Parameter Parity

**Feature**: 027-route-scout-parameter-parity  
**Phase**: 0 (Research & Unknowns Resolution)  
**Date**: 2026-02-03

## R1: Clap `#[command(flatten)]` Pattern

### Question
How does clap handle flattened argument structs with conflicting defaults or help text?

### Investigation

#### Existing Usage in Codebase
The `evefrontier-cli` crate already uses `#[command(flatten)]` successfully in `main.rs`:

```rust
#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    global: GlobalOptions,  // ← Already flattened

    #[command(subcommand)]
    command: Command,
}
```

**GlobalOptions** contains:
- `--data-dir`
- `--dataset`
- `--format`
- `--no-logo`
- `--no-color`
- `--no-footer`
- `--fmap-base-url`

All marked with `global = true` attribute, meaning they apply to all subcommands.

#### Clap Documentation Review
From clap 4.x documentation:
- **Flatten behavior**: Arguments from flattened structs are merged into the parent struct's help text
- **Conflict resolution**: Clap throws **compile-time errors** if two arguments have the same `long` name
- **Help text ordering**: Flattened arguments appear in the order they're declared in the parent struct
- **Default values**: Each struct's `#[arg(default_value = "...")]` is preserved independently
- **Grouping**: Can use `#[arg(help_heading = "Section Name")]` to group related flags in help output

#### Test Results

**Test Case 1: Flatten with Defaults**
```rust
#[derive(Args, Debug)]
struct SharedConfig {
    #[arg(long, default_value = "10")]
    quality: i64,
}

#[derive(Args, Debug)]
struct MyCommand {
    #[command(flatten)]
    shared: SharedConfig,
    
    #[arg(long, default_value = "5")]
    limit: usize,
}
```

**Result**: ✅ **WORKS**. Both defaults are preserved. Help text shows:
```
--quality <QUALITY>  [default: 10]
--limit <LIMIT>      [default: 5]
```

**Test Case 2: Multiple Flattened Structs**
```rust
#[derive(Args, Debug)]
struct RouteArgs {
    #[command(flatten)]
    constraints: CommonRouteConstraints,
    
    #[command(flatten)]
    ship: CommonShipConfig,
    
    #[command(flatten)]
    heat: CommonHeatConfig,
}
```

**Result**: ✅ **WORKS** as long as no argument names conflict. Clap merges all arguments into the parent command's help text.

**Test Case 3: Help Text Ordering**
Flattened arguments appear **after** the parent struct's own arguments in help output. Within flattened groups, order matches the struct field declaration order.

### Findings

#### Best Practices
1. **Use help headings** to organize flattened arguments:
   ```rust
   #[derive(Args, Debug, Clone)]
   pub struct CommonRouteConstraints {
       #[arg(long = "max-jump", help_heading = "Routing Constraints")]
       pub max_jump: Option<f64>,
       
       #[arg(long = "avoid", help_heading = "Routing Constraints")]
       pub avoid: Vec<String>,
   }
   ```

2. **Namespace carefully**: Ensure no two flattened structs use the same argument names (compile-time error otherwise)

3. **Document shared structs**: Add Rustdoc comments explaining that these structs are shared across multiple commands

4. **Test help output**: Use `cargo run -- <command> --help` to verify help text readability

#### Edge Cases
- **Conflicting names**: Clap will fail to compile if two flattened structs define `--foo`
- **Global flags**: Flattened structs in subcommands **cannot** use `global = true` (only root-level flattened structs can)
- **Value parsers**: Custom `value_parser` functions work correctly with flattened structs

### Decision
✅ **Use `#[command(flatten)]` for shared argument structs**. This pattern:
- Reduces code duplication (DRY principle)
- Ensures parameter consistency across commands
- Leverages clap's compile-time safety (conflicts detected early)
- Preserves help text clarity with `help_heading` annotations

### Alternatives Considered
❌ **Macros**: Too complex; clap's derive macros already handle code generation  
❌ **Trait-based composition**: Clap doesn't support trait-based argument definitions  
❌ **Copy-paste duplication**: Violates DRY; error-prone for future parameter additions

---

## R2: Backward Compatibility Strategy

### Question
How to ensure new parameters don't break existing scripts or automation?

### Analysis

#### Current Default Behaviors

**Route Command (Existing)**:
- `--algorithm`: Defaults to `dijkstra`
- `--max-jump`: No default (unrestricted if not specified)
- `--avoid`: Empty list by default
- `--avoid-gates`: `false` by default
- `--max-temp`: No default (unrestricted if not specified)
- `--ship`: `None` by default (no fuel projection)
- `--fuel-quality`: `10` by default
- `--dynamic-mass`: `false` by default
- `--optimize`: Defaults to `distance` if not specified

**Scout Range (Existing)**:
- `--limit`: `10` by default
- `--radius`: No default (required for spatial queries)
- `--max-temp`: No default (unrestricted if not specified)
- `--ship`: `None` by default
- `--fuel-quality`: `10.0` by default
- `--sys-temp-curve`: `Flux` by default

#### New Parameters for Scout Range

| Parameter | Default Value | Backward Compat Impact |
|-----------|---------------|------------------------|
| `--algorithm` | `dijkstra` | ✅ No impact (matches route default) |
| `--max-jump` | `None` (unrestricted) | ✅ No impact (existing behavior: no restriction) |
| `--avoid` | `[]` (empty) | ✅ No impact (existing behavior: no avoidance) |
| `--avoid-gates` | `false` | ✅ No impact (existing behavior: gates allowed) |
| `--avoid-critical-state` | `false` | ✅ No impact (existing behavior: no heat constraint) |
| `--no-avoid-critical-state` | `false` | ✅ No impact (explicit disable; default is already disabled) |
| `--dynamic-mass` | `false` | ✅ No impact (existing behavior: static mass) |
| `--optimize` | `distance` | ✅ No impact (matches route default) |
| `--max-spatial-neighbours` | `250` | ✅ No impact (matches route default) |

#### Verification Tests

**Test 1: Old Invocation Still Works**
```bash
# OLD (before parameter parity)
evefrontier-cli scout range Nod -r 50

# AFTER (with parameter parity)
evefrontier-cli scout range Nod -r 50
# Expected: Identical output (same systems, same distances, same order)
```

**Test 2: Output Format Unchanged**
```bash
# OLD
evefrontier-cli scout range Nod -r 50 --format json

# AFTER
evefrontier-cli scout range Nod -r 50 --format json
# Expected: JSON schema identical (no new fields unless --ship specified)
```

**Test 3: Error Messages Unchanged**
```bash
# OLD (invalid system name)
evefrontier-cli scout range "INVALID" -r 50
# Expected: Same fuzzy suggestion behavior

# AFTER (invalid system name)
evefrontier-cli scout range "INVALID" -r 50
# Expected: Same fuzzy suggestion behavior
```

### Findings

#### Migration Matrix

| Old Invocation | New Equivalent | Behavior Change |
|----------------|----------------|-----------------|
| `scout range Nod -r 50` | `scout range Nod -r 50` | ✅ **Identical** (all new params use backward-compat defaults) |
| `scout range Nod -r 50 --ship Reflex` | `scout range Nod -r 50 --ship Reflex` | ✅ **Identical** (fuel projection unchanged) |
| `scout range Nod -r 50 --max-temp 8000` | `scout range Nod -r 50 --max-temp 8000` | ✅ **Identical** (temperature filter preserved) |

#### Breaking Change Risk: **ZERO**

All new parameters are **additive**:
- Default values match "parameter not specified" behavior
- No existing parameter removed or renamed
- No changes to output schema unless new parameters explicitly used

### Decision
✅ **100% backward compatible**. All new parameters default to existing behavior. Users must **opt-in** to new features by specifying flags explicitly.

### Migration Guide (for quickstart.md)
No migration required. Existing scripts will continue to work unchanged. New features are opt-in.

---

## R3: Help Text Organization

### Question
Will adding many new parameters to `scout range --help` overwhelm users?

### Analysis

#### Current Help Text Length

**Route Command**: ~35 lines of parameter descriptions  
**Scout Range (Before)**: ~15 lines of parameter descriptions  
**Scout Range (After)**: ~35 lines (matches route)

#### Help Heading Strategy

Use clap's `help_heading` attribute to group related parameters:

```rust
#[derive(Args, Debug, Clone)]
pub struct CommonRouteConstraints {
    #[arg(long = "max-jump", help_heading = "ROUTING CONSTRAINTS")]
    pub max_jump: Option<f64>,
    
    #[arg(long = "avoid", help_heading = "ROUTING CONSTRAINTS")]
    pub avoid: Vec<String>,
    
    #[arg(long = "avoid-gates", help_heading = "ROUTING CONSTRAINTS")]
    pub avoid_gates: bool,
    
    #[arg(long = "max-temp", help_heading = "ROUTING CONSTRAINTS")]
    pub max_temp: Option<f64>,
}

#[derive(Args, Debug, Clone)]
pub struct CommonShipConfig {
    #[arg(long = "ship", help_heading = "SHIP & FUEL")]
    pub ship: Option<String>,
    
    #[arg(long = "fuel-quality", help_heading = "SHIP & FUEL")]
    pub fuel_quality: f64,
    
    #[arg(long = "cargo-mass", help_heading = "SHIP & FUEL")]
    pub cargo_mass: f64,
    
    #[arg(long = "fuel-load", help_heading = "SHIP & FUEL")]
    pub fuel_load: Option<f64>,
    
    #[arg(long = "dynamic-mass", help_heading = "SHIP & FUEL")]
    pub dynamic_mass: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CommonHeatConfig {
    #[arg(long = "avoid-critical-state", help_heading = "HEAT MECHANICS")]
    pub avoid_critical_state: bool,
    
    #[arg(long = "no-avoid-critical-state", help_heading = "HEAT MECHANICS")]
    pub no_avoid_critical_state: bool,
    
    #[arg(long = "sys-temp-curve", help_heading = "HEAT MECHANICS")]
    pub sys_temp_curve: TemperatureCurveArg,
}
```

#### Proposed Help Text Sections

```
NAVIGATION:
  <SYSTEM>                     System name to query
  -r, --radius <RADIUS>        Maximum distance in light-years
  -n, --limit <LIMIT>          Maximum number of results [default: 10]

ROUTING CONSTRAINTS:
  --max-jump <MAX_JUMP>        Maximum jump distance (light-years)
  --avoid <AVOID>              Systems to avoid when building paths
  --avoid-gates                Avoid gates entirely (spatial routes only)
  --max-temp <MAX_TEMP>        Maximum system temperature threshold in Kelvin

SHIP & FUEL:
  --ship <SHIP>                Ship name for fuel projection
  --fuel-quality <QUALITY>     Fuel quality rating (1-100) [default: 10]
  --cargo-mass <MASS>          Cargo mass in kilograms [default: 0]
  --fuel-load <LOAD>           Initial fuel load in units
  --dynamic-mass               Recalculate mass after each hop

HEAT MECHANICS:
  --avoid-critical-state       Reject jumps reaching critical temperature (≥150K)
  --no-avoid-critical-state    Disable temperature constraints
  --sys-temp-curve <CURVE>     Temperature model: flux or logistic [default: flux]

OPTIMIZATION:
  --algorithm <ALGORITHM>      Algorithm: bfs, dijkstra, a-star [default: dijkstra]
  --optimize <OBJECTIVE>       Optimize for distance or fuel
  --max-spatial-neighbours <N> Max spatial neighbours to consider [default: 250]

FILTERS:
  --include-ccp-systems        Include CCP developer/staging systems (AD###, V-###)
```

### Findings

#### Readability Improvements
- **Grouped parameters**: Logical sections reduce cognitive load
- **Consistent ordering**: Navigation → Constraints → Ship → Heat → Optimization → Filters
- **Clear headings**: UPPERCASE section names improve scannability
- **Inline defaults**: Users see default values immediately

#### Comparison to Similar CLIs
- **Git**: Uses similar section grouping (`--help` output organized by topic)
- **Docker**: Groups flags by category (networking, volumes, etc.)
- **Kubectl**: Uses help headings extensively for complex commands

### Decision
✅ **Use help headings** to organize parameters into logical sections. This:
- Improves help text readability
- Reduces cognitive overload from long parameter lists
- Follows CLI best practices (git, docker, kubectl precedent)
- Automatically generated by clap (no manual formatting required)

### Alternatives Considered
❌ **No headings**: Help text becomes a wall of text (hard to scan)  
❌ **Separate `--help-advanced`**: Clap doesn't support this natively; users expect all options in `--help`  
❌ **Man pages**: Out of scope; CLI-focused project doesn't justify man page infrastructure

---

## Summary

### Key Decisions
1. ✅ **Use `#[command(flatten)]`** for shared argument structs (R1)
2. ✅ **100% backward compatible** — all new parameters use existing behavior as defaults (R2)
3. ✅ **Use help headings** to organize parameters into logical sections (R3)

### Best Practices Identified
- Namespace shared structs carefully to avoid argument name conflicts
- Use `help_heading` annotations for all shared parameter groups
- Write comprehensive CLI integration tests verifying backward compatibility
- Document shared structs with Rustdoc comments explaining their purpose

### Risks Mitigated
- ✅ No compile-time conflicts (clap enforces uniqueness)
- ✅ No runtime regressions (default values preserve existing behavior)
- ✅ No user confusion (help text organized by topic)

### Next Steps
Proceed to **Phase 1** (Design) to create:
- `data-model.md` — Complete struct definitions with Rustdoc comments
- `quickstart.md` — Migration guide and usage examples
- Update agent context with shared argument pattern
