//! Output formatting for route rendering.
//!
//! This module provides formatters for rendering route summaries
//! in various output formats (text, rich, enhanced, etc.).

use std::io::{self, Write};

use evefrontier_lib::{RouteRenderMode, RouteStep, RouteSummary};

use crate::terminal::{format_with_separators, supports_color, ColorPalette};

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
pub fn render_text(summary: &RouteSummary, show_temps: bool) {
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
}

fn render_step_with_prefix(prefix: &str, step: &RouteStep, name: &str, show_temps: bool) {
    if let (Some(distance), Some(method)) = (step.distance, step.method.as_deref()) {
        if show_temps {
            if let Some(t) = step.min_external_temp {
                println!(
                    "{}{} [min {:.2}K] ({:.0}ly via {})",
                    prefix, name, t, distance, method
                );
            } else {
                println!("{}{} ({:.0}ly via {})", prefix, name, distance, method);
            }
        } else {
            println!("{}{} ({:.0}ly via {})", prefix, name, distance, method);
        }
    } else if show_temps {
        if let Some(t) = step.min_external_temp {
            println!("{}{} [min {:.2}K]", prefix, name, t);
        } else {
            println!("{}{}", prefix, name);
        }
    } else {
        println!("{}{}", prefix, name);
    }
}

fn render_text_step(step: &RouteStep, show_temps: bool) {
    let name = step.name.as_deref().unwrap_or("<unknown>");
    render_step_with_prefix(" - ", step, name, show_temps);
}

/// Render a route summary in rich text format using the library's renderer.
pub fn render_rich(summary: &RouteSummary, show_temps: bool) {
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
pub fn render_basic(summary: &RouteSummary, show_temps: bool) {
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
pub fn render_emoji(summary: &RouteSummary, show_temps: bool) {
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
pub fn render_note(summary: &RouteSummary) {
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
pub fn render_enhanced(summary: &RouteSummary) {
    let palette = ColorPalette::detect();
    let renderer = EnhancedRenderer::new(palette);
    renderer.render(summary);
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
    pub fn render(&self, summary: &RouteSummary) {
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
            if !is_last {
                self.render_step_details(step);
            }
        }

        self.render_footer(summary);
    }

    fn render_step(&self, step: &RouteStep, is_first: bool, is_last: bool) {
        let p = &self.palette;
        let name = step.name.as_deref().unwrap_or("<unknown>");

        // Determine tag based on position and method
        let (tag_color, tag_text) = self.get_step_tag(step, is_first, is_last);

        // Determine jump type label
        let jump_type = match step.method.as_deref() {
            Some("gate") => "gate",
            Some("jump") => "jump",
            _ => "",
        };

        // Circle color based on temperature
        let circle = self.get_temp_circle(step.min_external_temp.unwrap_or(0.0));

        // Print the tag and system name
        if let Some(distance) = step.distance {
            let dist_str = format_with_separators(distance as u64);
            if !jump_type.is_empty() {
                println!(
                    "{}{}{} {} {}{}{} ({}, {}ly)",
                    tag_color,
                    tag_text,
                    p.reset,
                    circle,
                    p.white_bold,
                    name,
                    p.reset,
                    jump_type,
                    dist_str
                );
            } else {
                println!(
                    "{}{}{} {} {}{}{} ({}ly)",
                    tag_color, tag_text, p.reset, circle, p.white_bold, name, p.reset, dist_str
                );
            }
        } else {
            println!(
                "{}{}{} {} {}{}{}",
                tag_color, tag_text, p.reset, circle, p.white_bold, name, p.reset
            );
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
        let p = &self.palette;
        let mut parts: Vec<String> = Vec::new();

        // Temperature
        if let Some(t) = step.min_external_temp {
            parts.push(format!("{}min {:>6.2}K{}", p.cyan, t, p.reset));
        }

        // Planets (omit if zero)
        let planets = step.planet_count.unwrap_or(0);
        if planets > 0 {
            let label = if planets == 1 { "Planet" } else { "Planets" };
            parts.push(format!("{}{:>2} {}{}", p.green, planets, label, p.reset));
        }

        // Moons (omit if zero)
        let moons = step.moon_count.unwrap_or(0);
        if moons > 0 {
            let label = if moons == 1 { "Moon" } else { "Moons" };
            parts.push(format!("{}{:>2} {}{}", p.blue, moons, label, p.reset));
        }

        if !parts.is_empty() {
            println!(
                "       {}│{} {}",
                p.gray,
                p.reset,
                parts.join(&format!("{}, {}", p.gray, p.reset))
            );
        }
    }

    fn render_footer(&self, summary: &RouteSummary) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
