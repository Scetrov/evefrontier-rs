## ADDED Requirements

### Requirement: Fmap waypoint-count integrity
Fmap encoding MUST reject waypoint collections whose length cannot be represented by the format's unsigned 16-bit count field and SHALL preserve exact encode/decode round trips for supported collections.

#### Scenario: Supported waypoint collection is encoded
- **WHEN** a valid collection contains at most `u16::MAX` waypoints
- **THEN** decoding the encoded token reproduces the original count, system IDs, and waypoint types

#### Scenario: Oversized waypoint collection is encoded
- **WHEN** a collection contains more than `u16::MAX` waypoints
- **THEN** encoding returns a typed error without truncating the count or producing a token

### Requirement: Safe spatial-index size parsing
File-backed spatial-index loading MUST validate the minimum header, metadata, payload, and checksum lengths with checked arithmetic before allocating a compressed-data buffer.

#### Scenario: Truncated index has a plausible header
- **WHEN** an index file contains a recognized header but is too short for its declared sections and checksum
- **THEN** loading returns a typed spatial-index error without arithmetic underflow, panic, or oversized allocation

### Requirement: Bounded spatial-neighbor request
Externally supplied Lambda `max_spatial_neighbors` values MUST be validated against the library's documented safe maximum before graph construction, and graph arithmetic SHALL remain overflow-safe for direct library callers.

#### Scenario: Lambda neighbor count is supported
- **WHEN** a route request supplies a neighbor count within the supported range or uses an existing documented default/unlimited sentinel
- **THEN** validation preserves the existing routing semantics

#### Scenario: Lambda neighbor count exceeds the safe maximum
- **WHEN** a route request supplies a value above the documented safe maximum, including `usize::MAX`
- **THEN** request validation rejects it with a client-facing problem response before graph construction

#### Scenario: Library caller supplies an extreme value
- **WHEN** graph construction receives a value that would overflow when accounting for the source node
- **THEN** graph construction returns or applies a bounded result without overflow or panic

### Requirement: Focused Rust fuzz targets
The repository SHALL provide coverage-guided Rust fuzz targets for fmap token decoding and round trips, spatial-index byte loading, and local dataset ZIP extraction.

#### Scenario: Arbitrary malformed input is exercised
- **WHEN** a fuzz target receives arbitrary bytes or structured mutations
- **THEN** the target accepts a valid result or typed error without panic, out-of-directory write, or violated format invariant

#### Scenario: Valid corpus seed is exercised
- **WHEN** a committed valid token, spatial index, or dataset archive seed runs
- **THEN** the corresponding parser succeeds and the fuzz oracle validates its structural invariants

### Requirement: Bounded fuzz orchestration
Fuzz tasks SHALL run through non-cacheable Nx targets with explicit time and resource budgets, SHALL be invocable manually and on a schedule, and SHALL preserve minimized crash inputs as deterministic regression tests.

#### Scenario: Scheduled fuzzing runs
- **WHEN** the scheduled or manually dispatched fuzz workflow executes
- **THEN** each configured target runs within its budget and publishes actionable crash artifacts on failure

#### Scenario: A fuzz defect is fixed
- **WHEN** a crash input demonstrates a repository defect
- **THEN** the minimized input or equivalent case is added to deterministic tests before the defect is considered resolved
