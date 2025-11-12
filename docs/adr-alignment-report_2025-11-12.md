# ADR Alignment Report — 2025-11-12

This report reviews ADRs under `docs/adrs/` and compares them to the current implementation under `crates/`.

| ADR | Decision Summary | Alignment | Evidence in Code | Deviations / Gaps | Recommended Actions |
| --- | --- | --- | --- | --- | --- |
| 0001-use-nygard-adr.md | Use Nygard-style ADRs, zero-padded IDs, immutable history | Fully aligned | ADRs exist in `docs/adrs` with zero-padded IDs; references in `README.md`, `docs/README.md` | None | Optionally add a CI check to validate ADR filename convention and immutability |
| 0002-workspace-structure.md | Cargo workspace with `evefrontier-lib` and `evefrontier-cli`; core logic in lib | Fully aligned | Library: `crates/evefrontier-lib/` holds logic (`db.rs`, `github.rs`, `graph.rs`, `path.rs`, `routing.rs`); CLI: `crates/evefrontier-cli/src/main.rs` is thin and delegates | None | Keep CLI thin; extend library for new behaviors as needed |
| 0003-downloader-caching.md | Download latest dataset, cache under OS cache dir, atomic writes, zip extraction | Fully aligned | `crates/evefrontier-lib/src/github.rs` (downloader, caching, extraction), `crates/evefrontier-lib/src/dataset.rs` (ensure/resolve dataset); tests under `crates/evefrontier-lib/tests/dataset_*` | None observed | Consider retries/backoff for robustness; document cache lifecycle |
| 0004-schema-detection.md | Runtime detection for dataset schema (SolarSystems/Jumps vs legacy) | Fully aligned | `crates/evefrontier-lib/src/db.rs` detects schema and adapts queries; tests: `tests/load_starmap.rs` | None | Add more fixtures if new schemas appear |
| 0005-cli-design.md | CLI remains thin; flags/options map onto library APIs; shared resolution for data path | Fully aligned | `crates/evefrontier-cli/src/main.rs` parses flags and calls library (`routing.rs`, `output.rs`, `dataset.rs`); `--data-dir`/env resolution honored | None | Keep new subcommands minimal and delegate to lib |
| 0006-software-components.md | Rust workspace; Node+pnpm for docs/tooling; Nx to orchestrate tasks | Partially aligned | Rust workspace present; helper scripts under `scripts/`; `AGENTS.md` mentions Nx | Missing Nx config (`package.json`, `nx.json`, `pnpm-lock.yaml`), no Nx targets wired | Scaffold minimal Node workspace + Nx; register build/test/lint/audit tasks |
| 0007-devsecops-practices.md | Clippy, fmt, tests, audit, markdown lint, CI gating | Partially aligned | Scripts present (`scripts/run-audit.js`, markdown/prettier helpers) | CI wiring not visible; audit gating unclear; Nx not configured | Add CI workflows for fmt/clippy/test/audit; integrate with Nx once scaffolded |
| 0008-software-currency.md | Keep dependencies current; automate outdated reporting | Partially aligned | Scripts like `scripts/outdated-report.js`, `scripts/check-pnpm-outdated.js` | Lacks Node workspace; no scheduled CI job | Add scheduled CI to run outdated checks; publish report artifact |
| 0009-kd-tree-spatial-index.md | Precompute and ship KD-tree spatial index; loader in lib; builder subcommand | Not aligned | No KD-tree module in lib; no builder CLI; no serialization/compression deps | Feature not implemented | Implement KD-tree index per ADR: lib loader/querier; CLI builder; choose format (e.g., `postcard` + `zstd`); tests and docs |
| 0010-maintain-changelog.md | Maintain human-readable CHANGELOG and require updates per change | Partially aligned | `CHANGELOG.md` exists; contributor docs reference it | No automated enforcement | Add CI check to ensure CHANGELOG is updated for code changes (allow docs-only exemptions) |

## Additional Observations

- Data path resolution order (CLI `--data-dir` > env `EVEFRONTIER_DATA_DIR` > platform default) is implemented across `crates/evefrontier-cli/src/main.rs` and `crates/evefrontier-lib/src/dataset.rs`.
- Routing algorithms and graph selection align with usage docs: `graph.rs`, `path.rs`, and `routing.rs` implement BFS/Dijkstra/A* and gate/spatial/hybrid graph modes, with constraints for max jump distance, avoided systems, etc.
- Downloader behavior matches ADRs: atomic writes and cache placement implemented in `github.rs` and `dataset.rs`.

## Proposed Follow-ups

- Nx/Node scaffold: add `package.json`, `pnpm-lock.yaml`, `nx.json`, targets for `build`, `test`, `lint`, `clippy`, `audit`, and docs scripts.
- KD-tree spatial index (ADR 0009): add library module for index build/load/query; CLI `index-build` subcommand; serialization format and compression; integrate into spatial routing path.
- CI enforcement: workflows for fmt/clippy/tests/audit; CHANGELOG update guard; optional ADR filename/style validator; scheduled outdated checks.
- Documentation updates: `README.md` and `docs/USAGE.md` for index builder, dataset/index caching behavior, and Nx commands.
