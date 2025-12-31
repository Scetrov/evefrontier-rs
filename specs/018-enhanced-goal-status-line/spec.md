# Feature Specification: Enhanced Mode GOAL Status Line

**Branch**: `018-enhanced-goal-status-line` | **Date**: 2025-12-31 | **Status**: Implemented

## Overview

Fix a bug in the `--format enhanced` output where the GOAL (final destination) step does not
display a status line showing minimum external temperature, planet count, and moon count, unlike
all other steps in the route.

Additionally, add a \"Black Hole\" indicator for the three black hole systems in the game
(30000001 A 2560, 30000002 M 974, 30000003 U 3183) which have no planets or moons.

As part of enhancing the enhanced output format, also implement color-coded fuel display
to improve visual distinction of fuel information: hop costs displayed in orange and remaining
fuel values in magenta.

## Background

In the current enhanced format output, each route step displays:
1. A tag line with the step type (STRT, JUMP, GATE, GOAL), system name, and distance
2. A status line below with system details (min temp, planets, moons)

However, the GOAL step only displays line 1 and is missing line 2.

### Current Behavior (Bug)

```
 JUMP  ● ULB-KQ6 (jump, 3ly)
       │ min  13.96K,  5 Planets, 23 Moons
 GOAL  ● M 974 (jump, 3ly)
                                          ← Missing status line!
───────────────────────────────────────
```

### Expected Behavior (Fixed)

```
 JUMP  ● ULB-KQ6 (jump, 3ly)
       │ min  13.96K,  5 Planets, 23 Moons
 GOAL  ● M 974 (jump, 3ly)
       │ Black Hole                        ← Status line with black hole indicator
───────────────────────────────────────
```

## Requirements

### Functional Requirements

1. **FR-1**: The GOAL step must display a status line with system details (min temp, planets, moons)
   exactly like all other steps in the enhanced format.
2. **FR-2**: Black hole systems (30000001, 30000002, 30000003) should display a \"Black Hole\" badge
   in an inverted magenta lozenge instead of temperature/planet/moon counts.
3. **FR-3**: Fuel information in status lines should use color coding for better visual distinction:
   - Hop cost (fuel consumed on this leg) in orange
   - Remaining fuel in magenta
4. **FR-4**: Planet and Moon count labels should maintain consistent padding alignment for both
   singular and plural forms to avoid ragged text alignment.

### Non-Functional Requirements

1. **NFR-1**: No changes to other output formats (text, rich, json, basic, emoji, note).
2. **NFR-2**: Must maintain consistent visual alignment with other status lines.

## Root Cause

In `crates/evefrontier-cli/src/output.rs`, the `EnhancedRenderer::render()` method contains:

```rust
for (i, step) in summary.steps.iter().enumerate() {
    let is_last = i + 1 == len;
    self.render_step(step, i == 0, is_last);
    if !is_last {  // ← This condition excludes the GOAL step
        self.render_step_details(step);
    }
}
```

The `if !is_last` condition was likely added to avoid a trailing status line before the footer,
but it incorrectly omits important system information for the destination.

## Solution

1. Remove the `if !is_last` condition so that `render_step_details()` is called for all steps
   including the GOAL step.
2. Add black hole detection (system IDs 30000001-30000003) and display a \"Black Hole\" badge
   for these systems instead of temperature/planet/moon information.
3. Implement color-coded fuel display using the color palette: orange for hop cost, magenta
   for remaining fuel.
4. Normalize planet/moon label widths to match the longest variant (\"Planets\"/\"Moons\" = 7/5 chars)
   to ensure alignment when switching between singular/plural forms.

## Acceptance Criteria

1. ✅ Running `evefrontier-cli route --from \"A 2560\" --to \"M 974\" --format enhanced` shows a
   status line under the GOAL step.
2. ✅ Black hole systems display \"Black Hole\" in an inverted magenta lozenge.
3. ✅ Fuel information displays with color coding (orange hop cost, magenta remaining).
4. ✅ Planet/Moon labels maintain consistent alignment (padded to plural width).
5. ✅ All existing tests continue to pass.
6. ✅ The visual alignment of the GOAL status line matches other status lines.

## Test Strategy

1. Manual verification of enhanced format output.
2. Ensure existing CLI integration tests pass (60 tests passing).

## Security Considerations

None - this is a display-only bug fix with no security implications.

## References

- `crates/evefrontier-cli/src/output.rs` - EnhancedRenderer implementation
- `crates/evefrontier-cli/src/terminal.rs` - Color palette with new TAG_BLACK_HOLE
- `docs/TODO.md` - Original bug report under "Known Issues / Tweaks"
