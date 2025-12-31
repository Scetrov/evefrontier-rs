# Library API Contract: fmap Module

**Version**: 1.0.0 | **Module**: `evefrontier_lib::fmap`

## Public API

### Types

```rust
/// Waypoint type for fmap route encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WaypointType {
    Start = 0,
    Jump = 1,
    NpcGate = 2,
    SmartGate = 3,
    SetDestination = 4,
}

/// A waypoint in an fmap-encoded route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waypoint {
    pub system_id: u32,
    pub waypoint_type: WaypointType,
}

/// An encoded fmap route token with metadata.
#[derive(Debug, Clone)]
pub struct FmapToken {
    pub token: String,
    pub waypoint_count: usize,
    pub bit_width: u8,
    pub version: u8,
}
```

### Constants

```rust
/// Base ID for EVE Frontier solar systems (30,000,000).
pub const BASE_SYSTEM_ID: u32 = 30_000_000;

/// Current fmap format version.
pub const FMAP_VERSION: u8 = 1;
```

### Functions

#### encode_fmap_token

```rust
/// Encode waypoints to an fmap token string.
///
/// # Arguments
/// * `waypoints` - Slice of Waypoint structs to encode
///
/// # Returns
/// * `Ok(FmapToken)` - Token with metadata on success
/// * `Err(Error)` - Encoding error with details
///
/// # Errors
/// * `FmapInvalidSystemId` - system_id < BASE_SYSTEM_ID
/// * `FmapCompressionError` - gzip compression failed
///
/// # Example
/// ```rust
/// use evefrontier_lib::fmap::{encode_fmap_token, Waypoint, WaypointType};
///
/// let waypoints = vec![
///     Waypoint { system_id: 30000142, waypoint_type: WaypointType::Start },
///     Waypoint { system_id: 30000144, waypoint_type: WaypointType::Jump },
/// ];
///
/// let token = encode_fmap_token(&waypoints)?;
/// println!("Token: {}", token.token);
/// ```
pub fn encode_fmap_token(waypoints: &[Waypoint]) -> Result<FmapToken, Error>;
```

#### decode_fmap_token

```rust
/// Decode an fmap token string to waypoints.
///
/// # Arguments
/// * `token` - Base64url-encoded, gzipped fmap token
///
/// # Returns
/// * `Ok(Vec<Waypoint>)` - Decoded waypoints on success
/// * `Err(Error)` - Decoding error with details
///
/// # Errors
/// * `FmapBase64DecodeError` - Invalid base64url encoding
/// * `FmapDecompressionError` - Gzip decompression failed
/// * `FmapUnsupportedVersion` - Version != 1
/// * `FmapInvalidBitWidth` - Bit width out of range (1-30)
/// * `FmapTruncatedData` - Unexpected end of data
///
/// # Example
/// ```rust
/// use evefrontier_lib::fmap::decode_fmap_token;
///
/// let waypoints = decode_fmap_token("H4sIAAAA...")?;
/// for wp in waypoints {
///     println!("{}: type {:?}", wp.system_id, wp.waypoint_type);
/// }
/// ```
pub fn decode_fmap_token(token: &str) -> Result<Vec<Waypoint>, Error>;
```

#### route_to_fmap_url

```rust
/// Convert a Route to a full fmap URL.
///
/// # Arguments
/// * `route` - Route computed by pathfinding
/// * `base_url` - Optional base URL (default: "https://starmap.evefrontier.space/?route=")
///
/// # Returns
/// * `Ok(String)` - Complete URL with encoded route
/// * `Err(Error)` - Encoding error
///
/// # Example
/// ```rust
/// use evefrontier_lib::fmap::route_to_fmap_url;
///
/// let route = plan_route(&starmap, &options)?;
/// let url = route_to_fmap_url(&route, None)?;
/// println!("View route: {}", url);
/// ```
pub fn route_to_fmap_url(route: &Route, base_url: Option<&str>) -> Result<String, Error>;
```

#### route_to_waypoints

```rust
/// Convert a Route to fmap Waypoints.
///
/// Maps route steps to appropriate waypoint types:
/// - First step → Start
/// - Gate jumps → NpcGate
/// - Spatial jumps → Jump
///
/// # Arguments
/// * `route` - Route computed by pathfinding
///
/// # Returns
/// Vector of Waypoints ready for encoding
///
/// # Example
/// ```rust
/// use evefrontier_lib::fmap::route_to_waypoints;
///
/// let waypoints = route_to_waypoints(&route);
/// ```
pub fn route_to_waypoints(route: &Route) -> Vec<Waypoint>;
```

## Error Types

Added to `evefrontier_lib::error::Error`:

```rust
/// Invalid fmap token: base64 decoding failed
#[error("Invalid fmap token: base64 decode failed")]
FmapBase64DecodeError {
    #[source]
    source: base64::DecodeError,
},

/// Invalid fmap token: gzip decompression failed  
#[error("Invalid fmap token: decompression failed")]
FmapDecompressionError {
    #[source]
    source: std::io::Error,
},

/// Invalid fmap token: compression failed
#[error("fmap compression failed")]
FmapCompressionError {
    #[source]
    source: std::io::Error,
},

/// Unsupported fmap format version
#[error("Unsupported fmap version {version}, expected {expected}")]
FmapUnsupportedVersion {
    version: u8,
    expected: u8,
},

/// Invalid fmap header: bit width out of range
#[error("Invalid fmap bit width {k}, must be 1-30")]
FmapInvalidBitWidth {
    k: u8,
},

/// Invalid fmap token: data truncated
#[error("fmap data truncated: expected {expected} bytes, got {actual}")]
FmapTruncatedData {
    expected: usize,
    actual: usize,
},

/// Invalid system ID for fmap encoding
#[error("Invalid system ID {system_id}: must be >= {base_id}")]
FmapInvalidSystemId {
    system_id: u32,
    base_id: u32,
},
```

## Usage Patterns

### Encode Route for Sharing

```rust
use evefrontier_lib::{routing::plan_route, fmap::route_to_fmap_url};

let route = plan_route(&starmap, &options)?;
let url = route_to_fmap_url(&route, None)?;
println!("Share this route: {}", url);
```

### Decode Shared Token

```rust
use evefrontier_lib::fmap::decode_fmap_token;

let waypoints = decode_fmap_token(token)?;
println!("Route has {} waypoints:", waypoints.len());
for (i, wp) in waypoints.iter().enumerate() {
    let name = starmap.system_name(wp.system_id).unwrap_or("Unknown");
    println!("  {}: {} ({:?})", i + 1, name, wp.waypoint_type);
}
```

### Custom Base URL

```rust
let url = route_to_fmap_url(
    &route, 
    Some("http://localhost:3000/public/?route=")
)?;
```
