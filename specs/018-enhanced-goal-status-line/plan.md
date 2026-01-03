# Implementation Plan: Enhanced goal status line (concise)

**Branch**: `018-enhanced-goal-status-line` | **Goal**: Ensure the GOAL step in the enhanced route format shows a consistent status line (min temperature, planet/moon counts) and fixes padding/alignment bugs across steps.

## Summary

Small UI polish: update the route formatter to include a status line for GOAL steps, fix label padding, and ensure enhanced format remains the default display with consistent alignment. Add unit tests for formatting and update `docs/USAGE.md` examples if necessary.

## Scope

- Formatter code in `crates/evefrontier-cli/src/output.rs` (or related module)
- Unit tests for enhanced format rendering
- Verify CLI integration tests that previously relied on strict ANSI output are robust (use `--no-color` or JSON when appropriate)

## Acceptance Criteria

- GOAL step includes the status line with min temp/planet/moon counts.
- Alignment and padding matches other steps and tests pass.

