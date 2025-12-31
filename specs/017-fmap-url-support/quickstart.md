# Quickstart: fmap URL Support

**Branch**: `017-fmap-url-support` | **Date**: 2025-12-31

## Overview

This feature adds support for generating fmap URLs compatible with the
[frontier-reapers/starmap](https://github.com/frontier-reapers/starmap) route visualization.

## Quick Usage

### CLI: Generate Shareable Route URL

```bash
# Plan a route and output as fmap URL
evefrontier-cli route Nod Brana --format fmap
# Output: https://starmap.evefrontier.space/?route=H4sIAAAA...

# Custom base URL for local development
evefrontier-cli route Nod Brana --format fmap \
  --fmap-base-url "http://localhost:3000/public/?route="
```

### CLI: Decode Shared Token

```bash
# Decode a token received from someone else
evefrontier-cli fmap decode "H4sIAAAAAAAAA2NgGAWjYBSMglEwCkYBAwAAAP__"

# Output:
# fmap Route Token (v1, k=8, 3 waypoints):
#   # | System ID  | System Name | Type
#  ---|------------|-------------|----------
#   1 | 30000142   | Nod         | Start
#   2 | 30000144   | Brana       | Jump
```

### Library: Encode Routes

```rust
use evefrontier_lib::fmap::{route_to_fmap_url, Waypoint, WaypointType};

// From a computed route
let url = route_to_fmap_url(&route, None)?;
println!("Share: {}", url);

// Manual waypoints
let waypoints = vec![
    Waypoint { system_id: 30000142, waypoint_type: WaypointType::Start },
    Waypoint { system_id: 30000144, waypoint_type: WaypointType::Jump },
];
let token = encode_fmap_token(&waypoints)?;
```

### Library: Decode Tokens

```rust
use evefrontier_lib::fmap::decode_fmap_token;

let waypoints = decode_fmap_token("H4sIAAAA...")?;
for wp in &waypoints {
    println!("System {}: {:?}", wp.system_id, wp.waypoint_type);
}
```

## Key Files

| File | Purpose |
|------|---------|
| `crates/evefrontier-lib/src/fmap.rs` | Core encoding/decoding logic |
| `crates/evefrontier-cli/src/commands/fmap.rs` | CLI fmap subcommand |
| `docs/fixtures/fmap_test_vectors.json` | Cross-implementation test vectors |

## Dependencies

```toml
# Already transitive; adding explicitly for clarity
flate2 = "1.0"
base64 = "0.21"
```

## Test Commands

```bash
# Run fmap-specific tests
cargo test -p evefrontier-lib fmap

# Cross-implementation validation
cargo test -p evefrontier-lib test_decode_js_reference_token
```

## See Also

- [spec.md](./spec.md) - Full feature specification
- [data-model.md](./data-model.md) - Data structures and encoding format
- [contracts/library-api.md](./contracts/library-api.md) - Library API reference
- [contracts/cli-api.md](./contracts/cli-api.md) - CLI usage reference
- [ROUTE_FEATURE.md](https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md) - External specification
