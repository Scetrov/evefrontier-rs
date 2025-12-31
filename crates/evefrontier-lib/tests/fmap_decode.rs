use base64::Engine;
use evefrontier_lib::fmap::{decode_fmap_token, encode_fmap_token, Waypoint, WaypointType};

// System IDs for test routes (relative to BASE_SYSTEM_ID = 30_000_000)
const JITA: u32 = 30_000_142;
const PERIMETER: u32 = 30_000_144;
const AMARR: u32 = 30_002_187;

#[test]
fn test_decode_single_waypoint() {
    // Encode a single waypoint
    let waypoints = vec![Waypoint {
        system_id: JITA,
        waypoint_type: WaypointType::Start,
    }];
    let token = encode_fmap_token(&waypoints).expect("encode failed");

    // Decode the token back
    let decoded = decode_fmap_token(&token.token).expect("decode failed");

    // Validate round-trip
    assert_eq!(decoded.waypoint_count, 1);
    assert_eq!(decoded.bit_width, token.bit_width);
    assert_eq!(decoded.version, token.version);
    assert_eq!(decoded.waypoints.len(), 1);

    let wp = &decoded.waypoints[0];
    assert_eq!(wp.system_id, JITA);
    assert_eq!(wp.waypoint_type, WaypointType::Start);
}

#[test]
fn test_decode_multiple_waypoints() {
    // Encode a three-waypoint route
    let waypoints = vec![
        Waypoint {
            system_id: JITA,
            waypoint_type: WaypointType::Start,
        },
        Waypoint {
            system_id: PERIMETER,
            waypoint_type: WaypointType::Jump,
        },
        Waypoint {
            system_id: AMARR,
            waypoint_type: WaypointType::SetDestination,
        },
    ];
    let token = encode_fmap_token(&waypoints).expect("encode failed");

    // Decode the token
    let decoded = decode_fmap_token(&token.token).expect("decode failed");

    // Validate round-trip
    assert_eq!(decoded.waypoint_count, 3);
    assert_eq!(decoded.waypoints.len(), 3);

    assert_eq!(decoded.waypoints[0].system_id, JITA);
    assert_eq!(decoded.waypoints[0].waypoint_type, WaypointType::Start);

    assert_eq!(decoded.waypoints[1].system_id, PERIMETER);
    assert_eq!(decoded.waypoints[1].waypoint_type, WaypointType::Jump);

    assert_eq!(decoded.waypoints[2].system_id, AMARR);
    assert_eq!(
        decoded.waypoints[2].waypoint_type,
        WaypointType::SetDestination
    );
}

#[test]
fn test_decode_invalid_base64() {
    // Invalid base64 should error
    let result = decode_fmap_token("!!!invalid base64!!!???");
    assert!(result.is_err());
}

#[test]
fn test_decode_invalid_version() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    // Manually construct a token with invalid version
    let raw_data = vec![
        99, // Invalid version (expected 1)
        8,  // bit_width = 8
        0, 0, // count = 0 (no waypoints)
    ];

    // Compress with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&raw_data).expect("write failed");
    let compressed = encoder.finish().expect("finish failed");

    // Encode as base64url
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed);

    // Try to decode - should fail with unsupported version error
    let result = decode_fmap_token(&token);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unsupported fmap version"));
}

#[test]
fn test_decode_truncated_data() {
    // Encode a token and truncate it
    let waypoints = vec![Waypoint {
        system_id: JITA,
        waypoint_type: WaypointType::Start,
    }];
    let token_str = encode_fmap_token(&waypoints).expect("encode failed").token;

    // Truncate the token (remove last few characters)
    if token_str.len() > 10 {
        let truncated = &token_str[..token_str.len() - 5];
        let result = decode_fmap_token(truncated);
        assert!(result.is_err());
    }
}

#[test]
fn test_round_trip_preserves_all_types() {
    // Test all 5 waypoint types to ensure 3-bit encoding works
    let waypoints = vec![
        Waypoint {
            system_id: JITA,
            waypoint_type: WaypointType::Start,
        },
        Waypoint {
            system_id: PERIMETER,
            waypoint_type: WaypointType::Jump,
        },
        Waypoint {
            system_id: AMARR,
            waypoint_type: WaypointType::NpcGate,
        },
        Waypoint {
            system_id: JITA + 1000,
            waypoint_type: WaypointType::SmartGate,
        },
        Waypoint {
            system_id: PERIMETER + 500,
            waypoint_type: WaypointType::SetDestination,
        },
    ];

    let token = encode_fmap_token(&waypoints).expect("encode failed");
    let decoded = decode_fmap_token(&token.token).expect("decode failed");

    for (original, decoded_wp) in waypoints.iter().zip(decoded.waypoints.iter()) {
        assert_eq!(original.system_id, decoded_wp.system_id);
        assert_eq!(original.waypoint_type, decoded_wp.waypoint_type);
    }
}
