#!/usr/bin/env python3
"""
Extract a route-testing fixture from the full e6c3 dataset.

This script extracts systems needed to test a percentage of routes in SampleRoutes.csv.
It identifies systems appearing frequently in discovered paths.

Usage:
    python extract_route_fixture.py [--threshold N] [--output FILENAME]

    --threshold N   Minimum occurrences for a system to be included (default: 4 for ~50%)
    --output NAME   Output database filename in docs/fixtures/ (default: route_testing.db)
"""
import sqlite3
import csv
import json
import sys
from pathlib import Path
from collections import Counter
import argparse

# Light-year in meters (used for distance calculations)
LIGHT_YEAR_METERS = 9.461e15

def analyze_sample_routes(csv_path, threshold=4):
    """Analyze SampleRoutes.csv to find corridor systems.

    Args:
        csv_path: Path to SampleRoutes.csv
        threshold: Minimum number of occurrences for a system to be included

    Returns:
        Tuple of (corridor_systems set, testable_routes list, all_systems dict, all_routes list)
    """
    systems = {}
    routes = []
    all_system_ids = []

    with open(csv_path, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            route_id = int(row['routeId'])
            path = json.loads(row['discoveredPath'])

            path_ids = []
            for hop in path:
                systems[hop['Id']] = hop['Name']
                all_system_ids.append(hop['Id'])
                path_ids.append(hop['Id'])

            routes.append({
                'route_id': route_id,
                'start_id': path_ids[0],
                'end_id': path_ids[-1],
                'avoid_gates': row['avoidGates'] == 'true',
                'max_ly': float(row['maxLightyears']),
                'path_ids': path_ids
            })

    # Count all system occurrences across all routes
    counts = Counter(all_system_ids)

    # Get corridor systems (appearing threshold+ times)
    corridor_systems = {sys_id for sys_id, count in counts.items() if count >= threshold}

    # Find routes fully contained in corridor
    corridor_routes = [r for r in routes if all(sid in corridor_systems for sid in r['path_ids'])]

    print(f"Total routes: {len(routes)}")
    print(f"Unique systems: {len(systems)}")
    print(f"Corridor systems ({threshold}+ occurrences): {len(corridor_systems)}")
    print(f"Routes testable with corridor: {len(corridor_routes)} ({100*len(corridor_routes)/len(routes):.1f}%)")

    return corridor_systems, corridor_routes, systems, routes


def extract_fixture_data(source_db_path, output_db_path, system_ids, system_names):
    """Extract fixture data for the given system IDs."""
    print(f"Opening source database: {source_db_path}")
    source = sqlite3.connect(source_db_path)

    # Convert to list for SQL queries
    system_id_list = sorted(system_ids)

    print(f"Extracting {len(system_id_list)} systems...")

    # Get all system data
    cur = source.cursor()
    placeholders = ','.join('?' * len(system_id_list))

    # Get constellation and region IDs for these systems
    cur.execute(f"""
        SELECT DISTINCT constellationId, regionId
        FROM SolarSystems
        WHERE solarSystemId IN ({placeholders})
    """, system_id_list)
    constellation_region = cur.fetchall()
    constellation_ids = list(set(r[0] for r in constellation_region))
    region_ids = list(set(r[1] for r in constellation_region))

    print(f"Found {len(constellation_ids)} constellations, {len(region_ids)} regions")

    # Get jumps between these systems
    cur.execute(f"""
        SELECT fromSystemId, toSystemId
        FROM Jumps
        WHERE fromSystemId IN ({placeholders}) AND toSystemId IN ({placeholders})
    """, system_id_list + system_id_list)
    jumps = cur.fetchall()
    print(f"Found {len(jumps)} jump gates")

    # Create output database
    print(f"Creating output database: {output_db_path}")
    if output_db_path.exists():
        output_db_path.unlink()

    output = sqlite3.connect(output_db_path)
    out_cur = output.cursor()

    # Create tables (matching e6c3 schema with temperature columns)
    out_cur.executescript("""
        CREATE TABLE Regions (
            regionId INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );

        CREATE TABLE Constellations (
            constellationId INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            regionId INTEGER NOT NULL,
            FOREIGN KEY (regionId) REFERENCES Regions(regionId)
        );

        CREATE TABLE SolarSystems (
            solarSystemId INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            regionId INTEGER NOT NULL,
            constellationId INTEGER NOT NULL,
            centerX REAL NOT NULL,
            centerY REAL NOT NULL,
            centerZ REAL NOT NULL,
            star_temperature REAL,
            star_luminosity REAL,
            FOREIGN KEY (regionId) REFERENCES Regions(regionId),
            FOREIGN KEY (constellationId) REFERENCES Constellations(constellationId)
        );

        CREATE TABLE Jumps (
            fromSystemId INTEGER NOT NULL,
            toSystemId INTEGER NOT NULL,
            PRIMARY KEY (fromSystemId, toSystemId),
            FOREIGN KEY (fromSystemId) REFERENCES SolarSystems(solarSystemId),
            FOREIGN KEY (toSystemId) REFERENCES SolarSystems(solarSystemId)
        );

        CREATE TABLE Planets (
            planetId INTEGER PRIMARY KEY,
            solarSystemId INTEGER NOT NULL,
            name TEXT,
            celestialIndex INTEGER,
            typeId INTEGER,
            centerX REAL,
            centerY REAL,
            centerZ REAL,
            radius REAL,
            density REAL,
            eccentricity REAL,
            escapeVelocity REAL,
            surfaceGravity REAL,
            temperature REAL,
            pressure REAL,
            orbitRadius REAL,
            orbitPeriod REAL,
            rotationRate REAL,
            mass REAL,
            typeDescription TEXT,
            FOREIGN KEY (solarSystemId) REFERENCES SolarSystems(solarSystemId)
        );

        CREATE TABLE Moons (
            moonId INTEGER PRIMARY KEY,
            planetId INTEGER NOT NULL,
            name TEXT,
            solarSystemId INTEGER,
            typeId INTEGER,
            centerX REAL,
            centerY REAL,
            centerZ REAL,
            radius REAL,
            density REAL,
            eccentricity REAL,
            escapeVelocity REAL,
            surfaceGravity REAL,
            temperature REAL,
            pressure REAL,
            orbitRadius REAL,
            orbitPeriod REAL,
            rotationRate REAL,
            mass REAL,
            spectralClass TEXT,
            typeDescription TEXT,
            FOREIGN KEY (planetId) REFERENCES Planets(planetId)
        );
    """)

    # Copy regions
    region_placeholders = ','.join('?' * len(region_ids))
    cur.execute(f"SELECT regionId, name FROM Regions WHERE regionId IN ({region_placeholders})", region_ids)
    regions = cur.fetchall()
    out_cur.executemany("INSERT INTO Regions (regionId, name) VALUES (?, ?)", regions)
    print(f"Copied {len(regions)} regions")

    # Copy constellations
    const_placeholders = ','.join('?' * len(constellation_ids))
    cur.execute(f"""
        SELECT constellationId, name, regionId
        FROM Constellations
        WHERE constellationId IN ({const_placeholders})
    """, constellation_ids)
    constellations = cur.fetchall()
    out_cur.executemany("INSERT INTO Constellations (constellationId, name, regionId) VALUES (?, ?, ?)", constellations)
    print(f"Copied {len(constellations)} constellations")

    # Copy solar systems - check for temperature columns
    cur.execute("PRAGMA table_info(SolarSystems)")
    columns = {row[1] for row in cur.fetchall()}
    has_star_temperature = 'star_temperature' in columns
    has_star_luminosity = 'star_luminosity' in columns

    if has_star_temperature and has_star_luminosity:
        cur.execute(f"""
            SELECT solarSystemId, name, regionId, constellationId, centerX, centerY, centerZ,
                   star_temperature, star_luminosity
            FROM SolarSystems
            WHERE solarSystemId IN ({placeholders})
        """, system_id_list)
    else:
        cur.execute(f"""
            SELECT solarSystemId, name, regionId, constellationId, centerX, centerY, centerZ,
                   NULL as star_temperature, NULL as star_luminosity
            FROM SolarSystems
            WHERE solarSystemId IN ({placeholders})
        """, system_id_list)
    solar_systems = cur.fetchall()
    out_cur.executemany("""
        INSERT INTO SolarSystems (solarSystemId, name, regionId, constellationId, centerX, centerY, centerZ, star_temperature, star_luminosity)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, solar_systems)
    print(f"Copied {len(solar_systems)} solar systems")

    # Copy jumps
    out_cur.executemany("INSERT INTO Jumps (fromSystemId, toSystemId) VALUES (?, ?)", jumps)
    print(f"Copied {len(jumps)} jumps")

    # Copy planets for these systems (full schema)
    cur.execute(f"""
        SELECT planetId, solarSystemId, name, celestialIndex, typeId, centerX, centerY, centerZ,
               radius, density, eccentricity, escapeVelocity, surfaceGravity, temperature,
               pressure, orbitRadius, orbitPeriod, rotationRate, mass, typeDescription
        FROM Planets
        WHERE solarSystemId IN ({placeholders})
    """, system_id_list)
    planets = cur.fetchall()
    out_cur.executemany("""
        INSERT INTO Planets (planetId, solarSystemId, name, celestialIndex, typeId, centerX, centerY, centerZ,
                            radius, density, eccentricity, escapeVelocity, surfaceGravity, temperature,
                            pressure, orbitRadius, orbitPeriod, rotationRate, mass, typeDescription)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, planets)
    print(f"Copied {len(planets)} planets")

    # Copy moons for these planets (full schema)
    planet_ids = [p[0] for p in planets]
    if planet_ids:
        planet_placeholders = ','.join('?' * len(planet_ids))
        cur.execute(f"""
            SELECT moonId, planetId, name, solarSystemId, typeId, centerX, centerY, centerZ,
                   radius, density, eccentricity, escapeVelocity, surfaceGravity, temperature,
                   pressure, orbitRadius, orbitPeriod, rotationRate, mass, spectralClass, typeDescription
            FROM Moons
            WHERE planetId IN ({planet_placeholders})
        """, planet_ids)
        moons = cur.fetchall()
        out_cur.executemany("""
            INSERT INTO Moons (moonId, planetId, name, solarSystemId, typeId, centerX, centerY, centerZ,
                              radius, density, eccentricity, escapeVelocity, surfaceGravity, temperature,
                              pressure, orbitRadius, orbitPeriod, rotationRate, mass, spectralClass, typeDescription)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, moons)
        print(f"Copied {len(moons)} moons")

    output.commit()
    output.close()
    source.close()

    return len(solar_systems), len(jumps)


def create_metadata(output_path, system_count, jump_count, route_count, system_names):
    """Create metadata JSON file for the fixture."""
    import hashlib
    import os

    # Calculate file hash
    with open(output_path, 'rb') as f:
        file_hash = hashlib.sha256(f.read()).hexdigest()

    file_size = os.path.getsize(output_path)

    # Sample system names for documentation
    sample_systems = list(system_names.values())[:10]

    metadata = {
        "source": "e6c3 dataset via extract_route_fixture.py",
        "description": f"Route testing fixture - systems appearing {4}+ times for testing ~50% of SampleRoutes.csv",
        "systems_count": system_count,
        "jumps_count": jump_count,
        "testable_routes": route_count,
        "sample_systems": sample_systems,
        "file_size_bytes": file_size,
        "sha256": file_hash
    }

    meta_path = output_path.with_suffix('.meta.json')
    with open(meta_path, 'w') as f:
        json.dump(metadata, f, indent=2)

    print(f"Created metadata: {meta_path}")
    return metadata


def main():
    parser = argparse.ArgumentParser(
        description="Extract route testing fixture from e6c3 dataset",
        epilog="Thresholds: 4 = ~50%, 5 = ~40%, 3 = ~58%, 2 = ~73%"
    )
    parser.add_argument('--threshold', type=int, default=4,
                       help='Minimum system occurrences to include (default: 4 for ~50%%)')
    parser.add_argument('--output', type=str, default='route_testing.db',
                       help='Output filename in docs/fixtures/ (default: route_testing.db)')
    args = parser.parse_args()

    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    # Find source database (e6c3 dataset)
    source_candidates = [
        Path.home() / ".local/share/evefrontier/static_data.db",
        Path.home() / ".cache/evefrontier_datasets/static_data.db",
        project_root / "static_data.db",
    ]

    source_db = None
    for candidate in source_candidates:
        if candidate.exists():
            source_db = candidate
            break

    if not source_db:
        print("Error: Could not find source e6c3 database.", file=sys.stderr)
        print("Checked:", file=sys.stderr)
        for c in source_candidates:
            print(f"  {c}", file=sys.stderr)
        sys.exit(1)

    csv_path = project_root / "docs" / "SampleRoutes.csv"
    if not csv_path.exists():
        print(f"Error: Could not find {csv_path}", file=sys.stderr)
        sys.exit(1)

    output_path = project_root / "docs" / "fixtures" / args.output

    # Analyze routes with configurable threshold
    corridor_systems, corridor_routes, system_names, all_routes = analyze_sample_routes(csv_path, args.threshold)

    # Filter to only corridor system names
    corridor_names = {sid: system_names[sid] for sid in corridor_systems}

    # Extract fixture
    system_count, jump_count = extract_fixture_data(source_db, output_path, corridor_systems, corridor_names)

    # Create metadata
    metadata = create_metadata(output_path, system_count, jump_count, len(corridor_routes), corridor_names)

    print("\n" + "="*60)
    print("FIXTURE CREATED SUCCESSFULLY")
    print("="*60)
    print(f"Output: {output_path}")
    print(f"Systems: {system_count}")
    print(f"Jumps: {jump_count}")
    print(f"Testable routes: {len(corridor_routes)} ({100*len(corridor_routes)/len(all_routes):.1f}% of total)")


if __name__ == '__main__':
    main()
