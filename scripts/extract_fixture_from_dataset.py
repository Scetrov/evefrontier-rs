#!/usr/bin/env python3
"""
Extract a minimal test fixture from the full e6c3 dataset.

This script:
1. Identifies Nod, Brana, and nearby systems
2. Includes all systems connected by gates to Nod or Brana
3. Includes all systems within 80 light-years of Brana
4. Extracts all related data (regions, constellations, jumps, planets, moons)
5. Creates a minimal database suitable for testing
"""
import sqlite3
import sys
from pathlib import Path

# Light-year in meters (used for distance calculations)
LIGHT_YEAR_METERS = 9.461e15
MAX_DISTANCE_LY = 80.0

def get_target_system_ids(conn):
    """Get the IDs of our primary target systems."""
    cur = conn.cursor()
    cur.execute("SELECT solarSystemId, name FROM SolarSystems WHERE name IN ('Nod', 'Brana', 'E1J-M5G')")
    systems = {name: sid for sid, name in cur.fetchall()}
    
    if 'Nod' not in systems or 'Brana' not in systems:
        print(f"Error: Could not find required systems. Found: {list(systems.keys())}", file=sys.stderr)
        sys.exit(1)
    
    return systems

def get_gate_connected_systems(conn, target_ids):
    """Get all systems connected by gates to the target systems."""
    cur = conn.cursor()
    
    # Get systems with direct gate connections (both directions)
    placeholders = ','.join('?' * len(target_ids))
    query = f"""
        SELECT DISTINCT solarSystemId FROM (
            SELECT fromSystemId AS solarSystemId FROM Jumps WHERE toSystemId IN ({placeholders})
            UNION
            SELECT toSystemId AS solarSystemId FROM Jumps WHERE fromSystemId IN ({placeholders})
            UNION
            SELECT solarSystemId FROM SolarSystems WHERE solarSystemId IN ({placeholders})
        )
    """
    
    cur.execute(query, target_ids + target_ids + target_ids)
    return set(row[0] for row in cur.fetchall())

def get_nearby_systems(conn, brana_id, max_distance_ly):
    """Get all systems within max_distance_ly of Brana."""
    cur = conn.cursor()
    
    # Get Brana's coordinates
    cur.execute("SELECT centerX, centerY, centerZ FROM SolarSystems WHERE solarSystemId = ?", (brana_id,))
    bx, by, bz = cur.fetchone()
    
    # Find nearby systems using 3D Euclidean distance
    query = """
        SELECT solarSystemId,
               SQRT(
                   POWER(centerX - ?, 2) + 
                   POWER(centerY - ?, 2) + 
                   POWER(centerZ - ?, 2)
               ) / ? AS distance_ly
        FROM SolarSystems
        WHERE distance_ly <= ?
    """
    
    cur.execute(query, (bx, by, bz, LIGHT_YEAR_METERS, max_distance_ly))
    return set(row[0] for row in cur.fetchall())

