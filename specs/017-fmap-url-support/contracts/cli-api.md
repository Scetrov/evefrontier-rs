# CLI API Contract: fmap Commands

**Version**: 1.0.0 | **Binary**: `evefrontier-cli`

## Extended Route Command

### `evefrontier-cli route --format fmap`

Generate fmap URL output instead of text route description.

```
USAGE:
    evefrontier-cli route [OPTIONS] <ORIGIN> <DESTINATION> --format fmap

OPTIONS:
    --format <FORMAT>          Output format [default: enhanced]
                               Values: basic, enhanced, json, ingame, fmap
    
    --fmap-base-url <URL>      Base URL for fmap output
                               [default: https://starmap.evefrontier.space/?route=]
                               [env: EVEFRONTIER_FMAP_BASE_URL]
    
    # ... existing route options ...
```

### Output Format

When `--format fmap` is specified:

```
https://starmap.evefrontier.space/?route=H4sIAAAAAAAAA2NgGAWjYBSMglEwCkYBAwAAAP__
```

With `--verbose`:

```
Route: Nod → Brana (2 jumps, 45.3 ly)
Token: H4sIAAAAAAAAA2NgGAWjYBSMglEwCkYBAwAAAP__
URL: https://starmap.evefrontier.space/?route=H4sIAAAAAAAAA2NgGAWjYBSMglEwCkYBAwAAAP__
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Route not found |
| 2 | Invalid origin/destination |
| 3 | Encoding error |

---

## New fmap Subcommand

### `evefrontier-cli fmap decode`

Decode and display an fmap token.

```
USAGE:
    evefrontier-cli fmap decode [OPTIONS] <TOKEN>

ARGS:
    <TOKEN>    The fmap token to decode (base64url string)

OPTIONS:
    -f, --format <FORMAT>    Output format [default: text]
                             Values: text, json
    
    --data-dir <PATH>        Path to dataset for system name lookup
                             [env: EVEFRONTIER_DATA_DIR]
    
    -h, --help               Print help information
```

### Text Output (default)

```
fmap Route Token (v1, k=8, 3 waypoints):

  # | System ID  | System Name | Type
 ---|------------|-------------|----------
  1 | 30000142   | Jita        | Start
  2 | 30000144   | Perimeter   | Jump
  3 | 30002187   | Amarr       | NpcGate
```

### JSON Output (`--format json`)

```json
{
  "version": 1,
  "bit_width": 8,
  "waypoints": [
    {
      "index": 1,
      "system_id": 30000142,
      "system_name": "Jita",
      "waypoint_type": "Start"
    },
    {
      "index": 2,
      "system_id": 30000144,
      "system_name": "Perimeter",
      "waypoint_type": "Jump"
    },
    {
      "index": 3,
      "system_id": 30002187,
      "system_name": "Amarr",
      "waypoint_type": "NpcGate"
    }
  ]
}
```

### Error Output

Invalid token:
```
Error: Invalid fmap token: base64 decode failed
  Token may be truncated or contain invalid characters.
  Ensure the entire token was copied correctly.
```

Unknown systems (without dataset):
```
fmap Route Token (v1, k=8, 3 waypoints):

  # | System ID  | System Name | Type
 ---|------------|-------------|----------
  1 | 30000142   | (unknown)   | Start
  2 | 30000144   | (unknown)   | Jump
  3 | 30002187   | (unknown)   | NpcGate

Note: System names not resolved. Use --data-dir to load dataset.
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid token format |
| 2 | Unsupported version |
| 3 | Data truncation |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `EVEFRONTIER_FMAP_BASE_URL` | Base URL for fmap output | `https://starmap.evefrontier.space/?route=` |
| `EVEFRONTIER_DATA_DIR` | Dataset path for name lookup | (OS cache dir) |

---

## Examples

### Generate Shareable URL

```bash
# Basic usage
evefrontier-cli route Nod Brana --format fmap

# With custom base URL
evefrontier-cli route Nod Brana --format fmap \
  --fmap-base-url "http://localhost:3000/public/?route="

# Via environment variable
export EVEFRONTIER_FMAP_BASE_URL="https://my-starmap.com/?route="
evefrontier-cli route Nod Brana --format fmap
```

### Decode Shared Token

```bash
# Decode with system names
evefrontier-cli fmap decode "H4sIAAAA..."

# Decode as JSON
evefrontier-cli fmap decode --format json "H4sIAAAA..."

# Decode without dataset (IDs only)
evefrontier-cli fmap decode "H4sIAAAA..." --data-dir /dev/null
```

### Pipeline Usage

```bash
# Generate route and copy URL to clipboard (Linux)
evefrontier-cli route Nod Brana --format fmap | xclip -selection clipboard

# Decode and filter
evefrontier-cli fmap decode --format json "$TOKEN" | jq '.waypoints[].system_name'
```
