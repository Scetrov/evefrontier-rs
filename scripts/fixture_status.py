#!/usr/bin/env python3
"""Fixture metadata helper for docs/fixtures/minimal_static_data.db.

Provides three commands:
  * status  - print current hash and table counts
  * record  - write metadata JSON to docs/fixtures/minimal_static_data.meta.json
  * verify  - ensure current fixture matches recorded metadata

The metadata captures:
  - dataset release marker (from docs/fixtures/static_data.db.release)
  - SHA-256 hash of the fixture database
  - row counts for core tables (SolarSystems, Jumps, Planets, Moons, Regions, Constellations)
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Dict, Any

REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DB = REPO_ROOT / "docs" / "fixtures" / "minimal_static_data.db"
RELEASE_MARKER = REPO_ROOT / "docs" / "fixtures" / "static_data.db.release"
METADATA_FILE = REPO_ROOT / "docs" / "fixtures" / "minimal_static_data.meta.json"
TABLES = [
    "Regions",
    "Constellations",
    "SolarSystems",
    "Jumps",
    "Planets",
    "Moons",
]


def read_release_marker() -> Dict[str, str]:
    data: Dict[str, str] = {}
    if not RELEASE_MARKER.exists():
        raise SystemExit(f"Release marker not found: {RELEASE_MARKER}")

    for line in RELEASE_MARKER.read_text().splitlines():
        line = line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        data[key.strip()] = value.strip()
    if "resolved" not in data:
        raise SystemExit("Release marker missing 'resolved=' entry")
    return data


def compute_sha256(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def count_tables(conn: sqlite3.Connection) -> Dict[str, int]:
    counts: Dict[str, int] = {}
    cur = conn.cursor()
    for table in TABLES:
        cur.execute(f"SELECT COUNT(*) FROM {table}")
        counts[table] = cur.fetchone()[0]
    return counts


def gather_metadata() -> Dict[str, Any]:
    if not FIXTURE_DB.exists():
        raise SystemExit(f"Fixture database not found: {FIXTURE_DB}")

    release = read_release_marker()
    hash_value = compute_sha256(FIXTURE_DB)
    with sqlite3.connect(FIXTURE_DB) as conn:
        counts = count_tables(conn)

    return {
        "fixture": str(FIXTURE_DB.relative_to(REPO_ROOT)),
        "release": release["resolved"],
        "sha256": hash_value,
        "tables": counts,
    }


def cmd_status() -> None:
    meta = gather_metadata()
    print(json.dumps(meta, indent=2, sort_keys=True))


def cmd_record() -> None:
    meta = gather_metadata()
    METADATA_FILE.write_text(json.dumps(meta, indent=2, sort_keys=True) + "\n")
    print(f"Recorded metadata to {METADATA_FILE}")


def cmd_verify() -> None:
    if not METADATA_FILE.exists():
        raise SystemExit(f"Metadata file missing: {METADATA_FILE}")

    current = gather_metadata()
    recorded = json.loads(METADATA_FILE.read_text())
    if current != recorded:
        print("Current fixture metadata does not match recorded metadata.")
        print("-- Recorded --")
        print(json.dumps(recorded, indent=2, sort_keys=True))
        print("-- Current --")
        print(json.dumps(current, indent=2, sort_keys=True))
        raise SystemExit(1)
    print("Fixture metadata verified.")


def main() -> None:
    parser = argparse.ArgumentParser(description="Manage EveFrontier fixture metadata")
    parser.add_argument(
        "command",
        choices=["status", "record", "verify"],
        help="Action to perform",
    )
    args = parser.parse_args()

    if args.command == "status":
        cmd_status()
    elif args.command == "record":
        cmd_record()
    else:
        cmd_verify()


if __name__ == "__main__":
    main()
