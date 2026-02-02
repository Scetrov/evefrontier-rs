# Implementation Plan: Scout CLI Subcommand

**Branch**: `025-scout-cli-subcommand` | **Date**: 2026-01-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/025-scout-cli-subcommand/spec.md`

## Summary

Add `scout gates` and `scout range` CLI subcommands to expose existing Lambda/service scout functionality via the command line. This leverages existing library code (`evefrontier-lib`) for starmap adjacency lookups and spatial index queries, with new CLI glue code for argument parsing and output formatting.

## Technical Context

**Language/Version**: Rust 1.93.0 (per `.rust-toolchain`)  
**Primary Dependencies**: clap (CLI parsing), serde/serde_json (JSON output), tracing (logging), evefrontier-lib (core logic)  
**Storage**: SQLite dataset (`static_data.db`) + spatial index (`.spatial.bin`)  
**Testing**: `cargo test` with integration tests using fixture dataset  
**Target Platform**: Linux, macOS, Windows (cross-platform CLI)  
**Project Type**: Single Rust workspace with multiple crates  
**Performance Goals**: Sub-second response for gate queries; <2s for range queries with 100 results  
**Constraints**: Must load spatial index for range queries; gate queries use adjacency list (no index needed)  
**Scale/Scope**: ~5,000 systems in e6c3 dataset; typical queries return 1-100 results

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ Pass | Will write tests first for new subcommands |
| II. Library-First Architecture | ✅ Pass | Reusing existing `evefrontier-lib` adjacency and spatial query logic |
| III. Architecture Decision Records | ✅ Pass | No new ADR needed—extending existing CLI pattern |
| IV. Clean Code & Cognitive Load | ✅ Pass | Following existing CLI structure in `main.rs` |
| V. Security-First Development | ✅ Pass | Input validation for system names; no external data handling |
| VI. Testing Tiers | ✅ Pass | Unit + integration tests; smoke test via CLI invocation |
| VII. Refactoring & Technical Debt | ✅ Pass | No legacy code touched; adding new functionality |

**All gates pass. Proceeding with implementation.**

## Project Structure

### Documentation (this feature)

```text
specs/025-scout-cli-subcommand/
├── spec.md              # Feature specification
├── plan.md              # This file (implementation plan)
├── research.md          # Phase 0 output (minimal—existing patterns)
├── data-model.md        # Phase 1 output (response structures)
├── quickstart.md        # Phase 1 output (usage examples)
└── tasks.md             # Phase 2 output (task breakdown)
```

### Source Code (changes)

```text
crates/evefrontier-cli/
├── src/
│   ├── main.rs              # Add Scout subcommand enum variant
│   ├── commands/
│   │   ├── mod.rs           # Export scout module
│   │   └── scout.rs         # NEW: Scout gates/range handlers
│   └── output_helpers.rs    # Extend with scout formatting functions
└── tests/
    └── scout.rs             # NEW: Integration tests for scout commands

docs/
├── USAGE.md                 # Add scout command documentation
└── README.md                # Update CLI command list
```

**Structure Decision**: Single project structure. New code added to existing `evefrontier-cli` crate following established patterns.

## Complexity Tracking

No violations to justify—this is a straightforward feature addition following existing patterns.

## Implementation Phases

### Phase 0: Research ✅ Complete

**Output**: [research.md](research.md)

No unknowns identified. Existing patterns are well-established:
- Lambda handlers in `evefrontier-lambda-scout-gates` and `evefrontier-lambda-scout-range` provide reference implementation
- CLI argument parsing follows `RouteCommandArgs` pattern in `main.rs`
- Output formatting follows `output_helpers.rs` patterns

### Phase 1: Design ✅ Complete

**Outputs**:
- [data-model.md](data-model.md) — Response structures and CLI argument types
- [quickstart.md](quickstart.md) — Usage examples and expected output
- [contracts/cli-interface.md](contracts/cli-interface.md) — CLI interface contract

**Data Model** (response structures):

```rust
/// Response for gate neighbors query
pub struct ScoutGatesResult {
    pub system: String,
    pub system_id: i64,
    pub neighbors: Vec<GateNeighbor>,
}

pub struct GateNeighbor {
    pub name: String,
    pub id: i64,
}

/// Response for range query
pub struct ScoutRangeResult {
    pub system: String,
    pub system_id: i64,
    pub radius: Option<f64>,
    pub max_temperature: Option<f64>,
    pub limit: usize,
    pub systems: Vec<RangeNeighbor>,
}

pub struct RangeNeighbor {
    pub name: String,
    pub id: i64,
    pub distance_ly: f64,
    pub min_temp_k: Option<f64>,
}
```

**CLI Interface**:

```rust
#[derive(Subcommand, Debug)]
enum Command {
    // ... existing commands ...
    
    /// Scout nearby systems (gates or spatial range)
    Scout(ScoutCommandArgs),
}

#[derive(Args, Debug)]
struct ScoutCommandArgs {
    #[command(subcommand)]
    subcommand: ScoutSubcommand,
}

#[derive(Subcommand, Debug)]
enum ScoutSubcommand {
    /// List gate-connected neighbors of a system
    Gates(ScoutGatesArgs),
    /// Find systems within spatial range
    Range(ScoutRangeArgs),
}

#[derive(Args, Debug)]
struct ScoutGatesArgs {
    /// System name to query
    system: String,
}

#[derive(Args, Debug)]
struct ScoutRangeArgs {
    /// System name to query
    system: String,
    /// Maximum number of results (default: 10, max: 100)
    #[arg(long, default_value = "10")]
    limit: usize,
    /// Maximum distance in light-years
    #[arg(long)]
    radius: Option<f64>,
    /// Maximum star temperature in Kelvin
    #[arg(long = "max-temp")]
    max_temp: Option<f64>,
}
```

### Phase 2: Tasks ✅ Complete

**Output**: [tasks.md](tasks.md)

24 tasks organized into 5 phases:
- **Phase 1**: Setup (T001-T004) — CLI argument types and module scaffolding
- **Phase 2**: Foundational (T005-T010) — Output formatting functions
- **Phase 3**: User Story 1 (T011-T014) — Scout Gates 🎯 MVP
- **Phase 4**: User Story 2 (T015-T018) — Scout Range
- **Phase 5**: Polish (T019-T024) — Documentation updates

High-level task breakdown:

1. **T1**: Add `ScoutSubcommand` enum and argument structs to `main.rs`
2. **T2**: Create `commands/scout.rs` with `handle_scout_gates()` and `handle_scout_range()` functions
3. **T3**: Add output formatting functions to `output_helpers.rs` for scout results
4. **T4**: Wire scout subcommand in main dispatch logic
5. **T5**: Write integration tests for both subcommands
6. **T6**: Update `docs/USAGE.md` with scout command examples
7. **T7**: Update `README.md` CLI command list
8. **T8**: Update `CHANGELOG.md`

## Dependencies

- **evefrontier-lib**: `Starmap.adjacency` for gate lookups, `SpatialIndex.nearest_filtered()` for range queries
- **No new crate dependencies required**

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Spatial index not built | Medium | High | Auto-build fallback (existing pattern); warn user |
| Unknown system name | High | Low | Fuzzy matching suggestions (existing pattern) |
| Large result sets | Low | Medium | Enforce limit=100 max; paginated output if needed |

## Acceptance Criteria

1. `evefrontier-cli scout gates "Nod"` returns gate neighbors
2. `evefrontier-cli scout range "Nod" --limit 5` returns nearby systems
3. `--format json` produces valid JSON output
4. Unknown systems show fuzzy suggestions
5. All tests pass
6. Documentation updated
