#!/usr/bin/env python3
"""Inspect the schema and contents of an evefrontier SQLite database.

Supports both modern (e6c3) schema with SolarSystems/Jumps tables
and legacy schema with mapSolarSystems table.
"""
import os
import sqlite3
import sys


def get_tables(cur):
    """Return list of table names in the database."""
    return [row[0] for row in cur.execute(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )]


def table_exists(tables, name):
    """Check if a table exists (case-insensitive)."""
    return name.lower() in [t.lower() for t in tables]


def safe_query(cur, query, description):
    """Execute a query with error handling, return rows or None on failure."""
    try:
        return list(cur.execute(query))
    except sqlite3.OperationalError as e:
        print(f'\n{description}: (query failed: {e})')
        return None


def main():
    if len(sys.argv) < 2:
        print('Usage: inspect_db.py <dbfile>')
        sys.exit(2)

    db = sys.argv[1]

    # Validate file exists before attempting to connect
    if not os.path.isfile(db):
        print(f'Error: Database file not found: {db}', file=sys.stderr)
        sys.exit(1)

    try:
        conn = sqlite3.connect(db)
    except sqlite3.OperationalError as e:
        print(f'Error: Unable to open database: {e}', file=sys.stderr)
        sys.exit(1)
    cur = conn.cursor()

    tables = get_tables(cur)
    print('Tables:')
    for name in tables:
        print(' -', name)

    # Modern schema (e6c3): SolarSystems
    if table_exists(tables, 'SolarSystems'):
        print('\nSolarSystems:')
        rows = safe_query(
            cur,
            'SELECT solarSystemId, name FROM SolarSystems ORDER BY solarSystemId',
            'SolarSystems'
        )
        if rows:
            for row in rows:
                print('  ', row)

    # Legacy schema: mapSolarSystems
    if table_exists(tables, 'mapSolarSystems'):
        print('\nmapSolarSystems:')
        rows = safe_query(
            cur,
            'SELECT solarSystemID, name FROM mapSolarSystems ORDER BY solarSystemID',
            'mapSolarSystems'
        )
        if rows:
            for row in rows:
                print('  ', row)

    # Planets (if present)
    if table_exists(tables, 'Planets'):
        print('\nPlanets:')
        rows = safe_query(
            cur,
            'SELECT planetID, name, solarSystemID FROM Planets ORDER BY planetID',
            'Planets'
        )
        if rows:
            for row in rows:
                print('  ', row)

    # Moons (if present)
    if table_exists(tables, 'Moons'):
        print('\nMoons:')
        rows = safe_query(
            cur,
            'SELECT moonID, name, planetID FROM Moons ORDER BY moonID',
            'Moons'
        )
        if rows:
            for row in rows:
                print('  ', row)

    # Jumps (if present)
    if table_exists(tables, 'Jumps'):
        print('\nJumps:')
        rows = safe_query(
            cur,
            'SELECT fromSystemId, toSystemId FROM Jumps ORDER BY fromSystemId, toSystemId',
            'Jumps'
        )
        if rows:
            for row in rows:
                print('  ', row)

    conn.close()


if __name__ == '__main__':
    main()
