# Phase 0 Research — Ship Data & Fuel Calculations

## Clarification: Performance Goals
- **Decision**: Target <5ms additional latency per route on CLI (fixture-sized routes ≤50 hops) and <10ms CPU overhead on Lambda/container for fuel projection; no extra allocations that exceed existing Lambda memory footprint (<512 MB).
- **Rationale**: Fuel projection is arithmetic over existing route steps; keeping overhead minimal preserves current UX and cold-start budgets.
- **Alternatives Considered**:
  - Permit unbounded overhead (rejected: degrades UX and violates performance expectations).
  - Defer projections to post-processing JSON step (rejected: duplicates traversal, higher cost).

## Clarification: Ship Catalog Scale & Release Cadence
- **Decision**: Assume ship catalog size in tens (<200 entries) with new releases per dataset drop (e.g., e6c4+). Load entire CSV into memory once per process; no pagination or DB table required.
- **Rationale**: ADR 0015 cites `ship_data.csv` in dataset releases; typical EVE ship catalogs are small relative to system graph. In-memory catalog keeps API simple and fast.
- **Alternatives Considered**:
  - Persist ships in SQLite table (rejected: unnecessary complexity for small catalogs).
  - Stream per lookup (rejected: repeated I/O, higher latency, harder validation).

## Best Practices: CSV Parsing & Validation
- **Decision**: Use `csv` + `serde` with typed struct and strict validation; reject rows with missing/negative values; clamp fuel/quality inputs to allowed ranges; surface actionable errors.
- **Rationale**: Prevents malformed dataset from propagating; aligns with OWASP A03 (Injection) and Security-First guidance.
- **Alternatives Considered**:
  - Lenient parsing with defaults (rejected: hides data quality issues, risks incorrect fuel math).

## Best Practices: Caching & Download
- **Decision**: Download `ship_data.csv` over HTTPS alongside `static_data.db`, store in `evefrontier_datasets/` with checksum marker; reuse cached file when checksum matches; expose override path for tests.
- **Rationale**: Matches existing dataset caching pattern, avoids partial writes, and supports deterministic tests.
- **Alternatives Considered**:
  - Separate cache directory for ship data (rejected: increases path sprawl, complicates cleanup).
