//! Output formatting for route rendering.
//!
//! This module provides formatters for rendering route summaries
//! in various output formats (text, rich, enhanced, etc.).

use crate::terminal::{supports_color, ColorPalette};
use evefrontier_lib::RouteSummary;

mod enhanced;
pub use enhanced::EnhancedRenderer;
mod text;
pub use text::{render_basic, render_emoji, render_json, render_note, render_rich, render_text};

/// Render using the enhanced renderer (keeps compatibility with previous API)
pub fn render_enhanced(summary: &RouteSummary, base_url: &str) {
    let palette = if supports_color() {
        ColorPalette::colored()
    } else {
        ColorPalette::plain()
    };

    let renderer = EnhancedRenderer::new(palette);
    renderer.render(summary, base_url);
}

/// Default base URL for fmap route viewer (route token appended directly).
pub const DEFAULT_FMAP_BASE_URL: &str = "https://fmap.scetrov.live/?route=";
/// Type width parameter indicating 3-bit waypoint type encoding.
pub const FMAP_TYPE_WIDTH_PARAM: &str = "&tw=3";

/// Print the CLI logo banner.
///
/// The logo adapts to terminal capabilities:
/// - Uses Unicode box-drawing characters when supported
/// - Falls back to ASCII when Unicode is not detected
/// - Respects `NO_COLOR` and `TERM=dumb` conventions
pub fn print_logo() {
    use crate::terminal::{colors, supports_unicode};

    let (orange, cyan, reset) = if supports_color() {
        (colors::ORANGE, colors::CYAN, colors::RESET)
    } else {
        ("", "", "")
    };

    if supports_unicode() {
        // Sci-fi glitch/neon style banner with cyan border and orange text
        println!(
            "{cyan}╭────────────────────────────────────────────────╮{reset}
{cyan}│{orange} ░█▀▀░█░█░█▀▀░░░█▀▀░█▀▄░█▀█░█▀█░▀█▀░▀█▀░█▀▀░█▀▄ {cyan}│{reset}
{cyan}│{orange} ░█▀▀░▀▄▀░█▀▀░░░█▀▀░█▀▄░█░█░█░█░░█░░░█░░█▀▀░█▀▄ {cyan}│{reset}
{cyan}│{orange} ░▀▀▀░░▀░░▀▀▀░░░▀░░░▀░▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀▀▀░▀░▀ {cyan}│{reset}
{cyan}├────────────────────────────────────────────────┤{reset}
{cyan}│{orange}                    [ C L I ]                   {cyan}│{reset}
{cyan}╰────────────────────────────────────────────────╯{reset}",
            cyan = cyan,
            orange = orange,
            reset = reset
        );
    } else {
        // Fallback ASCII banner
        println!(
            "{color}+--------------------------------------------------+
|  EVE FRONTIER                                    |
|  >> PATHFINDER COMMAND LINE INTERFACE            |
+--------------------------------------------------+{reset}",
            color = orange,
            reset = reset
        );
    }
}

