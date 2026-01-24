#!/usr/bin/env python3
"""Analyze SampleRoutes.csv to find a representative subset of systems for testing."""

import csv
import json
from collections import Counter
from pathlib import Path

def main():
    csv_path = Path(__file__).parent.parent / "docs" / "SampleRoutes.csv"

    # Parse CSV
    systems = {}
    routes = []
    all_system_ids = []

    with open(csv_path, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            route_id = int(row['routeId'])
            start_id = int(row['startSolarSystemId'])
            end_id = int(row['endSolarSystemId'])
            avoid_gates = row['avoidGates'] == 'true'
            max_ly = float(row['maxLightyears'])

            # Parse path
            path = json.loads(row['discoveredPath'])
            for hop in path:
                systems[hop['Id']] = hop['Name']
                all_system_ids.append(hop['Id'])

            routes.append({
                'route_id': route_id,
                'start_id': start_id,
                'start_name': systems[start_id],
                'end_id': end_id,
                'end_name': systems[end_id],
                'avoid_gates': avoid_gates,
                'max_ly': max_ly,
                'path_len': len(path),
                'path_ids': [hop['Id'] for hop in path]
            })

    print(f'Total routes: {len(routes)}')
    print(f'Unique systems: {len(systems)}')

    # Find commonly used systems (hubs)
    counts = Counter(all_system_ids)
    print('\nMost common systems (top 30):')
    for sys_id, count in counts.most_common(30):
        print(f'  {sys_id}: {systems[sys_id]} ({count} occurrences)')

    # Find routes that only use the top N systems
    core_systems = {sys_id for sys_id, _ in counts.most_common(50)}

    # Find routes fully contained in core systems
    contained_routes = []
    for r in routes:
        if all(sid in core_systems for sid in r['path_ids']):
            contained_routes.append(r)

    print(f'\nRoutes fully contained in top 50 systems: {len(contained_routes)}')
    for r in contained_routes:
        print(f"  Route {r['route_id']}: {r['start_name']} -> {r['end_name']} ({r['path_len']} hops)")

    # Find a good subset of systems that covers ~50% of routes
    # Strategy: Start with Strym (most common destination) and expand outward

    # Get all systems reachable through common paths
    strym_routes = [r for r in routes if r['end_name'] == 'Strym' or r['start_name'] == 'Strym']
    print(f'\nRoutes involving Strym: {len(strym_routes)}')

    # Find the "corridor" systems that appear in many Strym routes
    strym_system_ids = []
    for r in strym_routes:
        strym_system_ids.extend(r['path_ids'])

    strym_counts = Counter(strym_system_ids)
    print('\nCommon systems in Strym routes (top 25):')
    for sys_id, count in strym_counts.most_common(25):
        print(f'  {sys_id}: {systems[sys_id]} ({count} occurrences)')

    # Extract the core "Strym corridor" - systems appearing in 5+ Strym routes
    corridor_systems = {sys_id for sys_id, count in strym_counts.items() if count >= 5}
    print(f'\nCorridor systems (appearing in 5+ Strym routes): {len(corridor_systems)}')

    # Find routes fully contained in corridor
    corridor_routes = []
    for r in routes:
        if all(sid in corridor_systems for sid in r['path_ids']):
            corridor_routes.append(r)

    print(f'\nRoutes fully contained in corridor: {len(corridor_routes)}')
    for r in corridor_routes:
        print(f"  Route {r['route_id']}: {r['start_name']} -> {r['end_name']} ({r['path_len']} hops)")

    # Output recommended systems for fixture
    print('\n' + '='*60)
    print('RECOMMENDED SYSTEMS FOR FIXTURE DATABASE:')
    print('='*60)
    for sys_id in sorted(corridor_systems):
        print(f'{sys_id},{systems[sys_id]}')

if __name__ == '__main__':
    main()
