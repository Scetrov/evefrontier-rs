//! Output formatting for route rendering.
//!
//! This module provides formatters for rendering route summaries
//! in various output formats (text, rich, enhanced, etc.).

use std::io::{self, Write};

use evefrontier_lib::{RouteRenderMode, RouteStep, RouteSummary};

use crate::terminal::{format_with_separators, supports_color, ColorPalette};

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

/// Print the footer with elapsed time.
///
/// # Arguments
///
/// * `elapsed` - The duration since the command started
pub fn print_footer(elapsed: std::time::Duration) {
    use crate::terminal::colors;

    let (gray, reset) = if supports_color() {
        (colors::GRAY, colors::RESET)
    } else {
        ("", "")
    };

    let elapsed_ms = elapsed.as_millis();
    let time_str = if elapsed_ms < 1000 {
        format!("{}ms", elapsed_ms)
    } else {
        format!("{:.2}s", elapsed.as_secs_f64())
    };

    println!("\n{gray}Completed in {}{reset}", time_str);
}

/// Render a route summary in text format.
///
/// Human-friendly route view with algorithm annotation.
pub fn render_text(summary: &RouteSummary, show_temps: bool, base_url: &str) {
    let hops = summary.hops;
    let start = summary.start.name.as_deref().unwrap_or("<unknown>");
    let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");
    println!(
        "Route from {} to {} ({} jumps; algorithm: {}):",
        start, goal, hops, summary.algorithm
    );
    for step in &summary.steps {
        render_text_step(step, show_temps);
    }
    println!("\nTotal distance: {:.0}ly", summary.total_distance);
    println!("Total ly jumped: {:.0}ly", summary.jump_distance);

    if let Some(fuel) = &summary.fuel {
        if let Some(ship) = &fuel.ship_name {
            println!("Total fuel: {:.2} (ship: {})", fuel.total, ship);
        } else {
            println!("Total fuel: {:.2}", fuel.total);
        }

        if let Some(remaining) = fuel.remaining {
            println!("Fuel remaining: {:.2}", remaining);
        }

        for warning in &fuel.warnings {
            println!("Warning: {}", warning);
        }
    }

    if let Some(fmap_url) = &summary.fmap_url {
        println!(
            "\nfmap URL: {}{}{}",
            base_url, fmap_url, FMAP_TYPE_WIDTH_PARAM
        );
    }
}

fn render_step_with_prefix(prefix: &str, step: &RouteStep, name: &str, show_temps: bool) {
    let fuel_suffix = format_fuel_suffix(step);

    if let (Some(distance), Some(method)) = (step.distance, step.method.as_deref()) {
        if show_temps {
            if let Some(t) = step.min_external_temp {
                println!(
                    "{}{} [min {:.2}K] ({:.0}ly via {}){}",
                    prefix,
                    name,
                    t,
                    distance,
                    method,
                    fuel_suffix.as_deref().unwrap_or("")
                );
            } else {
                println!(
                    "{}{} ({:.0}ly via {}){}",
                    prefix,
                    name,
                    distance,
                    method,
                    fuel_suffix.as_deref().unwrap_or("")
                );
            }
        } else {
            println!(
                "{}{} ({:.0}ly via {}){}",
                prefix,
                name,
                distance,
                method,
                fuel_suffix.as_deref().unwrap_or("")
            );
        }
    } else if show_temps {
        if let Some(t) = step.min_external_temp {
            println!(
                "{}{} [min {:.2}K]{}",
                prefix,
                name,
                t,
                fuel_suffix.as_deref().unwrap_or("")
            );
        } else {
            println!("{}{}{}", prefix, name, fuel_suffix.as_deref().unwrap_or(""));
        }
    } else {
        println!("{}{}{}", prefix, name, fuel_suffix.as_deref().unwrap_or(""));
    }
}

fn render_text_step(step: &RouteStep, show_temps: bool) {
    let name = step.name.as_deref().unwrap_or("<unknown>");
    render_step_with_prefix(" - ", step, name, show_temps);
}

/// Render a route summary in rich text format using the library's renderer.
pub fn render_rich(summary: &RouteSummary, show_temps: bool, _base_url: &str) {
    print!(
        "{}",
        summary.render_with(RouteRenderMode::RichText, show_temps)
    );
}

/// Render a route summary in JSON format.
///
/// # Errors
///
/// Returns an error if JSON serialization or writing fails.
pub fn render_json(summary: &RouteSummary) -> io::Result<()> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, summary).map_err(io::Error::other)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

