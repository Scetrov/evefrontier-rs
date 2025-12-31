use evefrontier_lib::fmap::{
    encode_fmap_token, Waypoint, WaypointType, BASE_SYSTEM_ID, FMAP_VERSION,
};

#[test]
fn test_encode_single_waypoint() {
    let waypoints = vec![Waypoint {
        system_id: 30000142, // Jita
        waypoint_type: WaypointType::Start,
    }];

    let token = encode_fmap_token(&waypoints).expect("encode failed");
    assert_eq!(token.version, FMAP_VERSION);
    assert_eq!(token.waypoint_count, 1);
    assert!(!token.token.is_empty());
}

#[test]
fn test_encode_multiple_waypoints() {
    let waypoints = vec![
        Waypoint {
            system_id: 30000142,
            waypoint_type: WaypointType::Start,
        },
        Waypoint {
            system_id: 30000144,
            waypoint_type: WaypointType::Jump,
        },
        Waypoint {
            system_id: 30002187,
            waypoint_type: WaypointType::NpcGate,
        },
    ];

    let token = encode_fmap_token(&waypoints).expect("encode failed");
    assert_eq!(token.version, FMAP_VERSION);
    assert_eq!(token.waypoint_count, 3);
    assert!(!token.token.is_empty());
}

#[test]
fn test_encode_invalid_system_id_below_base() {
    let waypoints = vec![Waypoint {
        system_id: 29999999, // Below BASE_SYSTEM_ID
        waypoint_type: WaypointType::Start,
    }];

    let result = encode_fmap_token(&waypoints);
    assert!(
        result.is_err(),
        "should reject system ID below BASE_SYSTEM_ID"
    );
}

#[test]
fn test_calculate_bit_width_single_waypoint() {
    // Single waypoint with ID 30000142: offset = 142
    // k = max(1, 32 - clz(142)) = max(1, 32 - 25) = 7
    let waypoints = vec![Waypoint {
        system_id: BASE_SYSTEM_ID + 142,
        waypoint_type: WaypointType::Start,
    }];

    let token = encode_fmap_token(&waypoints).expect("encode failed");
    assert!(token.bit_width >= 1 && token.bit_width <= 30);
}

#[test]
fn test_encode_edge_case_max_offset() {
    // System ID near the upper limit but still valid
    let max_system_id = BASE_SYSTEM_ID + ((1u32 << 30) - 1);
    let waypoints = vec![Waypoint {
        system_id: max_system_id,
        waypoint_type: WaypointType::Start,
    }];

    let token = encode_fmap_token(&waypoints).expect("encode failed");
    assert_eq!(token.bit_width, 30);
}
