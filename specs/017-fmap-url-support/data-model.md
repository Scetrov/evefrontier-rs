# Data Model: fmap URL Support

**Branch**: `017-fmap-url-support` | **Date**: 2025-12-31

## Entities

### WaypointType

Enum representing the type of route waypoint for fmap encoding.

```rust
/// Waypoint type for fmap route encoding.
/// 
/// Values correspond to the frontier-reapers/starmap specification:
/// https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WaypointType {
    /// Route starting point (Type 0)
    Start = 0,
    /// Spatial jump without gate (Type 1)
    Jump = 1,
    /// Jump via NPC-controlled gate (Type 2)
    NpcGate = 2,
    /// Jump via player-deployed smart gate (Type 3)
    SmartGate = 3,
    /// Final destination marker (Type 4)
    SetDestination = 4,
}
```

**Validation Rules**:
- Value must be in range 0-4
- Only 2 bits used in encoding (values 0-3), SetDestination (4) requires 3 bits (future format)

### Waypoint

A single waypoint in an fmap route.

```rust
/// A waypoint in an fmap-encoded route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waypoint {
    /// EVE Frontier solar system ID (e.g., 30000142 for Jita)
    pub system_id: u32,
    /// Type of waypoint (Start, Jump, NpcGate, etc.)
    pub waypoint_type: WaypointType,
}
```

**Validation Rules**:
- `system_id` must be >= 30,000,000 (BASE_ID)
- `system_id - BASE_ID` must fit in 30 bits (max offset ~1 billion)
- `waypoint_type` must encode to 2 bits (values 0-3) for current format version

### FmapToken

Represents an encoded fmap URL token with metadata.

```rust
/// An encoded fmap route token with metadata.
#[derive(Debug, Clone)]
pub struct FmapToken {
    /// The base64url-encoded, gzipped token string
    pub token: String,
    /// Number of waypoints encoded
    pub waypoint_count: usize,
    /// Bit width used for system ID offsets
    pub bit_width: u8,
    /// Format version (currently always 1)
    pub version: u8,
}
```

### FmapHeader

Internal structure for the binary header.

```rust
/// Binary header structure for fmap encoding.
/// 
/// Layout:
/// - Byte 0: Version (u8, must be 1)
/// - Byte 1: Bit width k (u8, 1-30)
/// - Bytes 2-3: Waypoint count (u16 big-endian)
#[derive(Debug, Clone, Copy)]
struct FmapHeader {
    version: u8,
    bit_width: u8,
    count: u16,
}
```

**Validation Rules**:
- `version` must be 1 (current format)
- `bit_width` must be in range 1-30
- `count` maximum is 65535 (u16::MAX)

## Constants

```rust
/// Base ID for EVE Frontier/EVE Online solar systems.
/// All system IDs are encoded as offsets from this value.
pub const BASE_SYSTEM_ID: u32 = 30_000_000;

/// Current fmap format version.
pub const FMAP_VERSION: u8 = 1;

/// Header size in bytes.
pub const FMAP_HEADER_SIZE: usize = 4;

/// Maximum bit width for system ID offsets.
pub const MAX_BIT_WIDTH: u8 = 30;

/// Bits used to encode waypoint type.
pub const WAYPOINT_TYPE_BITS: u8 = 2;
```

## State Transitions

### Encoding Flow

```
Vec<Waypoint> → validate → calculate_bit_width → build_header → pack_bits → gzip → base64url → String
```

1. **Input**: Vector of Waypoint structs
2. **Validation**: Verify all system_ids >= BASE_ID, waypoint_types in range
3. **Calculate k**: Find max offset, compute bit width
4. **Build Header**: Create 4-byte header [version, k, count_hi, count_lo]
5. **Pack Bits**: For each waypoint, write k bits (offset) + 2 bits (type)
6. **Compress**: Gzip with level 9 compression
7. **Encode**: Convert to base64url (no padding)
8. **Output**: Token string

### Decoding Flow

```
String → base64url_decode → gunzip → parse_header → unpack_bits → validate → Vec<Waypoint>
```

