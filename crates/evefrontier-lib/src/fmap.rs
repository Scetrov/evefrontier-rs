//! fmap URL encoding/decoding module.
//! Implements bitpacked route tokens compatible with frontier-reapers/starmap.
//!
//! See: https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Base ID for EVE Frontier solar systems.
/// All system IDs are encoded as offsets from this value.
pub const BASE_SYSTEM_ID: u32 = 30_000_000;

/// Current fmap format version.
pub const FMAP_VERSION: u8 = 1;

/// Header size in bytes (version + k + count).
pub const FMAP_HEADER_SIZE: usize = 4;

/// Maximum bit width for system ID offsets.
pub const MAX_BIT_WIDTH: u8 = 30;

/// Bits used to encode waypoint type (5 types: 0-4 requires 3 bits).
pub const WAYPOINT_TYPE_BITS: u8 = 3;

/// Waypoint type for fmap route encoding.
///
/// Values correspond to the frontier-reapers/starmap specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl WaypointType {
    /// Create from numeric value.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WaypointType::Start),
            1 => Some(WaypointType::Jump),
            2 => Some(WaypointType::NpcGate),
            3 => Some(WaypointType::SmartGate),
            4 => Some(WaypointType::SetDestination),
            _ => None,
        }
    }
}

/// A waypoint in an fmap-encoded route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waypoint {
    /// EVE Frontier solar system ID (e.g., 30000142 for Jita).
    pub system_id: u32,
    /// Type of waypoint (Start, Jump, NpcGate, etc.).
    pub waypoint_type: WaypointType,
}

/// An encoded fmap route token with metadata.
#[derive(Debug, Clone)]
pub struct FmapToken {
    /// The base64url-encoded, gzipped token string.
    pub token: String,
    /// Number of waypoints encoded.
    pub waypoint_count: usize,
    /// Bit width used for system ID offsets.
    pub bit_width: u8,
    /// Format version (currently always 1).
    pub version: u8,
}

/// Binary header structure for fmap encoding.
///
/// Layout:
/// - Byte 0: Version (u8, must be 1)
/// - Byte 1: Bit width k (u8, 1-30)
/// - Bytes 2-3: Waypoint count (u16 big-endian)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct FmapHeader {
    version: u8,
    bit_width: u8,
    count: u16,
}

/// Helper for writing bits at a time to a byte buffer.
struct BitWriter {
    bytes: Vec<u8>,
    cur: u8,
    bits: usize, // bits currently in cur (0..8)
}

impl BitWriter {
    fn new() -> Self {
        BitWriter {
            bytes: Vec::new(),
            cur: 0,
            bits: 0,
        }
    }

    fn write_bits(&mut self, value: u32, bit_count: u8) {
        for i in (0..bit_count).rev() {
            let bit = ((value >> i) & 1) as u8;
            self.cur = (self.cur << 1) | bit;
            self.bits += 1;
            if self.bits == 8 {
                self.bytes.push(self.cur);
                self.cur = 0;
                self.bits = 0;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.bits > 0 {
            self.cur <<= 8 - self.bits;
            self.bytes.push(self.cur);
        }
        self.bytes
    }
}

/// Helper for reading bits at a time from a byte buffer.
/// Used in Phase 4 (decoding).
#[allow(dead_code)]
struct BitReader<'a> {
    buf: &'a [u8],
    idx: usize,       // byte index
    cur: u8,          // current byte
    bits_left: usize, // bits remaining in cur (0..8)
}

#[allow(dead_code)]
impl<'a> BitReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        BitReader {
            buf,
            idx: 0,
            cur: if !buf.is_empty() { buf[0] } else { 0 },
            bits_left: if !buf.is_empty() { 8 } else { 0 },
        }
    }

    fn read_bits(&mut self, bit_count: u8) -> Result<u32, Error> {
        let mut v = 0u32;
        for _ in 0..bit_count {
            if self.bits_left == 0 {
                self.idx += 1;
                if self.idx >= self.buf.len() {
                    return Err(Error::FmapTruncatedData {
                        expected: self.idx + 1,
                        actual: self.buf.len(),
                    });
                }
                self.cur = self.buf[self.idx];
                self.bits_left = 8;
            }
            let msb = (self.cur & 0x80) != 0;
            v = (v << 1) | (msb as u32);
            self.cur <<= 1;
            self.bits_left -= 1;
        }
        Ok(v)
    }
}

/// Calculate the required bit width for encoding a maximum offset.
fn calculate_bit_width(max_offset: u32) -> u8 {
    if max_offset == 0 {
        1
    } else {
        (32 - max_offset.leading_zeros()) as u8
    }
}