// Emoji and note renderers moved to `output::text` submodule

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_helpers::compute_details_column_widths;
    use crate::terminal::colors;
    // FuelProjection no longer required directly here; builders are used instead.

    #[test]
    fn test_get_temp_circle_hot() {
        let palette = ColorPalette::colored();
        let renderer = EnhancedRenderer::new(palette);
        let circle = renderer.get_temp_circle(60.0);
        assert!(circle.contains('●'));
        // Should contain red color code
        assert!(circle.contains("\x1b[31m"));
    }

    #[test]
    fn test_build_step_details_black_hole() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());
        let step = crate::test_helpers::RouteStepBuilder::new()
            .id(30000002)
            .name("M 974")
            .distance(3.0)
            .build();

        let widths = compute_details_column_widths(std::slice::from_ref(&step));
        let line = renderer
            .build_step_details_line(&step, &widths)
            .expect("line present");
        assert!(line.contains("Black Hole"));
        assert!(!line.contains("min"));
    }

    #[test]
    fn test_build_step_details_includes_fuel_and_counts() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());
        let step = crate::test_helpers::RouteStepBuilder::new()
            .index(2)
            .id(42)
            .name("Test")
            .distance(10.0)
            .min_temp(12.34)
            .planets(2)
            .moons(1)
            .fuel(5.0, 5.0, Some(95.0))
            .build();

        let widths = compute_details_column_widths(std::slice::from_ref(&step));
        let line = renderer
            .build_step_details_line(&step, &widths)
            .expect("line present");
        assert!(line.contains("min"));
        // Planets and moons moved to the header line
        assert!(!line.contains("Planets"));
        assert!(!line.contains("Moon"));
        assert!(line.contains("fuel"));

        // Header should include the planets/moons counts and they should be separated
        let header = renderer.build_step_header_line(&step, false, false);
        assert!(header.contains("2 Planets"));
        assert!(header.contains("1 Moon"));
        // Ensure there is at least one space between the tokens (no accidental run-together)
        assert!(header.contains("2 Planets 1 Moon"));
    }

    #[test]
    fn test_build_step_details_colors_fuel_when_colored() {
        let renderer = EnhancedRenderer::new(ColorPalette::colored());
        let step = crate::test_helpers::RouteStepBuilder::new()
            .index(2)
            .id(42)
            .name("Test")
            .distance(10.0)
            .fuel(3.5, 3.5, Some(96.5))
            .build();

        let widths = compute_details_column_widths(std::slice::from_ref(&step));
        let line = renderer
            .build_step_details_line(&step, &widths)
            .expect("line present");
        assert!(line.contains(colors::ORANGE));
        assert!(line.contains(colors::MAGENTA));
        assert!(line.contains("fuel 4"));
        assert!(line.contains("(rem 97)"));
    }

    #[test]
    fn test_small_heat_shows_less_than_marker() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());
        let step = evefrontier_lib::RouteStep {
            index: 2,
            id: 42,
            name: Some("Test".to_string()),
            distance: Some(10.0),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: Some(evefrontier_lib::ship::HeatProjection {
                hop_heat: 0.0001,
                warning: None,
                wait_time_seconds: None,
                residual_heat: Some(0.0001),
                can_proceed: true,
            }),
        };

        let widths = compute_details_column_widths(std::slice::from_ref(&step));
        let line = renderer
            .build_step_details_line(&step, &widths)
            .expect("line present");

        assert!(line.contains("heat +<0.01"));
    }

    #[test]
    fn test_padding_consistent_for_singular_plural() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());

        let singular = crate::test_helpers::RouteStepBuilder::new()
            .name("One")
            .distance(1.0)
            .planets(1)
            .moons(1)
            .build();

        let plural = crate::test_helpers::RouteStepBuilder::new()
            .name("One")
            .distance(1.0)
            .planets(2)
            .moons(2)
            .build();

        let singular_widths = compute_details_column_widths(std::slice::from_ref(&singular));
        let plural_widths = compute_details_column_widths(std::slice::from_ref(&plural));
        let singular_line = renderer.build_step_details_line(&singular, &singular_widths);
        let plural_line = renderer.build_step_details_line(&plural, &plural_widths);

        // (No debug prints in normal test run)

        // Planets/moons moved into header; details should be absent when there is
        // no other per-step metadata (min/temp/heat/fuel)
        assert!(singular_line.is_none());
        assert!(plural_line.is_none());

        // Header should include singular and plural labels with padding
        let singular_header = renderer.build_step_header_line(&singular, true, false);
        let plural_header = renderer.build_step_header_line(&plural, true, false);
        assert!(singular_header.contains(" 1 Planet ")); // padded
        assert!(plural_header.contains(" 2 Planets"));

        assert!(singular_header.contains(" 1 Moon ")); // padded
        assert!(plural_header.contains(" 2 Moons"));

        // And ensure combined tokens have a separating space
        assert!(singular_header.contains("1 Planet 1 Moon"));
    }

    #[test]
    fn test_column_widths_computation() {
        let _renderer = EnhancedRenderer::new(ColorPalette::plain());

        let steps = vec![
            crate::test_helpers::RouteStepBuilder::new()
                .index(1)
                .id(1)
                .name("Step 1")
                .distance(1.0)
                .planets(1)
                .moons(1)
                .fuel(3.0, 3.0, Some(90.0))
                .build(),
            crate::test_helpers::RouteStepBuilder::new()
                .index(2)
                .id(2)
                .name("Step 2")
                .distance(2.0)
                .method("gate")
                .min_temp(10.0)
                .planets(2)
                .moons(2)
                .fuel(4.5, 7.5, Some(85.5))
                .build(),
            crate::test_helpers::RouteStepBuilder::new()
                .index(3)
                .id(3)
                .name("Step 3")
                .distance(3.0)
                .min_temp(20.0)
                .planets(3)
                .moons(3)
                .fuel(5.0, 12.5, Some(80.0))
                .build(),
        ];

        let widths = compute_details_column_widths(&steps);
        assert_eq!(widths.fuel_val_width, 1); // "3", "5", "5" -> max len 1
        assert_eq!(widths.rem_val_width, 2); // "90", "86", "80" -> max len 2
        assert_eq!(widths.heat_val_width, 0); // No heat
    }

    #[test]
    fn test_render_step_details_alignment() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());

        let steps = vec![
            crate::test_helpers::RouteStepBuilder::new()
                .index(1)
                .id(1)
                .name("A")
                .distance(1.0)
                .planets(1)
                .moons(1)
                .fuel(3.0, 3.0, Some(90.0))
                .build(),
            crate::test_helpers::RouteStepBuilder::new()
                .index(2)
                .id(2)
                .name("B")
                .distance(2.0)
                .method("gate")
                .min_temp(10.0)
                .planets(2)
                .moons(2)
                .fuel(4.5, 7.5, Some(85.5))
                .build(),
        ];

        let widths = compute_details_column_widths(&steps);

        // Step 1
        let line1 = renderer
            .build_step_details_line(&steps[0], &widths)
            .unwrap();
        // "fuel 3" (width 1) -> "fuel 3"
        // "(rem 90)" (width 2) -> "(rem 90)"
        assert!(line1.contains("fuel 3"));
        assert!(line1.contains("(rem 90)"));

        // Step 2
        let line2 = renderer
            .build_step_details_line(&steps[1], &widths)
            .unwrap();
        // "fuel 5" (width 1) -> "fuel 5"
        // "(rem 86)" (width 2) -> "(rem 86)"
        assert!(line2.contains("fuel 5"));
        assert!(line2.contains("(rem 86)"));
    }
}
