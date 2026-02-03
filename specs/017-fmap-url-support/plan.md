# Implementation Plan: fmap URL Support

**Branch**: `017-fmap-url-support` | **Date**: 2025-12-31 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/017-fmap-url-support/spec.md`

## Summary

Implement fmap URL encoding/decoding to enable sharing routes with the frontier-reapers/starmap
visualization tool. The feature adds a new `fmap` module to the library with bitpacking,
compression, and base64url encoding, plus CLI integration via `--format fmap` and `fmap decode`.

## Technical Context

**Language/Version**: Rust 1.93.0 (per `.rust-toolchain`)  
**Primary Dependencies**: flate2 (gzip), base64 (encoding) - both already transitive deps  
**Storage**: N/A (stateless encoding/decoding)  
**Testing**: cargo test with fixtures in `docs/fixtures/`  
**Target Platform**: Linux (primary), macOS, Windows (CLI)  
**Project Type**: Library-first (evefrontier-lib) + CLI wrapper  
**Performance Goals**: Encode <1ms for 1000 waypoints, decode <1ms  
**Constraints**: Semantic compatibility with JavaScript reference implementation (identical decoded routes; bytes may differ)  
**Scale/Scope**: Routes typically 5-100 waypoints, max 65535 (u16 limit)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ PASS | Tests first: encode/decode, round-trip, cross-impl |
| II. Library-First Architecture | ✅ PASS | `fmap.rs` in evefrontier-lib, CLI is thin wrapper |
| III. ADR for Significant Decisions | ⚠️ DEFERRED | ADR for encoder/decoder algorithms per TODO.md |
| IV. Clean Code & Cognitive Load | ✅ PASS | Clear function names, no magic numbers (constants) |
| V. Security-First Development | ✅ PASS | Input validation, no external URLs in defaults |
| VI. Testing Tiers | ✅ PASS | Unit + integration + cross-impl test vectors |
| VII. Refactoring & Technical Debt | ✅ PASS | No new debt; clean implementation |

**Post-Phase 1 Re-check**: All gates pass. ADR for algorithms can be created after implementation
proves stable (per TODO.md pattern of documenting deviations only).

## Project Structure

### Documentation (this feature)

```text
specs/017-fmap-url-support/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research findings
├── data-model.md        # Data structures and encoding format
├── quickstart.md        # Quick usage guide
├── contracts/
│   ├── library-api.md   # Library API contract
│   └── cli-api.md       # CLI API contract
└── tasks.md             # Phase 2 implementation tasks (TBD)
```

### Source Code (repository root)

```text
crates/evefrontier-lib/src/
├── fmap.rs              # New: fmap encoding/decoding module
├── error.rs             # Extended with FmapError variants
├── output.rs            # Extended with fmap OutputFormat
└── lib.rs               # Add `pub mod fmap;`

crates/evefrontier-lib/tests/
└── fmap.rs              # Integration tests for fmap module

crates/evefrontier-cli/src/
├── main.rs              # Add fmap subcommand
├── commands/
│   └── fmap.rs          # fmap decode command handler
└── format.rs            # Extended with fmap format handling

docs/fixtures/
└── fmap_test_vectors.json  # Cross-implementation test data
```

**Structure Decision**: Single `fmap.rs` module in library; CLI wrapper in commands submodule.
Library-first architecture per Constitution II.

## Complexity Tracking

No constitution violations to justify. Implementation is straightforward:
- Single new module in library
- One new CLI subcommand
- No new external dependencies (using existing transitive deps)
- Clear algorithm from reference implementation

## Phase Summary

| Phase | Status | Output |
|-------|--------|--------|
| 0: Research | ✅ Complete | [research.md](./research.md) |
| 1: Design | ✅ Complete | [data-model.md](./data-model.md), [contracts/](./contracts/) |
| 2: Tasks | 📋 Pending | Run `/speckit.tasks` to generate |

## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Begin TDD cycle: write failing tests for encode/decode
3. Implement `fmap.rs` module
4. Extend CLI with `--format fmap` and `fmap decode`
5. Create cross-implementation test vectors
6. Update CHANGELOG.md and docs/USAGE.md

## References

- [Feature Spec](./spec.md)
- [Research](./research.md)
- [Data Model](./data-model.md)
- [Library API Contract](./contracts/library-api.md)
- [CLI API Contract](./contracts/cli-api.md)
- [ROUTE_FEATURE.md](https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md)
- [bitpacking.js](https://github.com/frontier-reapers/starmap/blob/main/src/bitpacking.js)
