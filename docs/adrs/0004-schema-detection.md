# ADR 0004: Database schema detection and query adaptation

## Status

Accepted

## Context

Different dataset releases may use differing DB schemas and table names. The library must be
resilient and able to load system and jump data from multiple schemas.

## Decision

Detect the schema at runtime (for example via `PRAGMA table_info` or querying `sqlite_master`) and
switch to a small set of isolated SQL queries for each supported schema. Keep schema detection and
queries isolated in `crates/evefrontier-lib/src/db.rs`.

Implementation notes (where to look)

- `crates/evefrontier-lib/src/db.rs` contains the loader and schema-detection code.
- The loader supports the `static_data.db` schema (tables `SolarSystems(solarSystemId, name)` and
  `Jumps(fromSystemId, toSystemId)`) and older `mapSolarSystems` schemas.

## Rationale

- Isolating queries per schema reduces risk of SQL errors and keeps migration logic easy to extend.
- Tests can provide small fixture DB files to validate each supported schema.

## Consequences

- Adding support for a new schema requires adding detection logic and a new set of queries in
  `db.rs`, plus tests.
