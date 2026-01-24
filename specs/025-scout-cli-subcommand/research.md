# Research: Scout CLI Subcommand

**Date**: 2026-01-24  
**Status**: Complete

## Research Summary

This feature requires minimal research as it reuses existing, well-established patterns from the codebase.

## Existing Implementations

### Reference: Lambda Scout-Gates Handler
**Location**: `crates/evefrontier-lambda-scout-gates/src/main.rs`

Key implementation details:
- Uses `starmap.system_id_by_name()` for system lookup
- Uses `starmap.adjacency.get(&system_id)` for gate neighbors
- Returns `Vec<Neighbor>` with name and ID
- Handles unknown systems with fuzzy suggestions via `starmap.fuzzy_system_matches()`

### Reference: Lambda Scout-Range Handler
**Location**: `crates/evefrontier-lambda-scout-range/src/main.rs`

Key implementation details:
- Uses `SpatialIndex.nearest_filtered()` for range queries
- Accepts `NeighbourQuery { k, radius, max_temperature }` struct
- Returns systems ordered by distance
- Includes min_temp_k for each system

### Reference: CLI Output Patterns
**Location**: `crates/evefrontier-cli/src/output_helpers.rs`

Patterns to follow:
- `ColorPalette` for consistent coloring
- Box-drawing characters for enhanced mode (`╔`, `║`, `╚`, etc.)
- `strip_ansi_to_string()` for width calculations
- JSON serialization via serde for `--format json`

## Decision Record

| Topic | Decision | Rationale |
|-------|----------|-----------|
| Subcommand structure | Nested `scout gates`/`scout range` | Matches semantic grouping; extensible for future scout types |
| Output formats | Reuse existing `OutputFormat` enum | Consistency with route command |
| Error handling | Reuse `fuzzy_system_matches()` pattern | Consistent UX across commands |
| Spatial index loading | Use existing `try_load_spatial_index()` | Auto-build fallback already implemented |

## Alternatives Considered

1. **Single `scout` command with mode flag**: Rejected; nested subcommands are more ergonomic
2. **Separate top-level commands (`gates`, `range`)**: Rejected; pollutes command namespace
3. **Library-level scout module**: Not needed; logic is simple enough to inline in CLI handler

## Conclusion

No unknowns remain. Implementation can proceed using established patterns from Lambda handlers and CLI output helpers.