1. **Input**: Token string (base64url encoded)
2. **Decode Base64**: Convert from base64url to bytes
3. **Decompress**: Gunzip to raw bytes
4. **Parse Header**: Extract version, k, count
5. **Validate Header**: Check version=1, k in range, count makes sense
6. **Unpack Bits**: For each waypoint, read k bits (offset) + 2 bits (type)
7. **Reconstruct**: Add BASE_ID to offsets, map types
8. **Output**: Vector of Waypoints

## Relationships

### Integration with Existing Route Types

```rust
// Conversion from Route to fmap Waypoints
impl From<&Route> for Vec<Waypoint> {
    fn from(route: &Route) -> Self {
        route.steps.iter().enumerate().map(|(i, step)| {
            let waypoint_type = if i == 0 {
                WaypointType::Start
            } else if step.is_gate {
                WaypointType::NpcGate  // Conservative default
            } else {
                WaypointType::Jump
            };
            Waypoint {
                system_id: step.system_id,
                waypoint_type,
            }
        }).collect()
    }
}
```

### Error Variants

```rust
/// Errors specific to fmap encoding/decoding operations.
pub enum Error {
    // ... existing variants ...
    
    /// Invalid fmap token: base64 decoding failed
    FmapBase64DecodeError { source: base64::DecodeError },
    
    /// Invalid fmap token: gzip decompression failed
    FmapDecompressionError { source: std::io::Error },
    
    /// Invalid fmap token: compression failed
    FmapCompressionError { source: std::io::Error },
    
    /// Unsupported fmap format version
    FmapUnsupportedVersion { version: u8, expected: u8 },
    
    /// Invalid fmap header: bit width out of range
    FmapInvalidBitWidth { k: u8 },
    
    /// Invalid fmap token: data truncated unexpectedly
    FmapTruncatedData { expected: usize, actual: usize },
    
    /// Invalid system ID: below BASE_ID
    FmapInvalidSystemId { system_id: u32, base_id: u32 },
}
```

## Serialization Format

### Binary Wire Format

```
┌─────────────────────────────────────────────────────────────┐
│                        HEADER (4 bytes)                      │
├──────────┬───────────┬──────────────────────────────────────┤
│ version  │ bit_width │        waypoint_count (BE u16)       │
│  (1 byte)│  (1 byte) │           (2 bytes)                  │
├──────────┴───────────┴──────────────────────────────────────┤
│                       PAYLOAD (variable)                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Waypoint 0: [offset: k bits][type: 2 bits]          │    │
│  │ Waypoint 1: [offset: k bits][type: 2 bits]          │    │
│  │ ...                                                  │    │
│  │ Waypoint N: [offset: k bits][type: 2 bits]          │    │
│  │ Padding: 0-7 bits to byte boundary                   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘

Compression: gzip(HEADER + PAYLOAD, level=9)
Encoding: base64url(compressed, no padding)
```

### Example Encoding

For route: Jita (30000142) → Perimeter (30000144)

1. **Waypoints**:
   - `{system_id: 30000142, type: Start(0)}`
   - `{system_id: 30000144, type: Jump(1)}`

2. **Offsets**: 
   - 30000142 - 30000000 = 142
   - 30000144 - 30000000 = 144

3. **Bit Width**: max(142, 144) = 144 → k = 8 bits

4. **Header**: `[0x01, 0x08, 0x00, 0x02]`
   - version=1, k=8, count=2

5. **Payload** (k=8 bits per offset + 2 bits per type):
   - Waypoint 0: `10001110` (142) + `00` (Start) = `10001110 00`
   - Waypoint 1: `10010000` (144) + `01` (Jump) = `10010000 01`
   - Combined: `10001110 00100100 0001xxxx` (x = padding)

6. **Compress**: gzip(header + payload)

7. **Encode**: base64url(compressed)

## File Organization

```
crates/evefrontier-lib/src/
├── fmap.rs              # New module with encoding/decoding
├── fmap/
│   ├── mod.rs           # Module exports
│   ├── types.rs         # WaypointType, Waypoint, FmapToken
│   ├── encode.rs        # Encoding logic
│   ├── decode.rs        # Decoding logic
│   └── bits.rs          # BitWriter, BitReader helpers
└── lib.rs               # Add `pub mod fmap;`

crates/evefrontier-cli/src/
├── main.rs              # Add fmap subcommand
└── commands/
    └── fmap.rs          # fmap decode command handler
```
