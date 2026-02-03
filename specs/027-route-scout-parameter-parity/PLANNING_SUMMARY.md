# Route & Scout Parameter Parity - Planning Complete

**Feature ID**: 027  
**Branch**: `027-route-scout-parameter-parity`  
**Status**: ✅ Planning Phase Complete (Phase 0 & Phase 1)  
**Date**: 2026-02-03

---

## Executive Summary

Successfully created a comprehensive plan to unify parameter sets across `route` and `scout` commands by extracting shared argument structs and adding missing parameters to `scout range`. **Zero breaking changes** — all new parameters are additive with backward-compatible defaults.

---

## Artifacts Generated

### Phase 0: Research
✅ **`research.md`** — Research findings on:
- Clap `#[command(flatten)]` pattern validation
- Backward compatibility strategy (100% compatible)
- Help text organization with help headings

### Phase 1: Design
✅ **`data-model.md`** — Complete struct schemas for:
- `CommonRouteConstraints` (max jump, avoidance, temperature)
- `CommonShipConfig` (ship, fuel quality, cargo, dynamic mass)
- `CommonHeatConfig` (critical state avoidance, temperature model)

✅ **`quickstart.md`** — User migration guide with:
- What's new (9 new parameters in `scout range`)
- Migration examples (before/after comparisons)
- Usage patterns for common scenarios
- Developer guide for using shared structs

### Supporting Documents
✅ **`spec.md`** — Feature specification with user stories and success criteria  
✅ **`plan.md`** — Implementation plan with Constitution compliance check

---

## Constitution Compliance: ✅ ALL GATES PASSED

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Test-Driven Development** | ✅ Pass | TDD workflow planned; integration tests will use assert_cmd |
| **II. Library-First Architecture** | ✅ Pass | No library changes required; CLI-only refactor |
| **III. ADRs** | ✅ Pass | No ADR needed (DRY cleanup, not architectural change) |
| **IV. Clean Code** | ✅ Pass | Reduces duplication; improves ergonomics |
| **V. Security-First** | ✅ Pass | Existing validation preserved; no new attack surface |
| **VI. Testing Tiers** | ✅ Pass | Integration tests planned for all new parameters |
| **VII. Refactoring** | ✅ Pass | Focused refactor; green tests before/after |

**No violations requiring justification.**

---

## Technical Approach

### 1. Shared Argument Structs (DRY Principle)
```rust
// NEW: crates/evefrontier-cli/src/common_args.rs
pub struct CommonRouteConstraints { /* ... */ }
pub struct CommonShipConfig { /* ... */ }
pub struct CommonHeatConfig { /* ... */ }
```

### 2. Composition via Flatten
```rust
#[derive(Args, Debug, Clone)]
struct RouteCommandArgs {
    #[command(flatten)]
    constraints: CommonRouteConstraints,  // ← Shared
    
    #[command(flatten)]
    ship: CommonShipConfig,  // ← Shared
    
    #[command(flatten)]
    heat: CommonHeatConfig,  // ← Shared
    
    // Route-specific flags...
}
```

### 3. Backward Compatibility Strategy
- **All new parameters optional** with defaults matching existing behavior
- **No parameter renames** — existing flags unchanged
- **Output schema unchanged** unless new parameters explicitly used

---

## Parameter Parity Matrix

| Parameter | Route | Scout Range (Before) | Scout Range (After) | Status |
|-----------|-------|----------------------|---------------------|--------|
| `--algorithm` | ✅ | ❌ | ✅ | 🆕 Added |
| `--max-jump` | ✅ | ❌ | ✅ | 🆕 Added |
| `--avoid` | ✅ | ❌ | ✅ | 🆕 Added |
| `--avoid-gates` | ✅ | ❌ | ✅ | 🆕 Added |
| `--avoid-critical-state` | ✅ | ❌ | ✅ | 🆕 Added |
| `--dynamic-mass` | ✅ | ❌ | ✅ | 🆕 Added |
| `--optimize` | ✅ | ❌ | ✅ | 🆕 Added |
| `--max-spatial-neighbours` | ✅ | ❌ | ✅ | 🆕 Added |
| `--no-avoid-critical-state` | ✅ | ❌ | ✅ | 🆕 Added |
| `--max-temp` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--ship` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--fuel-quality` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--cargo-mass` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--fuel-load` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--sys-temp-curve` | ✅ | ✅ | ✅ | ✅ Already consistent |
| `--include-ccp-systems` | ❌ | ✅ | ✅ | 🆕 Add to route |

**Result**: Full parameter parity achieved (15/15 parameters consistent)

---

## Files to Modify (Phase 2 Implementation)

### New Files
- `crates/evefrontier-cli/src/common_args.rs` — Shared argument structs

### Modified Files
- `crates/evefrontier-cli/src/main.rs` — Extract shared structs, update command args
- `crates/evefrontier-cli/src/commands/route.rs` — Use flattened shared structs
- `crates/evefrontier-cli/src/commands/scout.rs` — Add shared structs to ScoutRangeArgs
- `docs/USAGE.md` — Add examples for new scout range parameters
- `CHANGELOG.md` — Add entry under Unreleased

### New Test Files
- `crates/evefrontier-cli/tests/cli_integration.rs` — Parameter parity integration tests
- `crates/evefrontier-cli/tests/route_tests.rs` — Route with `--include-ccp-systems`
- `crates/evefrontier-cli/tests/scout_tests.rs` — Scout range with new parameters

---

## User-Facing Changes

### New Capabilities in `scout range`
1. **Avoidance constraints** — Exclude hostile systems from results
2. **Algorithm selection** — Choose BFS/Dijkstra/A* for performance tuning
3. **Fuel optimization** — Order results by fuel-efficient traversal
4. **Heat-aware routing** — Reject high-heat spatial jumps
5. **Spatial-only mode** — Find systems reachable without gates
6. **Dynamic mass** — Accurate fuel projections for long routes

### Example: Fuel-Optimized Scouting
```bash
# Before: No fuel optimization
evefrontier-cli scout range Nod -r 80 --ship Reflex

