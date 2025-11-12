# Test Fixtures

This directory contains the minimal test fixture database used by integration and unit tests.

## Current Fixture: `minimal_static_data.db`

The fixture is a small SQLite database with **3 test systems** and their connections:

| System ID | System Name | Connections |
|-----------|-------------|-------------|
| 100 | Y:170N | → AlphaTest, BetaTest |
| 101 | AlphaTest | → BetaTest |
| 102 | BetaTest | (endpoint) |

### Jump Graph

```
Y:170N (100)
  ├─→ AlphaTest (101) ─→ BetaTest (102)
  └─→ BetaTest (102)
```

This provides:
- Direct route: `Y:170N → BetaTest` (1 hop)
- Multi-hop route: `Y:170N → AlphaTest → BetaTest` (2 hops)
- Tests for route avoidance, algorithm selection, etc.

## ⚠️ Important: Do Not Overwrite

**This fixture is tracked in git and used by CI.** Do not replace it by running download commands with `--data-dir docs/fixtures/`.

The fixture includes a `.release` marker file (`minimal_static_data.db.release`) that prevents the CLI from attempting to re-download the dataset when using `--data-dir docs/fixtures/minimal_static_data.db`. This marker is created automatically by the CI workflow to ensure tests run entirely offline without hitting external dataset sources.

### For Local Testing with Real Data

If you need to test with production EVE Frontier data:

```bash
# Use a different directory for downloads
cargo run -p evefrontier-cli -- download --data-dir /tmp/test-data

# Or let it use the default cache location
cargo run -p evefrontier-cli -- download
```

### Regenerating the Fixture

If you need to recreate this fixture (e.g., schema changes):

```bash
python3 scripts/create_minimal_db.py
```

This script creates:
- 3 systems (Y:170N, AlphaTest, BetaTest)
- Region and constellation metadata
- Sample planets and moons
- Jumps connecting the systems
- Both `SolarSystems`/`Jumps` (new schema) and `mapSolarSystems` (legacy schema) tables

## Files

- `minimal_static_data.db` — SQLite database (committed to git)
- `minimal_static_data.db.release` — Ephemeral marker file created by the dataset download/caching mechanism (gitignored). This file only appears if a download command is accidentally run targeting this directory; it is not part of the committed fixture.

## Test Usage

All tests reference these system names:

```rust
// Library tests
let request = RouteRequest::bfs("Y:170N", "BetaTest");

// CLI tests
cmd.arg("route")
   .arg("--from").arg("Y:170N")
   .arg("--to").arg("BetaTest");
```

## Schema

The fixture includes both current and legacy schema for compatibility testing:

**Current schema:**
- `SolarSystems(solarSystemId, constellationID, regionID, name)`
- `Jumps(fromSystemId, toSystemId)`

**Legacy schema:**
- `mapSolarSystems(solarSystemID, name)`

Additional tables: `Regions`, `Constellations`, `Planets`, `Moons`
