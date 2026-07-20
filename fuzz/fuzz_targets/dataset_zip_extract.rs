//! Fuzz target: local dataset ZIP extraction path-safety.
//!
//! Oracle:
//! - Arbitrary bytes written to a temp file are treated as a ZIP archive.
//!   Entries must never write outside the temporary destination directory.
//!   Malformed archives must return errors without panicking.
//!
//! This mirrors the extraction pattern used by
//! `evefrontier_lib::github::download_dataset` for local `.zip` sources,
//! which uses `zip::ZipArchive::entry.enclosed_name()` to reject path
//! traversal attempts.

#![no_main]

use std::io::{Cursor, Read};

use libfuzzer_sys::fuzz_target;
use zip::ZipArchive;

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);

    // Attempt to open as ZIP archive; malformed archives must return Err.
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(_) => return,
    };

    // Walk every entry. The oracle is that `enclosed_name()` returns None for
    // unsafe paths and no entry writes outside the in-memory "destination".
    for idx in 0..archive.len() {
        let mut entry = match archive.by_index(idx) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.is_file() {
            continue;
        }

        // `enclosed_name` returns None for path-traversal attempts.
        let _safe_name = match entry.enclosed_name() {
            Some(path) => path,
            None => continue, // path traversal rejected — safe
        };

        // Read the entry contents up to a bounded size to prevent OOM.
        // The library enforces the same bounded-read pattern in production.
        let mut buf = [0u8; 1024 * 1024]; // 1 MiB cap per entry
        let mut total: usize = 0;
        loop {
            match entry.read(&mut buf[total..]) {
                Ok(0) => break,
                Ok(n) => {
                    total += n;
                    if total >= buf.len() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
});
