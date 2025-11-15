#!/usr/bin/env python3
"""
Create a minimal test fixture database from the e6c3 dataset.

This script is now a wrapper that calls extract_fixture_from_dataset.py
to generate a fixture based on real EVE Frontier data.

The fixture includes:
- Nod, Brana, E1J-M5G (target systems)
- All systems connected by gates to Nod or Brana
- All systems within 80 light-years of Brana

This approach ensures tests use realistic data while remaining fast and
reproducible. The e6c3 dataset is pinned as the reference version.

Usage:
    python3 scripts/create_minimal_db.py [source_db_path]

If source_db_path is not provided, the script will attempt to use
/tmp/e6c3_source/static_data.db (download it first with the CLI).
"""
import subprocess
import sys
from pathlib import Path

def main():
    script_dir = Path(__file__).parent
    extract_script = script_dir / 'extract_fixture_from_dataset.py'
    
    if not extract_script.exists():
        print(f"Error: Extract script not found: {extract_script}", file=sys.stderr)
        sys.exit(1)
    
    # Determine source database path
    if len(sys.argv) > 1:
        source_db = sys.argv[1]
    else:
        source_db = '/tmp/e6c3_source/static_data.db'
    
    if not Path(source_db).exists():
        print(f"Error: Source database not found: {source_db}", file=sys.stderr)
        print("\nTo download the e6c3 dataset, run:", file=sys.stderr)
        print("  cargo run -p evefrontier-cli -- download --data-dir /tmp/e6c3_source", file=sys.stderr)
        sys.exit(1)
    
    output_db = script_dir.parent / 'docs' / 'fixtures' / 'minimal_static_data.db'
    
    print(f"Extracting fixture from {source_db}...")
    result = subprocess.run(
        [sys.executable, str(extract_script), source_db, str(output_db)],
        check=False
    )
    
    sys.exit(result.returncode)

if __name__ == '__main__':
    main()


# ============================================================================
# LEGACY SYNTHETIC DATA GENERATION CODE (DEPRECATED)
# ============================================================================
# The code below is preserved for reference but is no longer used.
# The fixture is now generated from real e6c3 data using
# extract_fixture_from_dataset.py
# ============================================================================

def create_legacy_synthetic_fixture():
    """Legacy code that created synthetic test data (no longer used)."""
    import sqlite3
    import os

    out_dir = os.path.join('docs','fixtures')
    os.makedirs(out_dir, exist_ok=True)
    db_path = os.path.join(out_dir, 'minimal_static_data.db')
    if os.path.exists(db_path):
        os.remove(db_path)

    conn = sqlite3.connect(db_path)
    cur = conn.cursor()

    # Create tables
    cur.executescript(r'''
CREATE TABLE Regions(
  regionID INTEGER PRIMARY KEY,
  regionName TEXT NOT NULL
);

CREATE TABLE Constellations(
  constellationID INTEGER PRIMARY KEY,
  regionID INTEGER NOT NULL,
  constellationName TEXT NOT NULL,
  FOREIGN KEY(regionID) REFERENCES Regions(regionID)
);

CREATE TABLE SolarSystems(
  solarSystemId INTEGER PRIMARY KEY,
  constellationID INTEGER NOT NULL,
  regionID INTEGER NOT NULL,
  name TEXT NOT NULL,
  -- include coordinates so spatial routing works by default
  x REAL NOT NULL,
  y REAL NOT NULL,
  z REAL NOT NULL,
  FOREIGN KEY(constellationID) REFERENCES Constellations(constellationID),
  FOREIGN KEY(regionID) REFERENCES Regions(regionID)
);

-- older schema compatibility
CREATE TABLE mapSolarSystems(
  solarSystemID INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE Planets(
  planetID INTEGER PRIMARY KEY,
  solarSystemID INTEGER NOT NULL,
  name TEXT NOT NULL,
  FOREIGN KEY(solarSystemID) REFERENCES SolarSystems(solarSystemId)
);

CREATE TABLE Moons(
  moonID INTEGER PRIMARY KEY,
  planetID INTEGER NOT NULL,
  name TEXT NOT NULL,
  FOREIGN KEY(planetID) REFERENCES Planets(planetID)
);

CREATE TABLE Jumps(
  fromSystemId INTEGER NOT NULL,
  toSystemId INTEGER NOT NULL
);
''')

    # Insert a single region and constellation
    cur.execute('INSERT INTO Regions(regionID, regionName) VALUES (?,?)', (1, 'TestRegion'))
    cur.execute('INSERT INTO Constellations(constellationID, regionID, constellationName) VALUES (?,?,?)', (10, 1, 'TestConstellation'))

    # Insert systems: include Y:170N in this constellation
    systems = [
      (100, 10, 1, 'Y:170N', 0.0, 0.0, 0.0),
      (101, 10, 1, 'AlphaTest', 10.0, 0.0, 0.0),
      (102, 10, 1, 'BetaTest', 20.0, 0.0, 0.0)
    ]
    cur.executemany('INSERT INTO SolarSystems(solarSystemId, constellationID, regionID, name, x, y, z) VALUES (?,?,?,?,?,?,?)', systems)
    # also populate legacy table
    cur.executemany('INSERT INTO mapSolarSystems(solarSystemID, name) VALUES (?,?)', [(s[0], s[3]) for s in systems])

    # Planets and moons for Y:170N (system 100)
    planets = [
        (1001, 100, 'Y-Prime'),
        (1002, 100, 'Y-Secondary')
    ]
    cur.executemany('INSERT INTO Planets(planetID, solarSystemID, name) VALUES (?,?,?)', planets)

    moons = [
        (2001, 1001, 'Y-Prime-MoonA'),
        (2002, 1002, 'Y-Secondary-MoonA')
    ]
    cur.executemany('INSERT INTO Moons(moonID, planetID, name) VALUES (?,?,?)', moons)

    # Jumps (simple connections)
    jumps = [
        (100, 101),
        (101, 102),
        (100, 102)
    ]
    cur.executemany('INSERT INTO Jumps(fromSystemId, toSystemId) VALUES (?,?)', jumps)

    conn.commit()
    conn.close()
    print('Created', db_path)

