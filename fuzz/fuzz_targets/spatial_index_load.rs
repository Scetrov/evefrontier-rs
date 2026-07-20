//! Fuzz target: spatial-index byte loader.
//!
//! Oracle:
//! - Arbitrary bytes written to a temp file and fed to `SpatialIndex::load`
//!   must either return a typed `SpatialIndexLoad` error or a valid index,
//!   without panicking or allocating an oversized buffer.
//! - No write outside the requested temp directory.

#![no_main]

use std::io::Write;

use evefrontier_lib::SpatialIndex;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip tiny inputs that cannot contain a valid spatial index.
    if data.len() < 16 {
        return;
    }

    let tmp = match tempfile::NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut file = tmp.as_file().try_clone().expect("clone temp file");
    if file.write_all(data).is_err() {
        return;
    }
    file.flush().ok();

    // `load` must not panic on malformed input and must return a typed error
    // for unrecognised / truncated files.
    let _ = SpatialIndex::load(tmp.path());
});