/// Render a route summary in basic path format.
///
/// Uses `+`/`|`/`-` prefixes for first/middle/last steps.
pub fn render_basic(summary: &RouteSummary, show_temps: bool, _base_url: &str) {
    let len = summary.steps.len();
    if len == 0 {
        return;
    }
    for (i, step) in summary.steps.iter().enumerate() {
        let prefix = if i == 0 {
            '+'
        } else if i + 1 == len {
            '-'
        } else {
            '|'
        };
        let name = step.name.as_deref().unwrap_or("<unknown>");
        if show_temps {
            if let Some(t) = step.min_external_temp {
                println!("{} {} [min {:.2}K]", prefix, name, t);
            } else {
                println!("{} {}", prefix, name);
            }
        } else {
            println!("{} {}", prefix, name);
        }
    }
    println!("via {} gates / {} jump drive", summary.gates, summary.jumps);
}

/// Render a route summary in emoji format.
///
/// Uses emoji markers: 🚥 (start), 📍 (waypoint), 🚀️ (destination).
pub fn render_emoji(summary: &RouteSummary, show_temps: bool, _base_url: &str) {
    let hops = summary.hops;
    let start = summary.start.name.as_deref().unwrap_or("<unknown>");
    let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");
    println!("Route from {} to {} ({} jumps):", start, goal, hops);

    let len = summary.steps.len();
    for (i, step) in summary.steps.iter().enumerate() {
        let name = step.name.as_deref().unwrap_or("<unknown>");
        let icon = if i == 0 {
            "🚥"
        } else if i + 1 == len {
            "🚀️"
        } else {
            "📍"
        };
        render_emoji_step(icon, step, name, show_temps);
    }
    println!("\nTotal distance: {:.0}ly", summary.total_distance);
    println!("Total ly jumped: {:.0}ly", summary.jump_distance);
}

fn render_emoji_step(icon: &str, step: &RouteStep, name: &str, show_temps: bool) {
    let prefix = format!(" {} ", icon);
    render_step_with_prefix(&prefix, step, name, show_temps);
}

