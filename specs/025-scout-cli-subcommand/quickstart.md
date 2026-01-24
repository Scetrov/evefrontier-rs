# Quickstart: Scout CLI Subcommand

**Date**: 2026-01-24

## Prerequisites

1. EVE Frontier CLI installed (`evefrontier-cli`)
2. Dataset downloaded (`evefrontier-cli download`)
3. For range queries: spatial index built (`evefrontier-cli index-build`)

## Basic Usage

### Scout Gate Neighbors

Find all systems connected by jump gates to a given system:

```bash
# Basic usage
evefrontier-cli scout gates "Nod"

# With JSON output
evefrontier-cli scout gates "Brana" --format json

# With enhanced (box-drawing) output
evefrontier-cli scout gates "Nod" --format enhanced
```

### Scout Systems in Range

Find systems within a spatial radius of a given system:

```bash
# Default: 10 nearest systems
evefrontier-cli scout range "Nod"

# Limit results
evefrontier-cli scout range "Nod" --limit 5

# Filter by radius (light-years)
evefrontier-cli scout range "Nod" --radius 50.0

# Filter by temperature (Kelvin)
evefrontier-cli scout range "Nod" --max-temp 300

# Combined filters
evefrontier-cli scout range "Nod" --limit 20 --radius 100.0 --max-temp 350

# JSON output
evefrontier-cli scout range "Nod" --limit 5 --format json
```

## Example Output

### Gate Neighbors (Enhanced)

```
╔═══════════════════════════════════════════════════════════════════════╗
║  Scout: Gate Neighbors                                                ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Origin: Nod (ID: 30000001)                                           ║
║  Gate Connections: 3                                                  ║
╠═══════════════════════════════════════════════════════════════════════╣
║  [GATE]  Brana                                                        ║
║  [GATE]  D:2NAS                                                       ║
║  [GATE]  G:3OA0                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### Range Query (Enhanced)

```
╔═══════════════════════════════════════════════════════════════════════╗
║  Scout: Systems in Range                                              ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Origin: Nod (ID: 30000001)                                           ║
║  Radius: 50.0 ly | Max Temp: 300 K | Limit: 10                        ║
║  Systems Found: 5                                                     ║
╠═══════════════════════════════════════════════════════════════════════╣
║   1. Brana             12.4 ly    285 K                               ║
║   2. D:2NAS            23.1 ly    290 K                               ║
║   3. G:3OA0            34.7 ly    275 K                               ║
║   4. H:2L2S            41.2 ly    298 K                               ║
║   5. J:35IA            48.9 ly    280 K                               ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### Gate Neighbors (JSON)

```json
{
  "system": "Nod",
  "system_id": 30000001,
  "count": 3,
  "neighbors": [
    { "name": "Brana", "id": 30000002 },
    { "name": "D:2NAS", "id": 30000003 },
    { "name": "G:3OA0", "id": 30000004 }
  ]
}
```

## Error Handling

### Unknown System

```bash
$ evefrontier-cli scout gates "Unknown System"
error: System 'Unknown System' not found

Did you mean one of these?
  • Nod
  • Brana
  • D:2NAS
```

### No Spatial Index (Range Query)

```bash
$ evefrontier-cli scout range "Nod"
warning: Spatial index not found. Building on-demand...
[... results ...]
```

## Common Use Cases

1. **Exploration**: Find nearby systems to scout before planning a route
2. **Gate mapping**: Identify all systems reachable via a single gate jump
3. **Temperature-safe systems**: Find cool systems for route planning with `--max-temp`
4. **Jump range analysis**: Find all systems within ship's maximum jump range with `--radius`

## See Also

- `evefrontier-cli route` — Plan routes between systems
- `evefrontier-cli index-build` — Build spatial index for faster range queries
- `docs/USAGE.md` — Full CLI documentation
