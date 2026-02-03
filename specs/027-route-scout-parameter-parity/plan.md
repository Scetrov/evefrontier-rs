# Implementation Plan: Route & Scout Parameter Parity

**Branch**: `027-route-scout-parameter-parity` | **Date**: 2026-02-03 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/027-route-scout-parameter-parity/spec.md`

**Note**: This implementation plan follows the EVE Frontier Rust Workspace Constitution v1.1.0 and applies TDD, Library-First Architecture, and Clean Code principles.

## Summary

**Requirement**: Unify parameter sets across `route` and `scout` commands by extracting shared argument structs (`CommonRouteConstraints`, `CommonShipConfig`, `CommonHeatConfig`) and adding missing parameters to `scout range` subcommand.

**Technical Approach**: Refactor CLI argument parsing in `crates/evefrontier-cli/src/main.rs` to use `#[command(flatten)]` for shared parameter groups. Preserve backward compatibility by making all new parameters additive (default values match current behavior). No library API changes required - existing `RouteRequest` and `RouteConstraints` already support all parameters.

## Technical Context

**Language/Version**: Rust 1.93.0 (pinned via `.rust-toolchain`)  
**Primary Dependencies**: clap 4.x (CLI parsing), evefrontier-lib (routing/ship/heat APIs)  
**Storage**: SQLite (dataset loaded from `docs/fixtures/minimal/static_data.db`)  
**Testing**: cargo test with assert_cmd for CLI integration tests  
**Target Platform**: Linux/macOS/Windows (cross-platform CLI binary)  
**Project Type**: Single project (monorepo with Cargo workspace)  
**Performance Goals**: CLI startup <200ms, argument parsing <10ms (negligible impact from shared structs)  
**Constraints**: Zero breaking changes to existing CLI behavior; backward-compatible parameter additions only  
**Scale/Scope**: ~30 CLI parameters across 3 command structs; shared parameter groups reduce duplication from ~60 total lines to ~40 lines

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Test-Driven Development (NON-NEGOTIABLE)
**Status**: ✅ COMPLIANT  
**Plan**: All parameter refactoring will follow Red-Green-Refactor cycle:
1. Write CLI integration tests using `assert_cmd` that verify shared parameters work identically in `route` and `scout range`
2. Implement shared argument structs with `#[command(flatten)]`
3. Refactor to improve readability (extract helper functions for `RouteRequest` conversion)

**Coverage Target**: 80% for CLI integration tests (critical path: argument parsing → library API calls)

### II. Library-First Architecture
**Status**: ✅ COMPLIANT  
**Justification**: No library changes required. All business logic already exists in `evefrontier-lib::plan_route`, `RouteRequest`, and `RouteConstraints`. This is purely a CLI argument parsing refactor.

### III. Architecture Decision Records (Mandatory)
**Status**: ✅ COMPLIANT  
**Plan**: No ADR required for this change. This is a code cleanup/DRY refactor that improves CLI ergonomics without changing algorithms, data models, or architectural boundaries. Per Constitution Section III: "Small bug fixes and UI tweaks do not" require ADRs.

### IV. Clean Code & Cognitive Load
**Status**: ✅ COMPLIANT  
**Improvements**:
- **Reduced duplication**: Extract 3 shared parameter groups (`CommonRouteConstraints`, `CommonShipConfig`, `CommonHeatConfig`) from 2 command structs
- **Single Responsibility**: Each shared struct handles one concern (routing constraints, ship config, heat config)
- **Complexity**: No increase in McCabe complexity; existing logic moves to reusable structs

**Nesting Depth**: No changes to control flow; argument parsing remains flat with `#[command(flatten)]`

### V. Security-First Development
**Status**: ✅ COMPLIANT  
**Validation**: All input validation already exists in:
- `parse_fuel_quality()` — enforces range 1.0-100.0
- `parse_non_negative()` — rejects negative values
- `evefrontier-lib::plan_route()` — validates system names, constraints

**No new security considerations**: This change only reorganizes existing validated parameters.

### VI. Testing Tiers (Aligned with CI)
**Status**: ✅ COMPLIANT  
**Test Plan**:
1. **Unit Tests**: Verify shared struct instantiation and default values
2. **Integration Tests**: CLI smoke tests for `route` and `scout range` with all parameter combinations
3. **CI Checks**: All existing CI checks (format, clippy, build, test, audit) will pass

