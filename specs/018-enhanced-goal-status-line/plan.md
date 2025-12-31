// Implementation plan auto-filled for feature 018-enhanced-goal-status-line

# Implementation Plan: Enhanced Mode GOAL Status Line

**Branch**: `018-enhanced-goal-status-line` | **Date**: 2025-12-31 | **Spec**: `specs/018-enhanced-goal-status-line/spec.md`
**Input**: Feature specification from `/specs/018-enhanced-goal-status-line/spec.md`

## Summary

Fix the enhanced CLI output so GOAL steps render a status line (min temp, planets, moons) like other steps, and show a “Black Hole” badge for systems 30000001–30000003. Keep alignment unchanged and avoid affecting other formats.

## Technical Context

**Language/Version**: Rust 1.91.1  
**Primary Dependencies**: `clap` CLI, `thiserror`, `serde`, project libs in `crates/evefrontier-lib`  
**Storage**: SQLite starmap dataset (read-only)  
**Testing**: `cargo test` (workspace; focus on `-p evefrontier-cli`)  
**Target Platform**: Linux/macOS CLI, AWS Lambda binaries downstream  
**Project Type**: Multi-crate Rust workspace (library + CLI + lambdas)  
**Performance Goals**: No regressions; negligible impact (display-only change)  
**Constraints**: Preserve existing output formats; follow palette tag conventions  
**Scale/Scope**: Small UI/format fix

## Constitution Check

No constitution gates triggered; change is display-only and follows existing palette/tag patterns.

## Project Structure

### Documentation (this feature)

```text
specs/018-enhanced-goal-status-line/
├── plan.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/evefrontier-cli/src/output.rs      # Enhanced renderer logic
crates/evefrontier-cli/src/terminal.rs    # Color palette/tag styles
crates/evefrontier-cli/tests/             # CLI/regression tests
crates/evefrontier-lib/src/path.rs        # Fuel computations used by outputs
docs/USAGE.md                             # CLI examples
CHANGELOG.md                              # Release notes
```

**Structure Decision**: Use existing Rust workspace layout; CLI crate owns rendering, library crate owns path/fuel data.

## Complexity Tracking

Not applicable; no additional complexity introduced.
