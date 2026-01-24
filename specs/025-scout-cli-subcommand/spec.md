# Feature Specification: Scout CLI Subcommand

**Spec ID**: 025  
**Created**: 2026-01-24  
**Status**: Draft

## Overview

Expose the existing scout functionality (gate neighbors and spatial range queries) as CLI subcommands, providing parity between the Lambda/HTTP APIs and the command-line interface.

## Problem Statement

The EVE Frontier CLI currently supports route planning, dataset management, ship listings, and spatial index operations. However, the scout functionality—which allows users to query gate-connected neighbors and systems within a spatial range—is only available via:

1. AWS Lambda functions (`evefrontier-lambda-scout-gates`, `evefrontier-lambda-scout-range`)
2. HTTP microservices (`evefrontier-service-scout-gates`, `evefrontier-service-scout-range`)

Users who want to explore the starmap without planning a full route cannot do so from the CLI. This creates a gap in functionality between local CLI usage and API-based usage.

## Requirements

### Functional Requirements

1. **FR-1**: Add `scout gates <system>` subcommand that lists all gate-connected neighbors of a given system
2. **FR-2**: Add `scout range <system>` subcommand that lists systems within a spatial radius
3. **FR-3**: Support `--limit N` option for range queries (default: 10, max: 100)
4. **FR-4**: Support `--radius R` option for range queries (light-years)
5. **FR-5**: Support `--max-temp T` option for range queries (temperature filter in Kelvin)
6. **FR-6**: Output formats must match global `--format` flag (basic, enhanced, json)
7. **FR-7**: Unknown system names should suggest fuzzy matches (consistent with route command)

### Non-Functional Requirements

1. **NFR-1**: Reuse existing library code from `evefrontier-lib` (no duplication of logic)
2. **NFR-2**: Follow existing CLI patterns (clap argument parsing, output formatting, error handling)
3. **NFR-3**: Include unit and integration tests (minimum 80% coverage for new code)
4. **NFR-4**: Update documentation (`docs/USAGE.md`, `README.md`)

## User Interface

### Command Syntax

```bash
# Gate neighbors
evefrontier-cli scout gates <SYSTEM>
evefrontier-cli scout gates "Nod"
evefrontier-cli scout gates "Brana" --format json

# Range neighbors
evefrontier-cli scout range <SYSTEM> [OPTIONS]
evefrontier-cli scout range "Nod" --limit 20
evefrontier-cli scout range "Nod" --radius 50.0
evefrontier-cli scout range "Nod" --max-temp 300
evefrontier-cli scout range "Nod" --limit 5 --radius 30.0 --max-temp 250 --format json
```

### Output Examples

**Basic format (gates)**:
```
Gate neighbors of Nod (3 found):
  Brana
  D:2NAS
  G:3OA0
```

**Enhanced format (gates)**:
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

**JSON format (gates)**:
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

**Enhanced format (range)**:
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

## Success Criteria

1. Both `scout gates` and `scout range` subcommands work with the existing dataset
2. Output is consistent with the route command's formatting style
3. All tests pass (`cargo test --workspace`)
4. Documentation is updated with examples
5. CLI help text is clear and follows existing patterns

## Out of Scope

- Real-time API integration (this is local dataset only)
- Ship-specific filtering (fuel range calculations belong in route command)
- Interactive mode or TUI