**Fixture**: Uses existing `docs/fixtures/minimal/static_data.db` for reproducible tests

### VII. Refactoring & Technical Debt Management
**Status**: ✅ COMPLIANT  
**Scope**: This is a focused refactor that extracts shared argument structs without changing behavior. Green tests before and after. Reduces technical debt by eliminating parameter duplication.

**Documentation Updates Required**:
- `docs/USAGE.md` — Add examples showing `scout range` with new parameters
- CLI `--help` output — Automatically updated by clap derives

### Rust Best Practices & Standards
**Status**: ✅ COMPLIANT  
**Toolchain**: Rust 1.93.0 per `.rust-toolchain`  
**Lints**: All code passes `cargo fmt` and `cargo clippy -D warnings`  
**Error Handling**: No changes to error handling; existing `Result<T, Error>` patterns preserved  
**Documentation**: Rustdoc comments will be added to all new shared argument structs

### Development Workflow & Review Process
**Status**: ✅ COMPLIANT  
**Branch**: Created `027-route-scout-parameter-parity` per naming convention  
**PR Requirements**:
- [ ] All tests pass locally and in CI
- [ ] TDD cycle followed (tests → implementation → refactor)
- [ ] `CHANGELOG.md` entry added under Unreleased
- [ ] `docs/USAGE.md` updated with `scout range` parameter examples
- [ ] GPG-signed commits

### Versioning & Release Policy
**Status**: ✅ COMPLIANT  
**Version Impact**: MINOR version bump (new CLI parameters; backward compatible)  
**Release Tag**: Will be tagged as `v0.2.0` when merged (adds new features to `scout range`)

---

**Overall Constitution Compliance**: ✅ **ALL GATES PASSED**  
**Violations Requiring Justification**: None  
**Re-check Required After Phase 1**: No (design is straightforward; no architectural changes)

## Project Structure

### Documentation (this feature)

```text
specs/027-route-scout-parameter-parity/
├── plan.md              # This file
├── spec.md              # Feature specification (completed)
├── research.md          # Phase 0 output (research findings on clap flatten pattern)
├── data-model.md        # Phase 1 output (shared argument struct schemas)
├── quickstart.md        # Phase 1 output (migration guide for users/scripts)
└── contracts/           # N/A (no API contracts; CLI-only change)
```

### Source Code (repository root)

```text
crates/evefrontier-cli/
├── src/
│   ├── main.rs                  # [MODIFIED] Extract shared arg structs to common_args.rs
│   ├── common_args.rs           # [NEW] Shared argument structs (CommonRouteConstraints, CommonShipConfig, CommonHeatConfig)
│   ├── commands/
│   │   ├── route.rs             # [MODIFIED] Use shared args via #[command(flatten)]
│   │   └── scout.rs             # [MODIFIED] Add shared args to ScoutRangeArgs
│   └── output_helpers.rs        # [NO CHANGE] Output formatting preserved
└── tests/
    ├── cli_integration.rs       # [NEW] Integration tests for parameter parity
    ├── route_tests.rs           # [MODIFIED] Add test cases for route with --include-ccp-systems
    └── scout_tests.rs           # [MODIFIED] Add test cases for scout range with new params

docs/
└── USAGE.md                     # [MODIFIED] Add scout range examples with new parameters

CHANGELOG.md                     # [MODIFIED] Add entry under Unreleased
```

**Structure Decision**: Single project structure used (existing Cargo workspace). All changes isolated to `crates/evefrontier-cli` crate. No library changes required (`evefrontier-lib` APIs already support all parameters via `RouteRequest` and `RouteConstraints`).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations requiring justification.** This refactor strictly follows Constitution principles:
- TDD applied (tests first)
- Library-First preserved (no library changes)
- Clean Code improved (reduces duplication)
- Security maintained (existing validation preserved)

---

## Phase 0: Research & Unknowns Resolution

**Objective**: Resolve all NEEDS CLARIFICATION items from Technical Context and identify best practices for clap argument struct composition.

### Research Tasks

#### R1: Clap `#[command(flatten)]` Pattern
**Question**: How does clap handle flattened argument structs with conflicting defaults or help text?  
**Search Strategy**:
- Review clap documentation for `#[command(flatten)]` behavior with repeated arguments
- Check existing codebase for flatten usage patterns (GlobalOptions is already flattened)
- Test if defaults are inherited correctly when structs are composed

**Expected Outcome**: Confirmation that `#[command(flatten)]` merges arguments without conflicts; document any edge cases (e.g., help text ordering).