/// Render a route summary in notepad format.
///
/// Strict notepad format using `Sta`/`Dst`/`Jmp` lines with showinfo anchors.
pub fn render_note(summary: &RouteSummary, _base_url: &str) {
    let first = summary.steps.first();
    if let Some(step) = first {
        let name = step.name.as_deref().unwrap_or("<unknown>");
        println!("Sta <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
    }
    if summary.steps.len() >= 3 {
        let step = &summary.steps[1];
        let name = step.name.as_deref().unwrap_or("<unknown>");
        println!("Dst <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
    }
    if summary.steps.len() >= 2 {
        let step = summary.steps.last().expect("len>=2 has last");
        let name = step.name.as_deref().unwrap_or("<unknown>");
        println!("Jmp <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
    }
}

/// Render a route summary in enhanced format with system details.
///
/// Enhanced format with inverted tag labels and system details
/// (temperature, planets, moons). Uses ANSI colors when available.
pub fn render_enhanced(summary: &RouteSummary, base_url: &str) {
    let palette = ColorPalette::detect();
    let renderer = EnhancedRenderer::new(palette);
    renderer.render(summary, base_url);
}

/// Renderer for enhanced output format with colored tags and system details.
pub struct EnhancedRenderer {
    palette: ColorPalette,
}

impl EnhancedRenderer {
    /// Create a new enhanced renderer with the given color palette.
    #[must_use]
    pub const fn new(palette: ColorPalette) -> Self {
        Self { palette }
    }

    /// Render a route summary.
    pub fn render(&self, summary: &RouteSummary, base_url: &str) {
        let p = &self.palette;
        let hops = summary.hops;
        let start = summary.start.name.as_deref().unwrap_or("<unknown>");
        let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");

        println!(
            "Route from {}{}{} to {}{}{} ({} jumps):",
            p.white_bold, start, p.reset, p.white_bold, goal, p.reset, hops
        );

        let len = summary.steps.len();
        for (i, step) in summary.steps.iter().enumerate() {
            let is_last = i + 1 == len;
            self.render_step(step, i == 0, is_last);
            self.render_step_details(step);
        }

        self.render_footer(summary, base_url);
    }

    fn render_step(&self, step: &RouteStep, is_first: bool, is_last: bool) {
        println!("{}", self.build_step_header_line(step, is_first, is_last));
    }

    fn build_step_header_line(&self, step: &RouteStep, is_first: bool, is_last: bool) -> String {
        let p = &self.palette;
        let name = step.name.as_deref().unwrap_or("<unknown>");

        let (tag_color, tag_text) = self.get_step_tag(step, is_first, is_last);
        let jump_type = match step.method.as_deref() {
            Some("gate") => "gate",
            Some("jump") => "jump",
            _ => "",
        };

        if let Some(distance) = step.distance {
            let dist_str = format_with_separators(distance as u64);
            let mut line = if !jump_type.is_empty() {
                format!(
                    "{}{}{} {} {}{}{} ({}, {}ly)",
                    tag_color,
                    tag_text,
                    p.reset,
                    self.get_temp_circle(step.min_external_temp.unwrap_or(0.0)),
                    p.white_bold,
                    name,
                    p.reset,
                    jump_type,
                    dist_str
                )
            } else {
                format!(
                    "{}{}{} {} {}{}{} ({}ly)",
                    tag_color,
                    tag_text,
                    p.reset,
                    self.get_temp_circle(step.min_external_temp.unwrap_or(0.0)),
                    p.white_bold,
                    name,
                    p.reset,
                    dist_str
                )
            };

            // Append planets/moons if present (use singular/plural labels and include
            // spacing so tokens don't run together).
            if let Some(planets) = step.planet_count {
                if planets > 0 {
                    let label = if planets == 1 { "Planet" } else { "Planets" };
                    line.push_str(&format!("   {}{} {}{} ", p.green, planets, label, p.reset));
                }
            }
            if let Some(moons) = step.moon_count {
                if moons > 0 {
                    let label = if moons == 1 { "Moon" } else { "Moons" };
                    line.push_str(&format!(" {}{} {}{} ", p.blue, moons, label, p.reset));
                }
            }

            line
        } else {
            format!(
                "{}{}{} {} {}{}{}",
                self.get_step_tag(step, is_first, is_last).0,
                self.get_step_tag(step, is_first, is_last).1,
                p.reset,
                self.get_temp_circle(step.min_external_temp.unwrap_or(0.0)),
                p.white_bold,
                name,
                p.reset
            )
        }
    }

    fn get_step_tag(&self, step: &RouteStep, is_first: bool, is_last: bool) -> (&str, &str) {
        let p = &self.palette;
        if is_first {
            (p.tag_start, " STRT ")
        } else if is_last {
            (p.tag_goal, " GOAL ")
        } else {
            match step.method.as_deref() {
                Some("gate") => (p.tag_gate, " GATE "),
                _ => (p.tag_jump, " JUMP "),
            }
        }
    }

    fn get_temp_circle(&self, temp: f64) -> String {
        let p = &self.palette;
        if temp > 50.0 {
            format!("{}●{}", p.red, p.reset)
        } else if temp > 20.0 {
            format!("{}●{}", p.orange, p.reset)
        } else {
            "●".to_string()
        }
    }

    fn render_step_details(&self, step: &RouteStep) {
        if let Some(line) = self.build_step_details_line(step) {
            println!("{}", line);
        }
    }

    fn render_footer(&self, summary: &RouteSummary, base_url: &str) {
        let p = &self.palette;
        let gate_distance = summary.total_distance - summary.jump_distance;
        let total_str = format_with_separators(summary.total_distance as u64);
        let gates_str = format_with_separators(gate_distance as u64);
        let jumps_str = format_with_separators(summary.jump_distance as u64);

        // Find max width for right-alignment
        let max_width = total_str.len().max(gates_str.len()).max(jumps_str.len());

        println!();
        println!(
            "{}───────────────────────────────────────{}",
            p.gray, p.reset
        );
        println!(
            "  {}Total Distance:{}  {}{:>width$}ly{}",
            p.cyan,
            p.reset,
            p.white_bold,
            total_str,
            p.reset,
            width = max_width
        );
        println!(
            "  {}Via Gates:{}       {}{:>width$}ly{}",
            p.green,
            p.reset,
            p.white_bold,
            gates_str,
            p.reset,
            width = max_width
        );
        println!(
            "  {}Via Jumps:{}       {}{:>width$}ly{}",
            p.orange,
            p.reset,
            p.white_bold,
            jumps_str,
            p.reset,
            width = max_width
        );
        if let Some(fuel) = &summary.fuel {
            let ship = fuel.ship_name.as_deref().unwrap_or("<unknown ship>");
            let total_str = format_with_separators(fuel.total.ceil() as u64);
            // Append fuel quality percent, e.g. " (10% Fuel)"
            let quality_suffix = format!(" ({:.0}% Fuel)", fuel.quality);

            // Compute number column width that lines up with distance numbers
            let mut num_width = max_width;
            num_width = num_width.max(total_str.len());

            // Remaining (if present) might be wider; include in width calculation
            let remaining_str_opt = fuel
                .remaining
                .map(|r| format_with_separators(r.ceil() as u64));
            if let Some(ref rem) = remaining_str_opt {
                num_width = num_width.max(rem.len());
            }

            println!(
                "  {}Fuel ({}):{}   {}{:>width$}{}{}",
                p.cyan,
                ship,
                p.reset,
                p.white_bold,
                total_str,
                p.reset,
                quality_suffix,
                width = num_width
            );

            if let Some(remaining) = remaining_str_opt {
                println!(
                    "  {}Remaining:{}      {}{:>width$}{}",
                    p.green,
                    p.reset,
                    p.white_bold,
                    remaining,
                    p.reset,
                    width = num_width
                );
            }

            // Heat summary (if present) — print unique warnings once on a single line.
            if let Some(heat) = &summary.heat {
                // Deduplicate warnings while preserving severity ordering.
                use std::collections::HashSet;

                let mut seen: HashSet<String> = HashSet::new();
                let mut uniques: Vec<String> = Vec::new();
                for w in &heat.warnings {
                    let t = w.trim().to_string();
                    if seen.insert(t.clone()) {
                        uniques.push(t);
                    }
                }

                // Heat footer removed: warnings were noisy and added little value in the
                // enhanced footer. Individual per-step warnings are still shown inline
                // next to hop heat values.
            }
        }

        if let Some(fmap_url) = &summary.fmap_url {
            println!();
            println!(
                "  {}fmap URL:{}        {}{}{}{}{}",
                p.cyan, p.reset, p.white_bold, base_url, fmap_url, FMAP_TYPE_WIDTH_PARAM, p.reset
            );
        }
    }
}

impl EnhancedRenderer {
    /// Build the status line for a route step.
    fn build_step_details_line(&self, step: &RouteStep) -> Option<String> {
        let p = &self.palette;
        let mut parts: Vec<String> = Vec::new();

        // Black hole systems (IDs 30000001, 30000002, 30000003)
        let is_black_hole = matches!(step.id, 30000001..=30000003);
        if is_black_hole {
            parts.push(format!("{}▌Black Hole▐{}", p.tag_black_hole, p.reset));
        }

        // Temperature (skip for black holes - they have no planets orbiting)
        if !is_black_hole {
            if let Some(t) = step.min_external_temp {
                parts.push(format!("{}min {:>6.2}K{}", p.cyan, t, p.reset));
            }
        }

        // Planets and moons are shown on the header row (after the ')') per UX
        // request, so do not include them here in the details row.

        if let Some(fuel) = step.fuel.as_ref() {
            // Display fuel as integers in the UI (fuel units operate in whole units).
            let hop_int = fuel.hop_cost.ceil() as i64;
            let mut segment = format!("{}fuel {}{}", p.orange, hop_int, p.reset);

            if let Some(rem) = fuel.remaining {
                let rem_int = rem.ceil() as i64;
                segment.push_str(&format!(" {}(rem {}){}", p.magenta, rem_int, p.reset));
            }

            parts.push(segment);
        }

        if let Some(heat) = step.heat.as_ref() {
            // Display only the hop heat and any warning; cumulative residual heat is
            // intentionally omitted from the per-step bracketed display to avoid
            // implying indefinite accumulation across hops.
            // Format hop heat: if it's non-zero but rounds to 0.00 at two decimals, show
            // a small indicator '<0.01' so users can see that heat is non-zero.
            let heat_str = if heat.hop_heat >= 0.005 {
                format!("{:.2}", heat.hop_heat)
            } else if heat.hop_heat > 0.0 {
                "<0.01".to_string()
            } else {
                "0.00".to_string()
            };

            let segment = if let Some(w) = heat.warning.as_ref() {
                // Render labels with the surrounding spaces included in the styled
                // region so their background matches the tag badges (JUMP/GATE).
                let styled_w = match w.trim() {
                    "OVERHEATED" => format!("{} {} {}", p.label_overheated, w.trim(), p.reset),
                    "CRITICAL" => format!("{} {} {}", p.label_critical, w.trim(), p.reset),
                    other => format!(" {} ", other),
                };
                // Place styled label directly after the heat value; styled_w contains spaces inside the styled region.
                format!("{}heat +{} {}{}", p.red, heat_str, styled_w, p.reset)
            } else {
                format!("{}heat +{}{}", p.red, heat_str, p.reset)
            };

            parts.push(segment);
        }

        // If this step indicates a refuel was required, append a blue REFUEL tag.
        if let Some(fuel) = step.fuel.as_ref() {
            if let Some(w) = &fuel.warning {
                if w == "REFUEL" {
                    parts.push(format!("{} REFUEL {}", p.tag_refuel, p.reset));
                }
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(format!(
            "       {}│{} {}",
            p.gray,
            p.reset,
            parts.join(&format!("{}, {}", p.gray, p.reset))
        ))
    }
}

fn format_fuel_suffix(step: &RouteStep) -> Option<String> {
    let fuel = step.fuel.as_ref()?;
    let remaining = fuel
        .remaining
        .map(|v| format!(" (remaining: {})", v.ceil() as i64))
        .unwrap_or_default();
    Some(format!(
        " | fuel: {}{}",
        fuel.hop_cost.ceil() as i64,
        remaining
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::colors;
    use evefrontier_lib::FuelProjection;

    #[test]
    fn test_enhanced_renderer_creation() {
        let palette = ColorPalette::plain();
        let renderer = EnhancedRenderer::new(palette);
        assert!(renderer.palette.reset.is_empty());
    }

    #[test]
    fn test_get_temp_circle_cold() {
        let palette = ColorPalette::plain();
        let renderer = EnhancedRenderer::new(palette);
        let circle = renderer.get_temp_circle(10.0);
        assert_eq!(circle, "●");
    }

    #[test]
    fn test_get_temp_circle_warm() {
        let palette = ColorPalette::colored();
        let renderer = EnhancedRenderer::new(palette);
        let circle = renderer.get_temp_circle(30.0);
        assert!(circle.contains('●'));
        // Should contain orange color code
        assert!(circle.contains("\x1b[38;5;208m"));
    }

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
        let step = RouteStep {
            index: 1,
            id: 30000002,
            name: Some("M 974".to_string()),
            distance: Some(3.0),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: Some(0),
            moon_count: Some(0),
            fuel: None,
            heat: None,
        };

        let line = renderer
            .build_step_details_line(&step)
            .expect("line present");
        assert!(line.contains("Black Hole"));
        assert!(!line.contains("min"));
    }

    #[test]
    fn test_build_step_details_includes_fuel_and_counts() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());
        let step = RouteStep {
            index: 2,
            id: 42,
            name: Some("Test".to_string()),
            distance: Some(10.0),
            method: Some("jump".to_string()),
            min_external_temp: Some(12.34),
            planet_count: Some(2),
            moon_count: Some(1),
            fuel: Some(FuelProjection {
                hop_cost: 5.0,
                cumulative: 5.0,
                remaining: Some(95.0),
                warning: None,
            }),
            heat: None,
        };

        let line = renderer
            .build_step_details_line(&step)
            .expect("line present");
        assert!(line.contains("min"));
        // Planets and moons moved to the header line
        assert!(!line.contains("Planets"));
        assert!(!line.contains("Moon"));
        assert!(line.contains("fuel"));

        // Header should include the planets/moons counts
        let header = renderer.build_step_header_line(&step, false, false);
        assert!(header.contains("2 Planets"));
        assert!(header.contains("1 Moon"));
    }

    #[test]
    fn test_build_step_details_colors_fuel_when_colored() {
        let renderer = EnhancedRenderer::new(ColorPalette::colored());
        let step = RouteStep {
            index: 2,
            id: 42,
            name: Some("Test".to_string()),
            distance: Some(10.0),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: Some(FuelProjection {
                hop_cost: 3.5,
                cumulative: 3.5,
                remaining: Some(96.5),
                warning: None,
            }),
            heat: None,
        };

        let line = renderer
            .build_step_details_line(&step)
            .expect("line present");
        assert!(line.contains(colors::ORANGE));
        assert!(line.contains(colors::MAGENTA));
        assert!(line.contains("fuel 4"));
        assert!(line.contains("(rem 97)"));
    }

    #[test]
    fn test_small_heat_shows_less_than_marker() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());
        let step = RouteStep {
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

        let line = renderer
            .build_step_details_line(&step)
            .expect("line present");

        assert!(line.contains("heat +<0.01"));
    }

    #[test]
    fn test_padding_consistent_for_singular_plural() {
        let renderer = EnhancedRenderer::new(ColorPalette::plain());

        let singular = RouteStep {
            index: 1,
            id: 1,
            name: Some("One".to_string()),
            distance: Some(1.0),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: Some(1),
            moon_count: Some(1),
            fuel: None,
            heat: None,
        };

        let plural = RouteStep {
            planet_count: Some(2),
            moon_count: Some(2),
            ..singular.clone()
        };

        let singular_line = renderer.build_step_details_line(&singular);
        let plural_line = renderer.build_step_details_line(&plural);

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
    }
}
