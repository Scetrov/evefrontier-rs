# Test Fixtures

This directory contains test fixture databases used by integration and unit tests.

## Fixtures Overview

| Fixture | Systems | Jumps | Purpose |
|---------|---------|-------|---------|
| `minimal_static_data.db` | 8 | 12 | Core routing tests, schema validation |
| `route_testing.db` | 717 | 344 | Sample route validation (~50% coverage) |
| `ship_data.csv` | n/a | n/a | Ship catalog fixture for fuel projection tests |

## Current Fixtures

### 1. `minimal_static_data.db` - Core Fixture

A minimal fixture with 8 real EVE Frontier systems for unit testing:

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

### 2. `route_testing.db` - Sample Routes Fixture

A larger fixture containing 717 systems that cover ~50% of routes from `SampleRoutes.csv`:

- **717 solar systems** extracted from e6c3 dataset
- **344 jump gates** between these systems
- **2,700 planets** and **5,122 moons**
- **546 testable routes** (50.5% of sample routes)

The systems were selected by analyzing `SampleRoutes.csv` and extracting systems that appear
in 4+ route paths (the "corridor" systems).

## 3. `ship_data.csv` - Ship Catalog Fixture

A minimal ship catalog for fuel projection testing:

- **Ships included:** Reflex, and other EVE Frontier ships (see file for complete list)
- **Default test ship:** Reflex (used in CLI/Lambda tests with 10% fuel quality default)
- **Schema:** Base mass (kg), fuel capacity, cargo capacity, specific heat, max heat tolerance,
  heat dissipation rate

**Protection note:** This fixture should not be overwritten by dataset downloads. The library
includes a safety guard (`ensure_e6c3_dataset`) that rejects download targets resolving to
`docs/fixtures/minimal_static_data.db`, preventing accidental fixture replacement.

**Using the fixture in tests:**

```rust
use std::path::PathBuf;
use evefrontier_lib::ship::ShipCatalog;

fn fixture_path() -> PathBuf {
   PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../docs/fixtures/ship_data.csv")
}

#[test]
fn test_with_ship_catalog() {
   let catalog = ShipCatalog::from_path(&fixture_path())
      .expect("fixture should load");
   let reflex = catalog.get("Reflex").expect("Reflex available");
   assert!(reflex.base_mass_kg > 0.0);
}
```

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

### Route Testing Fixture

To regenerate the route testing fixture (requires `SampleRoutes.csv` in `docs/`):

```bash
# Download the dataset first
cargo run -p evefrontier-cli -- download

# Extract the fixture with ~50% route coverage (threshold 4+)
python3 scripts/extract_route_fixture.py --threshold 4

# Or for different coverage levels:
python3 scripts/extract_route_fixture.py --threshold 2  # ~73% coverage, 1443 systems
python3 scripts/extract_route_fixture.py --threshold 3  # ~58% coverage, 973 systems
python3 scripts/extract_route_fixture.py --threshold 5  # ~41% coverage, 578 systems
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

> [!IMPORTANT]
> **Do not replace it by running download commands with `--data-dir docs/fixtures/`.**
>
> The fixture includes a `.release` marker file that prevents the CLI from attempting to
> re-download when tests reference it.

> [!WARNING]
> **This fixture is tracked in git and used by CI.** Accidental overwrites can break tests. To
> test with production data:
>
> ```bash
> # Use a different directory for downloads
> evefrontier-cli download --data-dir /tmp/test-data
>
> # Or let it use the default cache location
> evefrontier-cli download
> ```

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

### 3. `ship_data.csv` - Ship Catalog Fixture

- **Fields**: `name,base_mass_kg,specific_heat,fuel_capacity,cargo_capacity,max_heat_tolerance,heat_dissipation_rate`
- **Entries**: 3 representative ships (Reflex, Forager, Warden) with validated positive values
- **Checksum**: `ship_data.csv.sha256` (refresh when regenerating the CSV)

> [!IMPORTANT]
> Do not overwrite `ship_data.csv` with downloaded data without updating the checksum.
> Use a separate path for ad-hoc downloads to avoid breaking tests.

#### Regeneration

If the ship catalog changes in a dataset release:

```bash
# Example: refresh ship fixture from a dataset source
python3 scripts/extract_ship_fixture.py /path/to/release/ship_data.csv docs/fixtures/ship_data.csv

# Recompute checksum
sha256sum docs/fixtures/ship_data.csv > docs/fixtures/ship_data.csv.sha256
```

Update fuel-related tests after refreshing the fixture to match the new data.
