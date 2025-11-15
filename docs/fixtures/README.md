# Test Fixtures

This directory contains the minimal test fixture database used by integration and unit tests.

## Current Fixture: `minimal_static_data.db`

The fixture is extracted from the **e6c3 dataset** and contains **8 real EVE Frontier systems** with authentic coordinates, metadata, and connectivity:

| System ID | System Name | Gate Connections | Notes |
|-----------|-------------|------------------|-------|
| 30000191 | Nod | → D:2NAS, H:2L2S, J:35IA | Primary test system (multi-hop routing) |
| 30000200 | Brana | → G:3OA0, Y:3R7E | Secondary test system |
| 30000190 | D:2NAS | → Nod | |
| 30000198 | G:3OA0 | → Brana | |
| 30000201 | H:2L2S | → Nod, Y:3R7E | Intermediate hop system |
| 30000195 | J:35IA | → Nod | |
| 30000260 | Y:3R7E | → Brana, H:2L2S | |
| 30022425 | E1J-M5G | (none) | Spatial-only system (no gates) |

### Routing Examples

**Gate-based routes:**
- Nod → Brana (via H:2L2S → Y:3R7E, 3 hops)
- Nod → D:2NAS (direct, 1 hop)

**Spatial routing:**
- Nod ↔ Brana (~297 light-years)
- E1J-M5G can only be reached via spatial jumps

### Dataset Content

- **8 solar systems** with real e6c3 coordinates
- **12 jump gates** (bidirectional stargate connections)
- **26 planets** with orbital parameters
- **43 moons** with full metadata
- **2 regions** and **4 constellations** (real metadata)

## Fixture Generation

### From Scratch (Recommended)

To regenerate the fixture from the latest e6c3 dataset:

```bash
# 1. Download the e6c3 dataset
cargo run -p evefrontier-cli -- download --data-dir /tmp/e6c3_source

# 2. Extract the fixture subset
python3 scripts/extract_fixture_from_dataset.py /tmp/e6c3_source/static_data.db docs/fixtures/minimal_static_data.db
```

Or use the wrapper script:

```bash
python3 scripts/create_minimal_db.py /tmp/e6c3_source/static_data.db
```

### Fixture Management Helpers

The repository ships with helper commands that keep the pinned fixture in sync:

```bash
# Show current release, hash, and table counts
make fixture-status

# Recompute metadata and ensure it matches the committed values
make fixture-verify

# After regenerating the fixture, refresh the metadata JSON
make fixture-record
```

The helpers wrap `scripts/fixture_status.py`, which records the following in
`docs/fixtures/minimal_static_data.meta.json`:

- `release`: should match `static_data.db.release` (currently `e6c3`).
- `sha256`: cryptographic hash of `minimal_static_data.db`.
- `tables`: row counts for Regions, Constellations, SolarSystems, Jumps, Planets, and Moons.

CI runs a Rust integration test (`dataset_fixture_metadata.rs`) that validates the
fixture against this metadata. If the database changes without updating the metadata,
the test fails, preventing accidental drift.

### CI Usage

The CI workflow generates the fixture fresh for each test run to ensure:
- The fixture is never stale or accidentally overwritten
- Tests are reproducible and isolated
- No dependency on git LFS or large binary files

### Local Testing

The git-tracked `minimal_static_data.db` is provided for convenience during local development. To ensure consistency:

**Do not replace it by running download commands with `--data-dir docs/fixtures/`.**

The fixture includes a `.release` marker file that prevents the CLI from attempting to re-download when tests reference it.

## ⚠️ Important: Do Not Overwrite

**This fixture is tracked in git and used by CI.** Accidental overwrites can break tests. To test with production data:

```bash
# Use a different directory for downloads
evefrontier-cli download --data-dir /tmp/test-data

# Or let it use the default cache location
evefrontier-cli download
```

## Test Usage

All tests reference real system names from the e6c3 dataset:

```rust
// Library tests
let request = RouteRequest::bfs("Nod", "Brana");

// CLI tests
cmd.arg("route")
   .arg("--from").arg("Nod")
   .arg("--to").arg("Brana");
```

## Dataset Pinning

The fixture is pinned to the **e6c3 release** for stability and reproducibility. See [ADR 0011: Test Fixture Dataset Pinning](../adrs/0011-test-fixture-dataset-pinning.md) for rationale and update procedures.

### When to Update

The fixture should be regenerated when:
- CCP releases a new dataset with breaking schema changes
- New EVE Frontier features require additional test data
- An ADR documents a decision to migrate to a newer dataset version

**The dataset version is not automatically updated** to prevent CI failures and test instability.

## Extraction Criteria

The `extract_fixture_from_dataset.py` script selects systems using:

1. **Anchor systems**: Nod and Brana (predefined)
2. **Gate-connected**: All systems with direct stargate connections to anchors
3. **Spatial proximity**: All systems within 80 light-years of Brana
4. **Related data**: Regions, constellations, planets, moons, NPC stations

This ensures the fixture includes realistic multi-hop routing, spatial jumps, and metadata for comprehensive testing.

## Schema

The fixture uses the e6c3 schema:

The fixture includes both current and legacy schema for compatibility testing:

**Current schema:**
- `SolarSystems(solarSystemId, constellationID, regionID, name)`
- `Jumps(fromSystemId, toSystemId)`

**Legacy schema:**
- `mapSolarSystems(solarSystemID, name)`

Additional tables: `Regions`, `Constellations`, `Planets`, `Moons`