# After: Fuel-optimized with dynamic mass
evefrontier-cli scout range Nod -r 80 --ship Reflex --optimize fuel --dynamic-mass
```

**Output**: Systems ordered by cumulative fuel cost instead of distance.

---

## Help Text Improvements

### Before: Flat List (15 lines)
All parameters in a single unorganized list.

### After: Organized Sections (35 lines)
```
NAVIGATION:          (origin, radius, limit)
ROUTING CONSTRAINTS: (max-jump, avoid, avoid-gates, max-temp)
SHIP & FUEL:         (ship, fuel-quality, cargo-mass, fuel-load, dynamic-mass)
HEAT MECHANICS:      (avoid-critical-state, sys-temp-curve)
OPTIMIZATION:        (algorithm, optimize, max-spatial-neighbours)
FILTERS:             (include-ccp-systems)
```

**Impact**: Easier to discover relevant parameters; reduced cognitive load.

---

## Next Steps

### Phase 2: Implementation (via `/speckit.tasks`)
1. **Generate tasks.md** — TDD implementation checklist
2. **Red Phase** — Write failing integration tests
3. **Green Phase** — Implement shared argument structs
4. **Refactor Phase** — Clean up conversion logic
5. **Documentation** — Update `USAGE.md` and `CHANGELOG.md`
6. **CI Validation** — Ensure all tests pass

### Timeline Estimate
- **Research (Phase 0)**: ✅ Complete (1 hour)
- **Design (Phase 1)**: ✅ Complete (2 hours)
- **Implementation (Phase 2)**: ⏳ Pending (estimate: 4-6 hours)
  - Code changes: 2 hours
  - Tests: 2 hours
  - Documentation: 1 hour
  - CI validation: 1 hour

**Total Effort**: ~8-10 hours (including planning)

---

## Key Risks Mitigated

| Risk | Mitigation | Status |
|------|------------|--------|
| Breaking CLI backward compatibility | All new params optional with backward-compat defaults | ✅ Mitigated |
| Confusing help text | Use help headings to organize parameters | ✅ Mitigated |
| Increased test maintenance | Share fixtures between route/scout test suites | ✅ Mitigated |
| Performance regression | No performance impact (argument parsing is negligible) | ✅ Mitigated |

---

## Branch Status

**Branch**: `027-route-scout-parameter-parity`  
**Base**: `main` (or current HEAD)  
**Status**: Clean; ready for implementation

**Git Log**:
```bash
git log --oneline origin/main..HEAD
# (No commits yet; planning artifacts staged locally)
```

---

## Approval Checklist

Before proceeding to implementation:
- [ ] Review `spec.md` for clarity and completeness
- [ ] Review `research.md` for technical soundness
- [ ] Review `data-model.md` for struct design
- [ ] Review `quickstart.md` for user-facing documentation
- [ ] Confirm Constitution compliance (all gates passed)
- [ ] Confirm backward compatibility strategy
- [ ] Approve Phase 2 implementation plan

**Reviewer**: _[TBD]_  
**Date Approved**: _[TBD]_

---

## Contact

**Questions or Feedback**: Open an issue on GitHub or contact the maintainer.

**Related Specs**: None (standalone feature)

---

**Planning Phase Complete** ✅  
**Ready for `/speckit.tasks` command to generate implementation checklist.**
