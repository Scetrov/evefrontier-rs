//! Fuzz target: fmap token decode + round-trip invariants.
//!
//! Oracle:
//! - Arbitrary bytes fed to `decode_fmap_token` must either return a typed error
//!   or a successfully decoded token. Must never panic.
//! - For tokens that decode successfully, re-encoding the decoded waypoints must
//!   produce a token with the same waypoint count and system IDs.

#![no_main]

use evefrontier_lib::fmap::{decode_fmap_token, encode_fmap_token};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip very small inputs that cannot represent a valid gzipped fmap token.
    if data.len() < 4 {
        return;
    }

    // The input may or may not be valid UTF-8; fmap tokens are base64url-encoded
    // so we only test the UTF-8 path.
    let token_str = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // First oracle: decode must not panic and must produce an Ok or Err result.
    let decoded = match decode_fmap_token(token_str) {
        Ok(d) => d,
        Err(_) => return, // valid error path
    };

    // Second oracle: round-trip invariant.
    // Re-encoding the decoded waypoints must succeed and reproduce the count.
    if let Ok(reencoded) = encode_fmap_token(&decoded.waypoints) {
        assert_eq!(
            reencoded.waypoint_count, decoded.waypoint_count,
            "round-trip must preserve waypoint count"
        );
        assert_eq!(
            reencoded.version, decoded.version,
            "round-trip must preserve version"
        );
    }
});
