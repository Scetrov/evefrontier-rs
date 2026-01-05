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

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: The CLI `route` command MUST default to the `enhanced` output format.
- **FR-002**: The CLI `route` command MUST default to using the `Reflex` ship model for fuel
  calculations if no ship is specified.
- **FR-003**: The CLI `route` command MUST default to a fuel quality of `10`.
- **FR-004**: The CLI `route` command MUST default to avoiding systems currently in a "Critical"
  state.
- **FR-005**: The CLI `route` command MUST default to the `fuel` optimization algorithm.
- **FR-006**: The CLI `route` command MUST default the `max-spatial-neighbours` parameter to `250`.
- **FR-007**: All defaults MUST be overridable by explicit command-line flags.
- **FR-008**: System MUST provide a mechanism (e.g., `--no-avoid-critical-state`) to disable the
  default critical state avoidance.
- **FR-009**: The implementation MUST target docker contianers, the lambda functions and the CLI tool

### Key Entities

- **Routing Context**: A collection of parameters (format, ship, avoidance rules, optimization goal)
  used to determine how a path search is executed.
- **Critical State**: A status of a star system that indicates it should be bypassed for safety
  reasons by default.

## Success Criteria

1. **Equivalence**: Running `evefrontier-cli route <origin> <destination>` produces identical
   binary/logic output to running the command with all requested flags explicitly passed in the
   current version.
2. **Zero-Config Usability**: New users get a high-quality, safe, and fuel-efficient route by
   default without needing to learn complex flags.
3. **Compatibility**: Existing CLI flags continue to function as literal overrides to these new
   defaults.
4. **Visibility**: If a default is applied (like ship choice), it should be clearly indicated in the
   `enhanced` format output.

- **[Entity 2]**: [What it represents, relationships to other entities]

## Success Criteria _(mandatory)_

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on
  first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]
