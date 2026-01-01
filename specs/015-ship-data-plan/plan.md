# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link] **Input**: Feature
specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See
`.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add robust ship data download and caching to the existing dataset downloader so `ship_data.csv` is
fetched from releases, stored in the same cache alongside the database, validated, and surfaced via
a library-first API (`ShipCatalog::from_path` / `ensure_ship_data`). Implementation follows TDD: add
unit tests for downloader behaviour, CSV parsing, checksum handling, and integration tests for the
CLI and Lambda layers.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.91.1  
**Primary Dependencies**: reqwest (blocking client, already used), zip, csv, serde, sha2 (for
checksums), tempfile, tracing  
**Storage**: OS cache directory under `evefrontier_datasets/` (same as DB cache)  
**Testing**: `cargo test` (unit/integration), new fixtures under `docs/fixtures/`  
**Target Platform**: Linux (CLI & Lambda targets), CI  
**Project Type**: Library-first feature implemented in `crates/evefrontier-lib`  
**Performance Goals**: negligible latency addition to route planning (<5ms per route on fixture).
Cache lookups should be O(1) file checks.  
**Constraints**: Follow TDD (see Constitution), preserve backward compatibility when ship data
missing, avoid writing into protected fixtures, enforce strict validation of CSV rows per
`ShipAttributes::validate()`.  
**Scale/Scope**: Small: modify downloader and add tests + CLI/Lambda thin integration.

## Constitution Check

GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.

- Library-First: implementation occurs in `crates/evefrontier-lib` (satisfied).
- TDD: tests will be added first for downloader and parsing (must be followed).
- Security-First: validate CSV inputs, write checksum sidecars, avoid path traversal on archive
  extraction (existing `extract_archive` already enforces this).

No violations; the work follows the Constitution.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: Implement as a library feature in `crates/evefrontier-lib`. Changes will be
localized to:

- `crates/evefrontier-lib/src/github.rs` — detect and fetch `ship_data.csv` release asset, write
  cached file `<tag>-ship_data.csv` and `<tag>-ship_data.csv.sha256`.
- `crates/evefrontier-lib/src/dataset.rs` — extend `DatasetPaths` to include
  `ship_data: Option<PathBuf>` and return it from `ensure_dataset`/`ensure_e6c3_dataset`.
- `crates/evefrontier-lib/src/ship.rs` — existing CSV parsing is sufficient; add unit tests that
  load cached file and validate parsing and error paths.
- `crates/evefrontier-lib/tests/` — add `download_ship_data.rs` and `ship_data_parsing.rs`
  integration/unit tests using fixtures in `docs/fixtures/`.
- `crates/evefrontier-cli/` — CLI: add `--list-ships` support and flags are already implemented; add
  integration tests to assert the flag uses the cached ship data.
- Lambda crates (`evefrontier-lambda-*`) — update request/response schemas and add small integration
  test to ensure `ship_data.csv` is bundled; bundling handled in a follow-up step after library API
  is stable.

This keeps the CLI & Lambda thin and adheres to the Constitution.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation                  | Why Needed         | Simpler Alternative Rejected Because |
| -------------------------- | ------------------ | ------------------------------------ |
| [e.g., 4th project]        | [current need]     | [why 3 projects insufficient]        |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient]  |
