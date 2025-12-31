# Feature Specification: fmap URL Support

**Branch**: `017-fmap-url-support` | **Date**: 2025-12-31 | **Status**: Draft

## Overview

Implement support for generating fmap URLs that are compatible with the
[frontier-reapers/starmap](https://github.com/frontier-reapers/starmap) route visualization feature.
This allows routes computed by the EVE Frontier CLI to be visualized in the web-based starmap
application.

## Background

The frontier-reapers starmap supports visualizing routes using a bitpacked data format passed via
the `route` query parameter. Routes are rendered as cyan lines on the map with a draggable table
showing waypoint details.

### External Specification Reference

- **Source**: https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md
- **Implementation Reference**: https://github.com/frontier-reapers/starmap/blob/main/src/bitpacking.js

## Requirements

### Functional Requirements

1. **FR-1**: Encode route waypoints to fmap URL format (bitpacked + gzip + base64url)
2. **FR-2**: Decode fmap URL tokens back to waypoint lists (for validation/debugging)
3. **FR-3**: Expose encoding/decoding as library functions in `evefrontier-lib`
4. **FR-4**: Add CLI output format option `--format fmap` to generate shareable URLs
5. **FR-5**: Add CLI command `fmap decode <token>` to decode and display route tokens
6. **FR-6**: Map waypoint types correctly: Start(0), Jump(1), NPC Gate(2), Smart Gate(3), Set Destination(4)

### Non-Functional Requirements

1. **NFR-1**: Encoding must be compatible with the JavaScript reference implementation
2. **NFR-2**: Round-trip encoding/decoding must be lossless
3. **NFR-3**: Generated URLs must be valid base64url (no padding, URL-safe characters)
4. **NFR-4**: Compression must use gzip with maximum compression level
5. **NFR-5**: Invalid tokens must produce clear error messages

## Data Format

### Wire Format (per frontier-reapers spec)

1. **Header** (4 bytes):
   - Version: 1 byte (must be `1`)
   - Bit width `k`: 1 byte (auto-calculated based on max offset from 30,000,000)
   - Waypoint count: 2 bytes (big-endian uint16)

2. **Payload** (variable):
   - For each waypoint:
     - System ID offset: `k` bits (unsigned, offset from BASE_ID 30,000,000)
     - Waypoint type: 2 bits (0-3)

3. **Compression**: Gzipped with maximum compression (level 9)
4. **Encoding**: Base64url (URL-safe, no padding)

### Waypoint Types

| Value | Name | Description |
|-------|------|-------------|
| 0 | Start | Route starting point |
| 1 | Jump | Spatial jump (not via gate) |
| 2 | NPC Gate | Jump via NPC-controlled gate |
| 3 | Smart Gate | Jump via player-deployed smart gate |
| 4 | Set Destination | Final destination marker |

## Integration Points

### Library Crate (`evefrontier-lib`)

- New module: `fmap.rs`
- Public functions:
  - `encode_fmap_token(waypoints: &[Waypoint]) -> Result<String, Error>`
  - `decode_fmap_token(token: &str) -> Result<Vec<Waypoint>, Error>`
  - `route_to_fmap_url(route: &Route, base_url: Option<&str>) -> Result<String, Error>`

### CLI Crate (`evefrontier-cli`)

- Extend `route` subcommand with `--format fmap` option
- Add `fmap decode <token>` subcommand for token inspection
- Output format: full URL with configurable base URL (default: `https://starmap.example.com/?route=`)

### Lambda Crates

- No changes required initially (can add fmap output format in future iteration)

## Acceptance Criteria

1. ✅ Library can encode routes matching the JavaScript reference implementation byte-for-byte
2. ✅ Library can decode tokens produced by the JavaScript reference implementation
3. ✅ CLI `--format fmap` produces valid, shareable URLs
4. ✅ CLI `fmap decode` displays route waypoints in human-readable format
5. ✅ Round-trip test passes: encode → decode → compare = identical
6. ✅ Cross-implementation test passes: decode JS-encoded token in Rust
7. ✅ Error handling covers: invalid tokens, unsupported versions, truncated data, invalid base64

## Out of Scope

- Web-based route visualization (separate ADR 0016 scope)
- Modification of routes after encoding
- Support for multiple route colors
- Route editing UI

## Security Considerations

- Input validation for decoded tokens (prevent buffer overflows)
- URL length limits for generated URLs (browser compatibility)
- No sensitive data in route tokens (only system IDs and types)

## Test Strategy

1. **Unit tests**: Bit packing/unpacking, base64url encoding/decoding
2. **Integration tests**: Full encode/decode cycle with known test vectors
3. **Cross-implementation tests**: Validate against JavaScript reference outputs
4. **Property-based tests**: Round-trip encoding for random valid routes

## References

- [ROUTE_FEATURE.md](https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md)
- [bitpacking.js](https://github.com/frontier-reapers/starmap/blob/main/src/bitpacking.js)
- [bitpacking.cs](https://github.com/frontier-reapers/starmap/blob/main/src/bitpacking.cs)
