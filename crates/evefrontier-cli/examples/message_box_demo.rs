//! Demonstration of the message box functionality.
//!
//! Run with: cargo run -p evefrontier-cli --example message_box_demo

use evefrontier_cli::terminal::ColorPalette;
use evefrontier_cli::{build_message_box, MessageBoxLevel};

fn main() {
    let palette = ColorPalette::colored();

    // Short info message
    println!("=== INFO Box (short message) ===");
    let info = build_message_box(
        MessageBoxLevel::Info,
        "All fuel and heat values are estimations",
        &palette,
        true,
        Some(80),
    );
    print!("{}", info);

    // Long warning message that will wrap
    println!("\n=== WARN Box (wrapping text) ===");
    let warn = build_message_box(
        MessageBoxLevel::Warn,
        "This route requires refueling at intermediate systems. Make sure you have sufficient fuel capacity and plan your stops accordingly to avoid being stranded.",
        &palette,
        true,
        Some(60),
    );
    print!("{}", warn);

    // Error message
    println!("\n=== ERROR Box ===");
    let error = build_message_box(
        MessageBoxLevel::Error,
        "Failed to compute route: No valid path found between systems",
        &palette,
        true,
        Some(80),
    );
    print!("{}", error);

    // ASCII mode (no Unicode support)
    println!("\n=== INFO Box (ASCII mode) ===");
    let ascii_info = build_message_box(
        MessageBoxLevel::Info,
        "This demonstrates ASCII box drawing for terminals without Unicode support",
        &palette,
        false, // ASCII mode
        Some(70),
    );
    print!("{}", ascii_info);

    // Auto-detect terminal width
    println!("\n=== WARN Box (auto-detect width) ===");
    let auto_width = build_message_box(
        MessageBoxLevel::Warn,
        "Terminal width will be auto-detected from the COLUMNS environment variable or default to 80 characters if not set",
        &palette,
        true,
        None, // Auto-detect
    );
    print!("{}", auto_width);
}
