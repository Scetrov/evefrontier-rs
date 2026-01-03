# Implementation Plan: Ship data downloader & catalog (concise)

**Branch**: `015-ship-data-plan` | **Goal**: Ensure `ship_data.csv` releases are discovered, cached alongside datasets, validated, and exposed via `ShipCatalog` for CLI and Lambda consumption.

## Summary

Implement downloader support to fetch `ship_data.csv` from dataset releases, store a `<tag>-ship_data.csv` cache entry, and load it via `ShipCatalog::from_path`. Add unit and integration tests under `crates/evefrontier-lib/tests/` using the `docs/fixtures/` ship CSV fixture.

## Scope

- Downloader: detect ship CSV assets and cache them with checksum sidecars.
- Library: expose `ShipCatalog::from_path` and `ensure_ship_data` helpers.
- CLI: `--list-ships` / `--ship` support (integration tests).
- Lambda: support bundled ship data during cold-start (follow-up bundling step).

## Acceptance Criteria

- Ship CSV is downloaded and cached next to dataset assets when present.
- `ShipCatalog::from_path` correctly parses common header variants and validates fields.
- Integration tests using `docs/fixtures/ship_data.csv` pass in CI.