#### R2: Backward Compatibility Strategy
**Question**: How to ensure new parameters don't break existing scripts or automation?  
**Search Strategy**:
- Identify all default values for new parameters (must match "not specified" behavior)
- Test CLI with old invocations to ensure output format unchanged
- Document migration path in quickstart.md

**Expected Outcome**: Migration matrix showing old invocation → new behavior equivalence.

#### R3: Help Text Organization
**Question**: Will adding many new parameters to `scout range --help` overwhelm users?  
**Search Strategy**:
- Review clap help grouping features (heading annotations)
- Analyze current help text length for route vs scout range
- Design help text groupings: Navigation, Constraints, Ship Config, Heat Config, Filters

**Expected Outcome**: Help text design that groups related parameters under clear headings.

---

**Research Deliverable**: `specs/027-route-scout-parameter-parity/research.md` with:
- Clap flatten pattern best practices
- Backward compatibility verification results
- Help text grouping recommendations

---

## Phase 1: Design & Contracts

**Prerequisites**: `research.md` complete

### D1: Data Model — Shared Argument Structs

**Output**: `specs/027-route-scout-parameter-parity/data-model.md`

#### CommonRouteConstraints
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
```

**Validation Rules**:
- `max_jump`: Must be positive if specified
- `avoid`: Empty list by default; no duplicates
- `avoid_gates`: False by default
- `max_temp`: Must be positive if specified

**State Transitions**: None (stateless configuration struct)

#### CommonShipConfig
```rust
#[derive(Args, Debug, Clone)]
pub struct CommonShipConfig {
    /// Ship name for fuel projection (case-insensitive)
    #[arg(long = "ship")]
    pub ship: Option<String>,
    
    /// Fuel quality rating (1-100)
    #[arg(long = "fuel-quality", default_value = "10", value_parser = parse_fuel_quality)]
    pub fuel_quality: f64,
    
    /// Cargo mass in kilograms
    #[arg(long = "cargo-mass", default_value = "0", value_parser = parse_non_negative)]
    pub cargo_mass: f64,
    
    /// Initial fuel load (units). Defaults to full capacity
    #[arg(long = "fuel-load", value_parser = parse_non_negative)]
    pub fuel_load: Option<f64>,
    
    /// Recalculate mass after each hop as fuel is consumed
    #[arg(long = "dynamic-mass", action = ArgAction::SetTrue)]
    pub dynamic_mass: bool,
}
```

**Validation Rules** (enforced by value parsers):
- `fuel_quality`: Range 1.0-100.0 (validated by `parse_fuel_quality`)
- `cargo_mass`: Non-negative (validated by `parse_non_negative`)
- `fuel_load`: Non-negative if specified (validated by `parse_non_negative`)
- `dynamic_mass`: False by default

#### CommonHeatConfig
```rust
#[derive(Args, Debug, Clone)]
pub struct CommonHeatConfig {
    /// Heat-aware routing (rejects jumps reaching critical temperature ≥150K)
    #[arg(long = "avoid-critical-state", action = ArgAction::SetTrue)]
    pub avoid_critical_state: bool,
    
    /// Disable temperature constraints for gate-only networks or high-risk planning
    #[arg(long = "no-avoid-critical-state", action = ArgAction::SetTrue)]
    pub no_avoid_critical_state: bool,
    
    /// Temperature calculation model: 'flux' or 'logistic'
    #[arg(long = "sys-temp-curve", value_enum, default_value_t = TemperatureCurveArg::default())]
    pub sys_temp_curve: TemperatureCurveArg,
}
```

**Validation Rules**:
- Mutual exclusion: If both `avoid_critical_state` and `no_avoid_critical_state` are true, `no_avoid_critical_state` takes precedence (explicit disable)
- `sys_temp_curve`: Defaults to `Flux` per existing behavior

**Relationships**:
- `CommonHeatConfig` depends on `TemperatureCurveArg` enum (already exists in main.rs)
- `CommonShipConfig` interacts with `ShipCatalog` at runtime (library layer)

---

### D2: CLI Command Updates

**RouteCommandArgs (Modified)**:
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
    
    /// Suppress minimum external temperature annotations in route output
    #[arg(long = "no-temp", action = ArgAction::SetTrue)]
    no_temp: bool,
    
    /// Optimization objective for planning: distance or fuel
    #[arg(long = "optimize", value_enum)]
    optimize: Option<RouteOptimizeArg>,
    
    /// Maximum number of spatial neighbours to consider
    #[arg(long = "max-spatial-neighbours", default_value = "250")]
    max_spatial_neighbours: usize,
}
```