/// Encode waypoints to raw bitpacked bytes with header and payload.
fn encode_raw_bitpacked(waypoints: &[Waypoint]) -> Result<Vec<u8>, Error> {
    // Validate all system IDs and find max offset
    let mut max_offset = 0u32;
    for wp in waypoints {
        if wp.system_id < BASE_SYSTEM_ID {
            return Err(Error::FmapInvalidSystemId {
                system_id: wp.system_id,
                base_id: BASE_SYSTEM_ID,
            });
        }
        let offset = wp.system_id - BASE_SYSTEM_ID;
        if offset > max_offset {
            max_offset = offset;
        }
    }

    let k = calculate_bit_width(max_offset);
    if k > MAX_BIT_WIDTH {
        return Err(Error::FmapInvalidBitWidth { k });
    }

    // Build header
    let count = waypoints.len() as u16;
    let mut header = vec![FMAP_VERSION, k, 0u8, 0u8];
    header[2..4].copy_from_slice(&count.to_be_bytes());

    // Pack waypoints into bits
    let mut writer = BitWriter::new();
    for wp in waypoints {
        let offset = wp.system_id - BASE_SYSTEM_ID;
        writer.write_bits(offset, k);
        writer.write_bits(wp.waypoint_type as u32, WAYPOINT_TYPE_BITS);
    }
    let payload = writer.finish();

    Ok([header, payload].concat())
}

/// Convert raw bytes to base64url (no padding).
fn to_base64url(bytes: &[u8]) -> String {
    use base64::Engine;
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    engine.encode(bytes)
}

/// Convert base64url string to raw bytes.
/// Used in Phase 4 (decoding).
#[allow(dead_code)]
fn from_base64url(s: &str) -> Result<Vec<u8>, Error> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    engine
        .decode(s)
        .map_err(|e| Error::FmapBase64DecodeError { source: e })
}

/// Decoded fmap token with extracted waypoints.
#[derive(Debug, Clone)]
pub struct DecodedFmapToken {
    pub version: u8,
    pub bit_width: u8,
    pub waypoint_count: usize,
    pub waypoints: Vec<Waypoint>,
}

/// Encode waypoints to an fmap token string.
///
/// # Arguments
/// * `waypoints` - Slice of Waypoint structs to encode
///
/// # Returns
/// * `Ok(FmapToken)` - Token with metadata on success
/// * `Err(Error)` - Encoding error with details
pub fn encode_fmap_token(waypoints: &[Waypoint]) -> Result<FmapToken, Error> {
    let raw = encode_raw_bitpacked(waypoints)?;

    // Compress with gzip
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    use std::io::Write;
    encoder
        .write_all(&raw)
        .map_err(|e| Error::FmapCompressionError { source: e })?;
    let compressed = encoder
        .finish()
        .map_err(|e| Error::FmapCompressionError { source: e })?;

    // Encode to base64url
    let token = to_base64url(&compressed);

    // Extract header info for metadata
    let version = raw[0];
    let bit_width = raw[1];

    Ok(FmapToken {
        token,
        waypoint_count: waypoints.len(),
        bit_width,
        version,
    })
}

/// Decode an fmap token string back to waypoints.
///
/// # Arguments
/// * `token_str` - Base64url-encoded gzipped token
///
/// # Returns
/// * `Ok(DecodedFmapToken)` - Decoded waypoints and metadata
/// * `Err(Error)` - Decoding error with details
pub fn decode_fmap_token(token_str: &str) -> Result<DecodedFmapToken, Error> {
    // Decode from base64url
    let compressed = from_base64url(token_str)?;

    // Decompress gzip
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut raw = Vec::new();
    decoder
        .read_to_end(&mut raw)
        .map_err(|e| Error::FmapDecompressionError { source: e })?;

    // Parse header (4 bytes minimum)
    if raw.len() < FMAP_HEADER_SIZE {
        return Err(Error::FmapTruncatedData {
            expected: FMAP_HEADER_SIZE,
            actual: raw.len(),
        });
    }

    let version = raw[0];
    let bit_width = raw[1];
    let count = u16::from_be_bytes([raw[2], raw[3]]) as usize;

    // Validate version
    if version != FMAP_VERSION {
        return Err(Error::FmapUnsupportedVersion {
            version,
            expected: FMAP_VERSION,
        });
    }

    // Validate bit width
    if bit_width == 0 || bit_width > MAX_BIT_WIDTH {
        return Err(Error::FmapInvalidBitWidth { k: bit_width });
    }

    // Parse payload
    let payload = &raw[FMAP_HEADER_SIZE..];
    let mut reader = BitReader::new(payload);
    let mut waypoints = Vec::with_capacity(count);

    for _ in 0..count {
        let offset = reader.read_bits(bit_width)?;
        let type_bits = reader.read_bits(WAYPOINT_TYPE_BITS)?;

        let system_id = offset
            .checked_add(BASE_SYSTEM_ID)
            .ok_or(Error::FmapInvalidSystemId {
                system_id: offset,
                base_id: BASE_SYSTEM_ID,
            })?;

        let waypoint_type = WaypointType::from_u8(type_bits as u8)
            .ok_or(Error::FmapInvalidBitWidth { k: type_bits as u8 })?;

        waypoints.push(Waypoint {
            system_id,
            waypoint_type,
        });
    }

    Ok(DecodedFmapToken {
        version,
        bit_width,
        waypoint_count: count,
        waypoints,
    })
}
