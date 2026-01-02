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
        // Pre-compute column widths for details rows so numeric columns can be
        // right-aligned across the entire route output.
        let widths = compute_details_column_widths(&summary.steps);

        for (i, step) in summary.steps.iter().enumerate() {
            let is_last = i + 1 == len;
            self.render_step(step, i == 0, is_last);
            self.render_step_details(step, &widths);
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

            // Append planets/moons if present (use singular/plural labels).
            // Build tokens without leading/trailing spaces and then join them with a
            // single space to guarantee at least one visible gap between tokens.
            let mut tokens: Vec<String> = Vec::new();
            if let Some(planets) = step.planet_count {
                if planets > 0 {
                    let label = if planets == 1 { "Planet" } else { "Planets" };
                    tokens.push(format!("{}{} {}{}", p.green, planets, label, p.reset));
                }
            }
            if let Some(moons) = step.moon_count {
                if moons > 0 {
                    let label = if moons == 1 { "Moon" } else { "Moons" };
                    tokens.push(format!("{}{} {}{}", p.blue, moons, label, p.reset));
                }
            }

            if !tokens.is_empty() {
                // Prefix with three spaces for alignment and join tokens with single space
                line.push_str("   ");
                line.push_str(&tokens.join(" "));
                line.push(' ');
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

    fn render_step_details(&self, step: &RouteStep, widths: &ColumnWidths) {
        if let Some(line) = self.build_step_details_line(step, widths) {
            println!("{}", line);
        }
    }

    fn build_step_details_line(&self, step: &RouteStep, widths: &ColumnWidths) -> Option<String> {
        let p = &self.palette;
        let is_black_hole = matches!(step.id, 30000001..=30000003);

        // 1. MIN segment (fixed width 11)
        const MIN_SEG_VISIBLE_WIDTH: usize = 11;
        let min_seg = if is_black_hole {
            format!("{}▌Black Hole▐{}", p.tag_black_hole, p.reset)
        } else if let Some(t) = step.min_external_temp {
            format!("{}min {:>6.2}K{}", p.cyan, t, p.reset)
        } else {
            " ".repeat(MIN_SEG_VISIBLE_WIDTH)
        };

        // 2. Fuel Cost segment
        let fuel_cost_seg = if widths.fuel_val_width > 0 {
            if let Some(f) = step.fuel.as_ref() {
                let hop_int = f.hop_cost.ceil() as i64;
                format!(
                    "{}fuel {:>width$}{}",
                    p.orange,
                    hop_int,
                    p.reset,
                    width = widths.fuel_val_width
                )
            } else {
                format!("     {:>width$}", "", width = widths.fuel_val_width)
            }
        } else {
            String::new()
        };

        // 3. Fuel Remaining segment
        let fuel_rem_seg = if widths.rem_val_width > 0 {
            if let Some(f) = step.fuel.as_ref() {
                if let Some(rem) = f.remaining {
                    let rem_int = rem.ceil() as i64;
                    format!(
                        "{}(rem {:>width$}){}",
                        p.magenta,
                        rem_int,
                        p.reset,
                        width = widths.rem_val_width
                    )
                } else {
                    " ".repeat(6 + widths.rem_val_width)
                }
            } else {
                " ".repeat(6 + widths.rem_val_width)
            }
        } else {
            String::new()
        };

        // Combine Fuel
        let fuel_seg = if !fuel_cost_seg.is_empty() {
            if !fuel_rem_seg.is_empty() {
                format!("{} {}", fuel_cost_seg, fuel_rem_seg)
            } else {
                fuel_cost_seg
            }
        } else {
            String::new()
        };

        // 4. Heat Cost segment
        let heat_cost_seg = if widths.heat_val_width > 0 {
            if let Some(h) = step.heat.as_ref() {
                let heat_str = if h.hop_heat >= 0.005 {
                    format!("{:.2}", h.hop_heat)
                } else if h.hop_heat > 0.0 {
                    "<0.01".to_string()
                } else {
                    "0.00".to_string()
                };
                format!(
                    "{}heat +{:>width$}{}",
                    p.red,
                    heat_str,
                    p.reset,
                    width = widths.heat_val_width
                )
            } else {
                format!("      {:>width$}", "", width = widths.heat_val_width)
            }
        } else {
            String::new()
        };

        // 5. Tags segment
        let mut tags = Vec::new();
        if let Some(h) = step.heat.as_ref() {
            if let Some(w) = &h.warning {
                let styled_w = match w.trim() {
                    "OVERHEATED" => format!("{} {} {}", p.label_overheated, w.trim(), p.reset),
                    "CRITICAL" => format!("{} {} {}", p.label_critical, w.trim(), p.reset),
                    other => format!(" {} ", other),
                };
                tags.push(styled_w);
            }
        }
        if let Some(f) = step.fuel.as_ref() {
            if let Some(w) = &f.warning {
                if w == "REFUEL" {
                    tags.push(format!("{} REFUEL {}", p.tag_refuel, p.reset));
                }
            }
        }
        let tags_seg = tags.join("  ");

        // Final assembly
        let mut segments = Vec::new();
        segments.push(min_seg);
        if !fuel_seg.is_empty() {
            segments.push(fuel_seg);
        }
        if !heat_cost_seg.is_empty() {
            segments.push(heat_cost_seg);
        }
        if !tags_seg.is_empty() {
            segments.push(tags_seg);
        }

        // Check content
        let has_fuel = step.fuel.is_some();
        let has_heat = step.heat.is_some();
        if !is_black_hole && !has_fuel && !has_heat && step.min_external_temp.is_none() {
            return None;
        }

        let joined = segments.join(", ");
        Some(format!("       {}│{} {}", p.gray, p.reset, joined))
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

/// Column widths for the details row alignment.
#[derive(Debug, Default)]
struct ColumnWidths {
    fuel_val_width: usize,
    rem_val_width: usize,
    heat_val_width: usize,
}

fn compute_details_column_widths(steps: &[RouteStep]) -> ColumnWidths {
    let mut fuel_val_width = 0usize;
    let mut rem_val_width = 0usize;
    let mut heat_val_width = 0usize;

    for step in steps {
        if let Some(fuel) = step.fuel.as_ref() {
            let hop = format!("{}", fuel.hop_cost.ceil() as i64);
            fuel_val_width = fuel_val_width.max(hop.len());
            if let Some(r) = fuel.remaining {
                let rem = format!("{}", r.ceil() as i64);
                rem_val_width = rem_val_width.max(rem.len());
            }
        }

        if let Some(h) = step.heat.as_ref() {
            let heat_str = if h.hop_heat >= 0.005 {
                format!("{:.2}", h.hop_heat)
            } else if h.hop_heat > 0.0 {
                "<0.01".to_string()
            } else {
                "0.00".to_string()
            };
            heat_val_width = heat_val_width.max(heat_str.len());
        }
    }

    ColumnWidths {
        fuel_val_width,
        rem_val_width,
        heat_val_width,
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

        let widths = compute_details_column_widths(std::slice::from_ref(&step));
        let line = renderer
            .build_step_details_line(&step, &widths)
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
            RouteStep {
                index: 1,
                id: 1,
                name: Some("Step 1".to_string()),
                distance: Some(1.0),
                method: Some("jump".to_string()),
                min_external_temp: None,
                planet_count: Some(1),
                moon_count: Some(1),
                fuel: Some(FuelProjection {
                    hop_cost: 3.0,
                    cumulative: 3.0,
                    remaining: Some(90.0),
                    warning: None,
                }),
                heat: None,
            },
            RouteStep {
                index: 2,
                id: 2,
                name: Some("Step 2".to_string()),
                distance: Some(2.0),
                method: Some("gate".to_string()),
                min_external_temp: Some(10.0),
                planet_count: Some(2),
                moon_count: Some(2),
                fuel: Some(FuelProjection {
                    hop_cost: 4.5,
                    cumulative: 7.5,
                    remaining: Some(85.5),
                    warning: None,
                }),
                heat: None,
            },
            RouteStep {
                index: 3,
                id: 3,
                name: Some("Step 3".to_string()),
                distance: Some(3.0),
                method: Some("jump".to_string()),
                min_external_temp: Some(20.0),
                planet_count: Some(3),
                moon_count: Some(3),
                fuel: Some(FuelProjection {
                    hop_cost: 5.0,
                    cumulative: 12.5,
                    remaining: Some(80.0),
                    warning: None,
                }),
                heat: None,
            },
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
            RouteStep {
                index: 1,
                id: 1,
                name: Some("A".to_string()),
                distance: Some(1.0),
                method: Some("jump".to_string()),
                min_external_temp: None,
                planet_count: Some(1),
                moon_count: Some(1),
                fuel: Some(FuelProjection {
                    hop_cost: 3.0,
                    cumulative: 3.0,
                    remaining: Some(90.0),
                    warning: None,
                }),
                heat: None,
            },
            RouteStep {
                index: 2,
                id: 2,
                name: Some("B".to_string()),
                distance: Some(2.0),
                method: Some("gate".to_string()),
                min_external_temp: Some(10.0),
                planet_count: Some(2),
                moon_count: Some(2),
                fuel: Some(FuelProjection {
                    hop_cost: 4.5,
                    cumulative: 7.5,
                    remaining: Some(85.5),
                    warning: None,
                }),
                heat: None,
            },
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