**ScoutRangeArgs (Modified)**:
```rust
#[derive(Args, Debug, Clone)]
pub struct ScoutRangeArgs {
    /// System name to query (case-sensitive; fuzzy suggestions on mismatch)
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
    
    /// Maximum distance in light-years from the origin system
    #[arg(long, short = 'r')]
    radius: Option<f64>,
    
    /// Include CCP developer/staging systems (AD###, V-###) in results
    #[arg(long, action = ArgAction::SetTrue)]
    include_ccp_systems: bool,
    
    /// Optimization objective for planning: distance or fuel
    #[arg(long = "optimize", value_enum)]
    optimize: Option<RouteOptimizeArg>,
    
    /// Maximum number of spatial neighbours to consider
    #[arg(long = "max-spatial-neighbours", default_value = "250")]
    max_spatial_neighbours: usize,
}
```

**ScoutGatesArgs (No Changes)**:
- Gates command remains unchanged (already minimal; no routing constraints apply)
- `--include-ccp-systems` already exists

---

### D3: Quickstart / Migration Guide

**Output**: `specs/027-route-scout-parameter-parity/quickstart.md`

**Content**:
1. **For End Users**:
   - "What's New": List of new parameters available in `scout range`
   - Before/After examples showing equivalent behavior
   - Migration checklist for scripts using `scout range`

2. **For Developers**:
   - How to use `#[command(flatten)]` with shared argument structs
   - Where to add new parameters (which shared struct?)
   - Test patterns for verifying parameter parity

**Example Section**:
```markdown
### New Parameters in `scout range`

#### Avoidance Constraints
```bash
# OLD: No way to avoid systems in scout range
evefrontier-cli scout range Nod -r 50

# NEW: Avoid hostile systems
evefrontier-cli scout range Nod -r 50 --avoid "Brana" --avoid "H:2L2S"
```

#### Fuel Optimization
```bash
# OLD: Static mass calculation only
evefrontier-cli scout range Nod -r 80 --ship Reflex

# NEW: Dynamic mass recalculation
evefrontier-cli scout range Nod -r 80 --ship Reflex --dynamic-mass --optimize fuel
```
```

---

### D4: Agent Context Update

Run `.specify/scripts/bash/update-agent-context.sh copilot` to add:
- Shared argument structs pattern (`CommonRouteConstraints`, `CommonShipConfig`, `CommonHeatConfig`)
- Parameter parity convention (all routing commands share constraint parameters)
- Test expectations (parameter behavior must be identical across `route` and `scout range`)

**Agent-specific file updated**: `.github/copilot-instructions.md` (technology stack section)

---

## Phase 2: Implementation Planning (STOP HERE)

**Phase 2 is NOT part of `/speckit.plan` output.** Implementation tasks will be generated by `/speckit.tasks` command after Phase 0 and Phase 1 are complete and reviewed.

**Next Steps**:
1. Review `research.md` (Phase 0 output)
2. Review `data-model.md` and `quickstart.md` (Phase 1 output)
3. Run `/speckit.tasks` to generate `tasks.md` with TDD implementation checklist

---

## Summary & Handoff

### Artifacts Generated
- ✅ `spec.md` — Feature specification (user requirements, success criteria)
- ⏳ `research.md` — Phase 0 research findings (to be generated)
- ⏳ `data-model.md` — Phase 1 shared argument struct schemas (to be generated)
- ⏳ `quickstart.md` — Phase 1 migration guide (to be generated)

### Branch Status
**Branch**: `027-route-scout-parameter-parity` (created and checked out)  
**Base**: Current main branch  
**Status**: Planning phase complete; ready for Phase 0 research execution

### Implementation Path
1. **Phase 0** (research): Validate clap flatten pattern, confirm backward compatibility strategy
2. **Phase 1** (design): Define shared argument struct schemas, update CLI command structs
3. **Phase 2** (implementation): TDD cycle → refactor → update docs → CI validation

### Key Risks Mitigated
- ✅ No library changes required (isolates impact to CLI crate)
- ✅ Backward compatibility enforced by default values matching current behavior
- ✅ Test-first approach ensures no regressions
- ✅ Help text design prevents user confusion from parameter overload