def extract_fixture_data(source_db_path, output_db_path):
    """Extract minimal fixture from source database."""
    print(f"Opening source database: {source_db_path}")
    source = sqlite3.connect(source_db_path)
    
    # Get target systems
    target_systems = get_target_system_ids(source)
    print(f"Found target systems: {target_systems}")
    
    # Collect all systems to include
    brana_id = target_systems['Brana']
    nod_id = target_systems['Nod']
    target_ids = list(target_systems.values())
    
    print("Finding gate-connected systems...")
    gate_systems = get_gate_connected_systems(source, target_ids)
    print(f"  Found {len(gate_systems)} systems connected by gates")
    
    print(f"Finding systems within {MAX_DISTANCE_LY} ly of Brana...")
    nearby_systems = get_nearby_systems(source, brana_id, MAX_DISTANCE_LY)
    print(f"  Found {len(nearby_systems)} systems within range")
    
    # Combine all systems
    all_systems = gate_systems | nearby_systems
    print(f"\nTotal systems to include: {len(all_systems)}")
    
    # Get system names for verification
    cur = source.cursor()
    placeholders = ','.join('?' * len(all_systems))
    cur.execute(f"SELECT solarSystemId, name FROM SolarSystems WHERE solarSystemId IN ({placeholders}) ORDER BY name", 
                tuple(all_systems))
    system_names = cur.fetchall()
    print(f"\nSystems included:")
    for sid, name in system_names[:20]:  # Show first 20
        print(f"  {name} ({sid})")
    if len(system_names) > 20:
        print(f"  ... and {len(system_names) - 20} more")
    
    # Create output database
    print(f"\nCreating output database: {output_db_path}")
    if Path(output_db_path).exists():
        Path(output_db_path).unlink()
    
    output = sqlite3.connect(output_db_path)
    out_cur = output.cursor()
    
    # Copy schema (get CREATE TABLE statements)
    cur.execute("SELECT sql FROM sqlite_master WHERE type='table' AND sql IS NOT NULL ORDER BY name")
    for (create_sql,) in cur.fetchall():
        out_cur.execute(create_sql)
    
    # Get region and constellation IDs for these systems
    cur.execute(f"""
        SELECT DISTINCT regionId, constellationId 
        FROM SolarSystems 
        WHERE solarSystemId IN ({placeholders})
    """, tuple(all_systems))
    region_constellation_ids = cur.fetchall()
    region_ids = set(r[0] for r in region_constellation_ids)
    constellation_ids = set(r[1] for r in region_constellation_ids)
    
    # Copy Regions
    region_ph = ','.join('?' * len(region_ids))
    cur.execute(f"SELECT * FROM Regions WHERE regionId IN ({region_ph})", tuple(region_ids))
    regions = cur.fetchall()
    if regions:
        out_cur.executemany(f"INSERT INTO Regions VALUES ({','.join('?' * len(regions[0]))})", regions)
    print(f"Copied {len(regions)} regions")
    
    # Copy Constellations
    const_ph = ','.join('?' * len(constellation_ids))
    cur.execute(f"SELECT * FROM Constellations WHERE constellationId IN ({const_ph})", tuple(constellation_ids))
    constellations = cur.fetchall()
    if constellations:
        out_cur.executemany(f"INSERT INTO Constellations VALUES ({','.join('?' * len(constellations[0]))})", constellations)
    print(f"Copied {len(constellations)} constellations")
    
    # Copy SolarSystems
    cur.execute(f"SELECT * FROM SolarSystems WHERE solarSystemId IN ({placeholders})", tuple(all_systems))
    systems = cur.fetchall()
    if systems:
        out_cur.executemany(f"INSERT INTO SolarSystems VALUES ({','.join('?' * len(systems[0]))})", systems)
    print(f"Copied {len(systems)} solar systems")
    
    # Copy Jumps (only between included systems)
    cur.execute(f"""
        SELECT * FROM Jumps 
        WHERE fromSystemId IN ({placeholders}) 
          AND toSystemId IN ({placeholders})
    """, tuple(all_systems) + tuple(all_systems))
    jumps = cur.fetchall()
    if jumps:
        out_cur.executemany(f"INSERT INTO Jumps VALUES ({','.join('?' * len(jumps[0]))})", jumps)
    print(f"Copied {len(jumps)} jumps")
    
    # Copy Planets
    cur.execute(f"SELECT * FROM Planets WHERE solarSystemId IN ({placeholders})", tuple(all_systems))
    planets = cur.fetchall()
    planet_ids = set(p[0] for p in planets)
    if planets:
        out_cur.executemany(f"INSERT INTO Planets VALUES ({','.join('?' * len(planets[0]))})", planets)
    print(f"Copied {len(planets)} planets")
    
    # Copy Moons
    if planet_ids:
        planet_ph = ','.join('?' * len(planet_ids))
        cur.execute(f"SELECT * FROM Moons WHERE planetId IN ({planet_ph})", tuple(planet_ids))
        moons = cur.fetchall()
        if moons:
            out_cur.executemany(f"INSERT INTO Moons VALUES ({','.join('?' * len(moons[0]))})", moons)
        print(f"Copied {len(moons)} moons")
    
    # Copy NpcStations if they exist
    try:
        cur.execute(f"SELECT * FROM NpcStations WHERE solarSystemId IN ({placeholders})", tuple(all_systems))
        stations = cur.fetchall()
        if stations:
            out_cur.executemany(f"INSERT INTO NpcStations VALUES ({','.join('?' * len(stations[0]))})", stations)
        print(f"Copied {len(stations)} NPC stations")
    except sqlite3.OperationalError:
        print("NpcStations table not found in source (skipping)")
    
    output.commit()
    output.close()
    source.close()
    
    print(f"\n✓ Fixture created successfully: {output_db_path}")
    print(f"  Total systems: {len(all_systems)}")
    print(f"  Total jumps: {len(jumps)}")

if __name__ == '__main__':
    import sys
    
    if len(sys.argv) > 1:
        source_path = sys.argv[1]
    else:
        source_path = '/tmp/e6c3_source/static_data.db'
    
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
    else:
        output_path = 'docs/fixtures/minimal/static_data.db'
    
    if not Path(source_path).exists():
        print(f"Error: Source database not found: {source_path}", file=sys.stderr)
        print(f"Usage: {sys.argv[0]} [source_db] [output_db]", file=sys.stderr)
        print(f"  source_db: Path to full e6c3 dataset (default: /tmp/e6c3_source/static_data.db)", file=sys.stderr)
        print(f"  output_db: Path to output fixture (default: docs/fixtures/minimal/static_data.db)", file=sys.stderr)
        sys.exit(1)
    
    extract_fixture_data(source_path, output_path)
