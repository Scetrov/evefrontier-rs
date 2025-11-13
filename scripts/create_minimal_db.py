#!/usr/bin/env python3
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
