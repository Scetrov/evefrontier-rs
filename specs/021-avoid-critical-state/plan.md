# Implementation Plan: Avoid-critical-state (concise)

**Branch**: `021-avoid-critical-state` | **Goal**: Add a conservative per-hop avoidance check (`--avoid-critical-state`) that prevents spatial hops whose instantaneous temperature (ambient + hop delta-T) would reach or exceed `HEAT_CRITICAL`.

## Summary

Minimal, low-risk implementation that performs a per-edge check during planning when `--avoid-critical-state` is specified and a ship/loadout is provided. This avoids introducing stateful searches or heuristic changes in the MVP.

## Scope

- Library: implement check in `PathConstraints::allows` (done).
- CLI: expose `--avoid-critical-state` and require `--ship`; validate inputs (done).
- Tests: unit tests for per-hop blocking and integration tests for CLI behavior (done/added).
- Docs: update `docs/USAGE.md` and `docs/HEAT_MECHANICS.md` to describe behavior and limitations (done).

## Acceptance Criteria

- `--avoid-critical-state` without `--ship` errors with a helpful message.
- Routes that would require an instantaneous hop into CRITICAL are excluded when feasible.
- Heat calculation errors do not silently block routes (fail-safe allow and log warning).
 - Heat calculation errors conservatively **block** edges and log a warning (conservative fail-safe behavior).
- Relevant unit and integration tests pass and are included in the PR.

## Next Steps / Follow-ups

- Future work: stateful search for residual heat across hops (tracked in tasks.md as T014).

