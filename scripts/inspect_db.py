#!/usr/bin/env python3
import sqlite3
import sys

if len(sys.argv) < 2:
    print('Usage: inspect_db.py <dbfile>')
    sys.exit(2)

db = sys.argv[1]
conn = sqlite3.connect(db)
cur = conn.cursor()

print('Tables:')
for row in cur.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"):
    print(' -', row[0])

print('\nSolarSystems:')
for row in cur.execute('SELECT solarSystemId, name FROM SolarSystems ORDER BY solarSystemId'):
    print('  ', row)

print('\nmapSolarSystems:')
for row in cur.execute('SELECT solarSystemID, name FROM mapSolarSystems ORDER BY solarSystemID'):
    print('  ', row)

print('\nPlanets:')
for row in cur.execute('SELECT planetID, name, solarSystemID FROM Planets ORDER BY planetID'):
    print('  ', row)

print('\nMoons:')
for row in cur.execute('SELECT moonID, name, planetID FROM Moons ORDER BY moonID'):
    print('  ', row)

print('\nJumps:')
for row in cur.execute('SELECT fromSystemId, toSystemId FROM Jumps ORDER BY fromSystemId, toSystemId'):
    print('  ', row)

conn.close()
