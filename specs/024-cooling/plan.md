# Implementation Plan: 024-cooling

**Branch**: `024-cooling` | **Date**: 2026-01-06 | **Spec**: [/specs/024-cooling/spec.md](/specs/024-cooling/spec.md)
**Input**: Feature specification from `/specs/024-cooling/spec.md`

## Summary
Implement a cooling time indicator in the route output. The calculation uses Newton's Law of Cooling ($T(t) = T_{env} + (T_0 - T_{env}) e^{-kt}$) to estimate the time required for a ship to cool from its post-jump temperature to the nominal temperature ($30K$) or the system-specific jump threshold. The indicator will be formatted as `XmYs` (e.g., `2m4s`).

## Technical Context
**Language/Version**: Rust 1.91.1
**Primary Dependencies**: `evefrontier-lib`, `clap`, `serde`
**Storage**: N/A (runtime calculation)
**Testing**: `cargo test`, unit tests in `crates/evefrontier-lib/src/ship.rs` and `crates/evefrontier-lib/src/output.rs`.
**Target Platform**: CLI and AWS Lambda
**Project Type**: Library-First (Library -> CLI/Lambda)
**Performance Goals**: Fast pathfinding and route projection (<100ms)
**Constraints**: 
- Model: $t = -\frac{1}{k} \ln\left(\frac{T_{threshold} - T_{env}}{T_0 - T_{env}}\right)$
- $T_{threshold} = 30.0$ (Nominal)
- $T_0 = T_{ambient} + \Delta T_{jump}$
- $T_{env} = T_{ambient}$ (min_external_temp)

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] I. TDD (Tests first)
- [x] II. Library-First (Logic in `evefrontier-lib`)
- [x] III. ADR (Not strictly needed for this minor enhancement, but logic documented)
- [x] IV. Clean Code ( descriptive names, single responsibility)
- [x] V. Security-First (Input validation for temperatures)
- [x] VI. Testing Tiers (Unit tests for formula)

## Project Structure

### Documentation (this feature)
```text
specs/024-cooling/
├── plan.md              # This file
├── research.md          # Newton's Law of Cooling details
├── data-model.md        # Updates to HeatProjection
└── quickstart.md        # Usage examples
```

### Source Code (repository root)
```text
crates/
├── evefrontier-lib/
│   ├── src/
│   │   ├── ship.rs       # Newton's Law formulas
│   │   ├── output.rs     # Formatting 2m4s
│   │   └── routing.rs    # Integration into route steps
├── evefrontier-cli/      # UI updates
└── evefrontier-lambda-*/ # Response updates
```

**Structure Decision**: Following the existing Library-First architecture. logic additions to `ship.rs`, integration in `routing.rs`, and formatting in `output.rs`.

## Complexity Tracking
| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |
