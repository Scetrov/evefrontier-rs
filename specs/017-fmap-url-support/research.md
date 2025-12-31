# Research: fmap URL Support

**Branch**: `017-fmap-url-support` | **Date**: 2025-12-31

## Research Questions

### RQ-1: Bit Packing Implementation in Rust

**Question**: What is the best approach for bit-level packing/unpacking in Rust?

**Research Findings**:
- The `bitvec` crate provides a comprehensive bit manipulation API but may be overkill for this use case
- Manual bit manipulation using shifts and masks is straightforward and matches the reference implementation
- The reference implementation uses a simple `BitWriter`/`BitReader` pattern with byte buffers

**Decision**: Implement custom `BitWriter`/`BitReader` structs mirroring the JavaScript reference
- Direct port ensures byte-for-byte compatibility
- No external dependencies needed
- Simpler to audit and test

**Alternatives Considered**:
- `bitvec` crate: More feature-rich but adds dependency and may have different byte ordering
- `bitstream-io` crate: Good for streaming but adds complexity

### RQ-2: Gzip Compression in Rust

**Question**: What library should we use for gzip compression/decompression?

**Research Findings**:
- `flate2` is the de facto standard for gzip in Rust (used by 90%+ of projects)
- Supports compression levels 0-9 (reference uses level 9/best compression)
- Well-maintained, stable API, no-std support available

**Decision**: Use `flate2` crate with `GzEncoder`/`GzDecoder`
- Mature, well-tested library
- Matches compression output of Node.js `zlib.gzipSync`
- Already used by many dependencies in the workspace

**Alternatives Considered**:
- `libflate`: Pure Rust but less optimized
- `miniz_oxide`: Lower-level, more work to use

### RQ-3: Base64url Encoding

**Question**: What library should we use for base64url encoding/decoding?

**Research Findings**:
- `base64` crate provides configurable alphabets including URL_SAFE_NO_PAD
- Reference implementation removes padding and replaces `+` with `-`, `/` with `_`
- The `base64` crate's `URL_SAFE_NO_PAD` config matches this exactly

**Decision**: Use `base64` crate with `URL_SAFE_NO_PAD` configuration
- Already a transitive dependency in the workspace
- Directly matches reference implementation behavior

**Alternatives Considered**:
- Manual implementation: More work, error-prone
- `data-encoding`: Good but `base64` is more widely used

### RQ-4: System ID Base Offset

**Question**: What is the BASE_ID and why is it used?

**Research Findings**:
- EVE Online system IDs start at 30,000,000 (Jita is 30000142)
- EVE Frontier uses the same ID scheme from the e6c3 dataset
- Subtracting BASE_ID (30,000,000) reduces the bit width needed to encode system IDs
- Maximum offset determines `k` (bits per system ID): `k = max(1, 32 - clz32(maxOffset))`

**Decision**: Use BASE_ID = 30,000,000 per reference implementation
- Required for compatibility
- Verified against e6c3 dataset system IDs

### RQ-5: Waypoint Type Mapping

**Question**: How should we map EVE Frontier route steps to waypoint types?

**Research Findings**:
- Reference defines types: Start(0), Jump(1), NPC Gate(2), Smart Gate(3), Set Destination(4)
- EVE Frontier routes distinguish between spatial jumps and gate jumps
- Current `RouteStep` structure has `is_gate: bool` and `distance_ly: f64`

**Decision**: Map as follows:
- First waypoint → Start (0)
- Gate jump (is_gate=true) → NPC Gate (2) (conservative default; no way to distinguish smart gates currently)
- Spatial jump (is_gate=false) → Jump (1)
- Last waypoint → unchanged (not Set Destination, as that's for user marking)

**Alternatives Considered**:
- Use Set Destination (4) for final waypoint: Not accurate per spec semantics
- Detect smart gates: Would require additional data not in current routes

### RQ-6: Error Handling Strategy

**Question**: What error types should be defined for fmap operations?

**Research Findings**:
- Reference implementation throws errors for: too short buffer, unsupported version, invalid k, EOF
- Rust should use typed errors via `thiserror` consistent with rest of library

**Decision**: Add `FmapError` variants to existing `Error` enum:
- `FmapInvalidToken { reason: String }`: General decoding errors
- `FmapUnsupportedVersion { version: u8 }`: Future-proofing for format changes
- `FmapTruncatedData`: Unexpected end of data
- `FmapCompressionError`: Gzip encode/decode failures

### RQ-7: CLI Integration Approach

**Question**: How should fmap output integrate with existing CLI?

**Research Findings**:
- Current `--format` options: basic, enhanced, json, ingame
- Adding `fmap` as a format option is consistent with existing pattern
- Base URL should be configurable (env var or flag)

**Decision**: 
- Add `--format fmap` to `route` subcommand
- Add `--fmap-base-url` flag (default: `https://starmap.evefrontier.space/?route=`)
- Add `fmap decode <token>` subcommand for inspection

### RQ-8: Cross-Implementation Compatibility Testing

**Question**: How can we ensure byte-for-byte compatibility with the JavaScript reference?

**Research Findings**:
- Need test vectors generated by the JavaScript implementation
- Should test both encoding and decoding directions
- Edge cases: single waypoint, maximum k value, many waypoints

**Decision**: Create test fixtures with pre-computed tokens:
1. Generate test cases using `bitpacking.js`
2. Store expected tokens in test fixtures
3. Test Rust decode → compare with expected waypoints
4. Test Rust encode → compare with expected token string
5. Test round-trip: JS encode → Rust decode → Rust encode → compare

## Dependencies Summary

| Dependency | Version | Purpose |
|------------|---------|---------|
| `flate2` | ~1.0 | Gzip compression/decompression |
| `base64` | ~0.21 | Base64url encoding/decoding |

Both are already transitive dependencies; adding them directly doesn't increase dependency count.

## Implementation Notes

### Bit Width Calculation

```rust
fn calculate_bit_width(max_offset: u32) -> u8 {
    // k = max(1, 32 - leading_zeros(max_offset))
    if max_offset == 0 {
        1
    } else {
        (32 - max_offset.leading_zeros()) as u8
    }
}
```

### Header Format

```
Byte 0: Version (always 1)
Byte 1: k (bit width for system ID offsets)
Bytes 2-3: Count (big-endian u16)
```

### Payload Format

For each waypoint:
- `k` bits: (system_id - 30_000_000) as unsigned
- 2 bits: waypoint type (0-3)

Padding: Unused bits in final byte are zero-padded.

## Open Questions (Resolved)

None remaining - all research questions addressed.

## References

- [flate2 crate documentation](https://docs.rs/flate2)
- [base64 crate documentation](https://docs.rs/base64)
- [JavaScript reference implementation](https://github.com/frontier-reapers/starmap/blob/main/src/bitpacking.js)
