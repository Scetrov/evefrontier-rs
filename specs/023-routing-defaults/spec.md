# Feature Specification: Sensible Routing Defaults

**Feature Branch**: `023-routing-defaults`  
**Created**: 2026-01-05  
**Status**: Draft  
**Input**: User description: "set some sensible defaults for routing: --format enhanced --ship
Reflex --fuel-quality 10 --avoid-critical-state --optimize fuel --max-spatial-neighbours 250"

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Simplified Routing for Common Loadout (Priority: P1)

As a frequent pilot of a Reflex ship, I want the CLI to automatically assume my ship and preferred
routing parameters so that I don't have to type long commands every time I want to find a route.

**Why this priority**: High value for user experience and reducing command-line friction for the
most common use case.

**Independent Test**: Can be tested by running the `route` command without any optional flags and
verifying the output reflects the new defaults (enhanced format, Reflex ship, fuel optimization,
etc.).

**Acceptance Scenarios**:

1. **Given** a standard environment, **When** I run `evefrontier-cli route "Nod" "Brana"`, **Then**
   the output should show a fuel-optimized route for a Reflex ship in enhanced format.
2. **Given** the default settings, **When** the route is calculated, **Then** it should
   automatically avoid systems in a critical state unless explicitly told otherwise.

---

### User Story 2 - Explicit Control and Overrides (Priority: P2)

As a pilot using a different ship or needing a different routing strategy, I want my explicit
command-line flags to override the new defaults so that I still have full control when I need it.

**Why this priority**: Essential for maintaining existing functionality and flexibility for
non-standard use cases.

**Independent Test**: Can be tested by running `route` with flags that contradict the defaults and
verifying the results match the explicit flags.

**Acceptance Scenarios**:

1. **Given** defaults are active, **When** I run
   `evefrontier-cli route "Nod" "Brana" --ship "None" --format basic`, **Then** the output should
   use the basic format and not include ship-specific fuel calculations.
2. **Given** defaults are active, **When** I set `--fuel-quality 100`, **Then** the routing
   calculations should use the quality value of 100 instead of the default 10.

---

### User Story 3 - Disabling Default Protections (Priority: P3)

As a pilot willing to take risks, I want to be able to disable the default avoidance of systems in a
critical state so I can find faster routes if necessary.

**Why this priority**: Provides a way to bypass "safety" defaults when the user's specific context
requires it.

**Independent Test**: Verify that a "no-avoid" or "force" flag allows routing through critical
systems that would otherwise be avoided by default.

**Acceptance Scenarios**:

1. **Given** a system is in a critical state and is on the shortest path, **When** I run with
   `--no-avoid-critical-state`, **Then** the route should include that system.

---

### Edge Cases

- **Missing Ship Data**: What happens if the default ship "Reflex" is missing from the local
  `ship_data.csv`?
- **Invalid Environment**: How does the system handle these defaults if the spatial index is missing
  but required for the default `max-spatial-neighbours` search?

