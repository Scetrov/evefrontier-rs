# CLI Contracts: Scout Subcommand

This document defines the CLI interface contracts for the scout subcommand.

## Command Interface

### scout gates

```
evefrontier-cli scout gates <SYSTEM>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `SYSTEM` | String | Yes | System name (case-sensitive with fuzzy suggestions on mismatch) |

**Global flags applied**: `--format`, `--data-dir`, `--no-logo`

**Exit codes**: Standard Rust/CLI exit codes (non-zero on error).

### scout range

```
evefrontier-cli scout range <SYSTEM> [OPTIONS]
```

| Argument/Option | Type | Required | Default | Description |
|-----------------|------|----------|---------|-------------|
| `SYSTEM` | String | Yes | — | System name (case-sensitive with fuzzy suggestions on mismatch) |
| `--limit, -n` | usize | No | 10 | Maximum results (1-100) |
| `--radius, -r` | f64 | No | None | Maximum distance in light-years |
| `--max-temp, -t` | f64 | No | None | Maximum temperature in Kelvin |

**Global flags applied**: `--format`, `--data-dir`, `--no-logo`

**Exit codes**: Standard Rust/CLI exit codes (non-zero on error).

## Output Formats

### Basic (default)

Plain text, one neighbor per line:

```
Gates from Nod (3 connections):
  Brana
  D:2NAS
  G:3OA0
```

### Enhanced

Box-drawing format with metadata (see quickstart.md for examples).

### JSON

Structured JSON output (see data-model.md for schema).

## Behavior Contract

1. **Fuzzy matching**: If exact system name not found, suggest up to 5 similar names.
2. **Case-sensitivity**: System names are matched case-sensitively; fuzzy suggestions are offered on mismatch.
3. **Gate neighbors**: Returned in alphabetical order by name.
4. **Range neighbors**: Returned in ascending distance order.
5. **Spatial index**: Range query auto-builds index if missing (with warning).
6. **Empty results**: Return success (exit 0) with count=0 and empty list.
