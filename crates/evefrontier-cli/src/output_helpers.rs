use crate::output::FMAP_TYPE_WIDTH_PARAM;
use crate::terminal::{colors, ColorPalette};
use evefrontier_lib::{RouteStep, RouteSummary};

/// Column widths for the details row alignment.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ColumnWidths {
    pub(crate) fuel_val_width: usize,
    pub(crate) rem_val_width: usize,
    pub(crate) heat_val_width: usize,
    pub(crate) cooldown_val_width: usize,
}

/// Compute column widths used by the enhanced renderer for right alignment.
pub(crate) fn compute_details_column_widths(steps: &[RouteStep]) -> ColumnWidths {
    let mut fuel_val_width = 0usize;
    let mut rem_val_width = 0usize;
    let mut heat_val_width = 0usize;
    let mut cooldown_val_width = 0usize;

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

            if let Some(wait) = h.wait_time_seconds {
                if wait > 0.5 {
                    let cd_str = format_cooldown_duration(wait);
                    cooldown_val_width = cooldown_val_width.max(cd_str.len());
                }
            }
        }
    }

    ColumnWidths {
        fuel_val_width,
        rem_val_width,
        heat_val_width,
        cooldown_val_width,
    }
}

#[test]
fn build_enhanced_footer_includes_params() {
    use crate::terminal::ColorPalette;
    use evefrontier_lib::output::RouteParametersSummary;
    use evefrontier_lib::routing::{RouteAlgorithm, RouteOptimization};
    use evefrontier_lib::{RouteEndpoint, RouteOutputKind};

    let palette = ColorPalette::plain();
    let summary = RouteSummary {
        kind: RouteOutputKind::Route,
        algorithm: RouteAlgorithm::AStar,
        hops: 3,
        gates: 1,
        jumps: 2,
        total_distance: 100.0,
        jump_distance: 50.0,
        start: RouteEndpoint {
            id: 1,
            name: Some("A".to_string()),
        },
        goal: RouteEndpoint {
            id: 2,
            name: Some("B".to_string()),
        },
        steps: Vec::new(),
        fuel: None,
        heat: None,
        fmap_url: None,
        parameters: Some(RouteParametersSummary {
            algorithm: RouteAlgorithm::AStar,
            optimization: RouteOptimization::Fuel,
            fuel_quality: 10.0,
            ship_name: Some("Reflex".to_string()),
            avoid_critical_state: true,
            max_spatial_neighbors: Some(250),
            avoid_gates: false,
            max_jump: None,
        }),
    };

    let lines = build_enhanced_footer(&summary, "https://fmap/", &palette);
    let params_line = lines.iter().find(|l| l.contains("Parameters:"));
    assert!(
        params_line.is_some(),
        "expected a Parameters line in footer"
    );
    let pl = params_line.unwrap();
    let pl_clean = strip_ansi_to_string(pl);
    assert!(pl_clean.contains("Optimize") || pl_clean.contains("Optimize:"));
    assert!(pl_clean.contains("Ship: Reflex"));
    assert!(pl_clean.contains("Fuel quality: 10%"));
}

/// Format a small fuel suffix used in compact text renderers.
pub(crate) fn format_fuel_suffix(step: &RouteStep) -> Option<String> {
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

/// Format a cooling duration into a concise string like "2m4s".
pub(crate) fn format_cooldown_duration(seconds: f64) -> String {
    if seconds <= 0.0 {
        return "0s".to_string();
    }
    let total_secs = seconds.round() as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;

    if mins > 0 {
        format!("{}m{}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Build the estimation warning box as a string so tests can inspect it.
pub(crate) fn build_estimation_warning_box(
    prefix_visible: &str,
    msg: &str,
    supports_unicode: bool,
) -> String {
    // Compute visible width while ignoring ANSI color escape sequences so callers can
    // pass colored prefixes (e.g., "\x1b[34m🛈 INFO\x1b[0m") without breaking alignment.
    // left padding (1) + separator (1) + (message + prefix) visible width
    // The visible inner content measured by tests strips the leading and trailing
    // single-space padding, so compute the repeat count accordingly.
    let prefix_count = strip_ansi_to_string(prefix_visible).chars().count();
    let msg_count = strip_ansi_to_string(msg).chars().count();
    // include left padding (1), separator (1), and right padding (1)
    let inner_width = prefix_count + 1 + msg_count + 2;

    if supports_unicode {
        let mut out = String::new();
        out.push_str(&format!("┌{}┐\n", "─".repeat(inner_width)));
        out.push_str(&format!("│ {} {} │\n", prefix_visible, msg));
        out.push_str(&format!("└{}┘\n", "─".repeat(inner_width)));
        out
    } else {
        let mut out = String::new();
        out.push_str(&format!("+{}+\n", "-".repeat(inner_width)));
        out.push_str(&format!("| {} {} |\n", prefix_visible, msg));
        out.push_str(&format!("+{}+\n", "-".repeat(inner_width)));
        out
    }
}

// (additional tests added later in the file)

/// Strip ANSI color escape sequences and return the cleaned string.
///
/// Note: use `strip_ansi_to_string(..).chars().count()` when you need the visible width.
pub(crate) fn strip_ansi_to_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        if c == '\x1b' {
            if let Some('[') = iter.peek() {
                iter.next();
            }
            for ch in iter.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Print estimation warning box using gray/reset color markers (for simple renderers).
pub(crate) fn print_estimation_warning_box_gray_reset(gray: &str, reset: &str) {
    use crate::terminal::colors;
    use crate::terminal::supports_unicode;
    let msg = "All fuel and heat values are based upon estimations of the code that CCP uses; they may deviate by up to ±10%";
    let prefix_visible = "🛈 INFO";

    let prefix = if crate::terminal::supports_color() {
        format!("{}{}{}", colors::BLUE, "🛈 INFO", reset)
    } else {
        prefix_visible.to_string()
    };

    let s = build_estimation_warning_box(&prefix, msg, supports_unicode());
    for line in s.lines() {
        println!("{}{}{}", gray, line, reset);
    }
}

/// Print estimation warning box using a color palette (used by EnhancedRenderer).
pub(crate) fn print_estimation_warning_box_with_palette(palette: &ColorPalette) {
    let msg = "All fuel and heat values are based upon estimations of the code that CCP uses; they may deviate by up to ±10%";
    let prefix_visible = "🛈 INFO";
    let prefix_colored = if crate::terminal::supports_color() {
        format!("{}{}{}", palette.blue, prefix_visible, palette.reset)
    } else {
        prefix_visible.to_string()
    };

    let s = build_estimation_warning_box(&prefix_colored, msg, crate::terminal::supports_unicode());
    println!("{}", s);
}

/// Print the footer with elapsed time.
#[allow(dead_code)]
pub(crate) fn print_footer(elapsed: std::time::Duration) {
    let (gray, reset) = if crate::terminal::supports_color() {
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

    println!("\n{}Completed in {}{}", gray, time_str, reset);
}

#[cfg(test)]
mod footer_tests {
    use super::*;

    #[test]
    fn print_footer_smoke() {
        // ensure it doesn't panic
        print_footer(std::time::Duration::from_millis(123));
    }
}

/// Return a temperature indicator circle using colors in the palette.
pub(crate) fn get_temp_circle(temp: f64, palette: &ColorPalette) -> String {
    if temp > 50.0 {
        format!("{}●{}", palette.red, palette.reset)
    } else if temp > 20.0 {
        format!("{}●{}", palette.orange, palette.reset)
    } else {
        "●".to_string()
    }
}

/// Build the MIN segment for a step details line.
pub(crate) fn build_min_segment(step: &RouteStep, palette: &ColorPalette) -> String {
    let is_black_hole = matches!(step.id, 30000001..=30000003);
    const MIN_SEG_VISIBLE_WIDTH: usize = 11;
    if is_black_hole {
        format!("{}▌Black Hole▐{}", palette.tag_black_hole, palette.reset)
    } else if let Some(t) = step.min_external_temp {
        format!("{}min {:>6.2}K{}", palette.cyan, t, palette.reset)
    } else {
        " ".repeat(MIN_SEG_VISIBLE_WIDTH)
    }
}

/// Build the fuel cost and remaining segments combined (if any).
pub(crate) fn build_fuel_segment(
    step: &RouteStep,
    widths: &ColumnWidths,
    palette: &ColorPalette,
) -> Option<String> {
    if widths.fuel_val_width > 0 {
        if let Some(f) = step.fuel.as_ref() {
            let hop_int = f.hop_cost.ceil() as i64;
            let fuel_cost_seg = format!(
                "{}fuel {:>width$}{}",
                palette.orange,
                hop_int,
                palette.reset,
                width = widths.fuel_val_width
            );

            let fuel_rem_seg = if widths.rem_val_width > 0 {
                if let Some(rem) = f.remaining {
                    let rem_int = rem.ceil() as i64;
                    Some(format!(
                        "{}(rem {:>width$}){}",
                        palette.magenta,
                        rem_int,
                        palette.reset,
                        width = widths.rem_val_width
                    ))
                } else {
                    Some(" ".repeat(6 + widths.rem_val_width))
                }
            } else {
                None
            };

            let mut res = if let Some(rem) = fuel_rem_seg {
                format!("{} {}", fuel_cost_seg, rem)
            } else {
                fuel_cost_seg
            };

            if let Some(w) = &f.warning {
                if w == "REFUEL" {
                    res.push_str(&format!(" {}{}{}", palette.tag_refuel, w, palette.reset));
                }
            }

            Some(res)
        } else {
            Some(format!(
                "     {:>width$}",
                "",
                width = widths.fuel_val_width
            ))
        }
    } else {
        None
    }
}

/// Build the heat cost segment (if any).
pub(crate) fn build_heat_segment(
    step: &RouteStep,
    widths: &ColumnWidths,
    palette: &ColorPalette,
) -> Option<String> {
    if widths.heat_val_width > 0 {
        if let Some(h) = step.heat.as_ref() {
            let heat_str = if h.hop_heat >= 0.005 {
                format!("{:.2}", h.hop_heat)
            } else if h.hop_heat > 0.0 {
                "<0.01".to_string()
            } else {
                "0.00".to_string()
            };
            let heat_part = format!(
                "{}heat {:>width$}{}",
                palette.red,
                heat_str,
                palette.reset,
                width = widths.heat_val_width
            );

            let cooldown_part = if widths.cooldown_val_width > 0 {
                if let Some(wait) = h.wait_time_seconds {
                    if wait > 0.5 {
                        let cd_str = format_cooldown_duration(wait);
                        Some(format!(
                            " {}({} to cool){}",
                            palette.gray, cd_str, palette.reset
                        ))
                    } else {
                        Some(" ".repeat(11 + widths.cooldown_val_width))
                    }
                } else {
                    Some(" ".repeat(11 + widths.cooldown_val_width))
                }
            } else {
                None
            };

            let mut res = heat_part;

            if let Some(w) = &h.warning {
                let styled_w = match w.trim() {
                    "OVERHEATED" => {
                        format!(" {}{}{}", palette.label_overheated, w.trim(), palette.reset)
                    }
                    "CRITICAL" => {
                        format!(" {}{}{}", palette.label_critical, w.trim(), palette.reset)
                    }
                    other => format!(" {} ", other),
                };
                res.push_str(&styled_w);
            }

            if let Some(cd) = cooldown_part {
                res.push_str(&cd);
            }

            Some(res)
        } else {
            let mut padding = 6 + widths.heat_val_width;
            if widths.cooldown_val_width > 0 {
                padding += 11 + widths.cooldown_val_width;
            }
            Some(" ".repeat(padding))
        }
    } else {
        None
    }
}

/// Build a list of planet/moon tokens for header line (e.g., "2 Planets", "1 Moon").
pub(crate) fn build_planet_moon_tokens(step: &RouteStep, palette: &ColorPalette) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    if let Some(planets) = step.planet_count {
        if planets > 0 {
            let label = if planets == 1 { "Planet" } else { "Planets" };
            tokens.push(format!(
                "{}{} {}{}",
                palette.green, planets, label, palette.reset
            ));
        }
    }
    if let Some(moons) = step.moon_count {
        if moons > 0 {
            let label = if moons == 1 { "Moon" } else { "Moons" };
            tokens.push(format!(
                "{}{} {}{}",
                palette.blue, moons, label, palette.reset
            ));
        }
    }
    tokens
}

/// Build the enhanced footer as a list of lines so callers can print them.
pub fn build_enhanced_footer(
    summary: &RouteSummary,
    base_url: &str,
    palette: &ColorPalette,
) -> Vec<String> {
    use crate::terminal::format_with_separators;

    let p = palette;
    let gate_distance = summary.total_distance - summary.jump_distance;
    let total_str = format_with_separators(summary.total_distance as u64);
    let gates_str = format_with_separators(gate_distance as u64);
    let jumps_str = format_with_separators(summary.jump_distance as u64);

    // Find max width for right-alignment
    let max_width = total_str.len().max(gates_str.len()).max(jumps_str.len());

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "{}───────────────────────────────────────{}",
        p.gray, p.reset
    ));
    lines.push(format!(
        "  {}Total Distance:{}  {}{:>width$}ly{}",
        p.cyan,
        p.reset,
        p.white_bold,
        total_str,
        p.reset,
        width = max_width
    ));
    lines.push(format!(
        "  {}Via Gates:{}       {}{:>width$}ly{}",
        p.green,
        p.reset,
        p.white_bold,
        gates_str,
        p.reset,
        width = max_width
    ));
    lines.push(format!(
        "  {}Via Jumps:{}       {}{:>width$}ly{}",
        p.orange,
        p.reset,
        p.white_bold,
        jumps_str,
        p.reset,
        width = max_width
    ));

    if let Some(fuel) = &summary.fuel {
        let ship = fuel.ship_name.as_deref().unwrap_or("<unknown ship>");
        let total_str = format_with_separators(fuel.total.ceil() as u64);
        let quality_suffix = format!(" ({:.0}% Fuel)", fuel.quality);

        let mut num_width = max_width;
        num_width = num_width.max(total_str.len());
        let remaining_str_opt = fuel
            .remaining
            .map(|r| format_with_separators(r.ceil() as u64));
        if let Some(ref rem) = remaining_str_opt {
            num_width = num_width.max(rem.len());
        }

        lines.push(format!(
            "  {}Fuel ({}):{}   {}{:>width$}{}{}",
            p.cyan,
            ship,
            p.reset,
            p.white_bold,
            total_str,
            p.reset,
            quality_suffix,
            width = num_width
        ));

        if let Some(remaining) = remaining_str_opt {
            lines.push(format!(
                "  {}Remaining:{}      {}{:>width$}{}",
                p.green,
                p.reset,
                p.white_bold,
                remaining,
                p.reset,
                width = num_width
            ));
        }
    }

    if let Some(fmap_url) = &summary.fmap_url {
        lines.push(String::new());
        lines.push(format!(
            "  {}fmap URL:{}        {}{}{}{}{}",
            p.cyan, p.reset, p.white_bold, base_url, fmap_url, FMAP_TYPE_WIDTH_PARAM, p.reset
        ));
    }

    // Render an applied-parameters summary in a human-friendly form
    if let Some(params) = &summary.parameters {
        let algo = format!("{}", params.algorithm);
        let optimization = match params.optimization {
            evefrontier_lib::routing::RouteOptimization::Fuel => "Fuel",
            evefrontier_lib::routing::RouteOptimization::Distance => "Distance",
        };
        let ship = params.ship_name.as_deref().unwrap_or("<none>");
        let fuel_q = format!("{:.0}%", params.fuel_quality);
        let avoid_crit_val = if params.avoid_critical_state {
            "Yes"
        } else {
            "No"
        };
        let max_sp = params
            .max_spatial_neighbors
            .map(|n| n.to_string())
            .unwrap_or_else(|| "auto".to_string());
        let avoid_gates_val = if params.avoid_gates { "Yes" } else { "No" };

        lines.push(String::new());
        lines.push(format!(
            "  {}Parameters:{}  {}Algorithm:{} {} • {}Optimize:{} {} • {}Ship:{} {} • {}Fuel quality:{} {} • {}Avoid critical state:{} {} • {}Max spatial neighbors:{} {} • {}Avoid gates:{} {}",
            p.cyan,
            p.reset,
            p.magenta,
            p.reset,
            algo,
            p.magenta,
            p.reset,
            optimization,
            p.magenta,
            p.reset,
            ship,
            p.magenta,
            p.reset,
            fuel_q,
            p.magenta,
            p.reset,
            avoid_crit_val,
            p.magenta,
            p.reset,
            max_sp,
            p.magenta,
            p.reset,
            avoid_gates_val
        ));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::ColorPalette;
    use crate::test_helpers::RouteStepBuilder;

    #[test]
    fn build_box_colored_alignment() {
        use crate::terminal::colors;

        let msg = "All fuel and heat values are based upon estimations of the code that CCP uses; they may deviate by up to ±10%";
        let prefix_colored = format!("{}{}{}", colors::BLUE, "🛈 INFO", colors::RESET);
        let s = build_estimation_warning_box(&prefix_colored, msg, true);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3);
        let top = lines[0];
        let mid = lines[1];
        let bot = lines[2];

        // Visible middle content length (strip ANSI & border spaces)
        let visible_mid = strip_ansi_to_string(mid);
        let inner = visible_mid
            .trim_start_matches('│')
            .trim_start_matches(' ')
            .trim_end_matches(' ')
            .trim_end_matches('│');
        let inner_len = inner.chars().count() + 1;

        // Top and bottom should match the inner width (count of box drawing dashes)
        let expected_top = format!("┌{}┐", "─".repeat(inner_len));
        let expected_bot = format!("└{}┘", "─".repeat(inner_len));
        // (no debug prints)
        assert_eq!(top, expected_top);
        assert_eq!(bot, expected_bot);
        assert!(mid.contains("🛈 INFO"));
    }

    #[test]
    fn compute_widths_empty() {
        let steps: Vec<evefrontier_lib::RouteStep> = Vec::new();
        let widths = compute_details_column_widths(&steps);
        assert_eq!(widths, ColumnWidths::default());
    }

    #[test]
    fn compute_widths_values() {
        let steps = vec![crate::test_helpers::RouteStepBuilder::new()
            .with_fuel_projection(evefrontier_lib::FuelProjection {
                hop_cost: 12.3,
                cumulative: 12.3,
                remaining: Some(123.0),
                warning: None,
            })
            .with_heat(evefrontier_lib::ship::HeatProjection {
                hop_heat: 0.02,
                warning: None,
                wait_time_seconds: None,
                residual_heat: None,
                can_proceed: true,
            })
            .build()];

        let widths = compute_details_column_widths(&steps);
        assert_eq!(widths.fuel_val_width, 2); // "13"
        assert_eq!(widths.rem_val_width, 3); // "123"
        assert_eq!(widths.heat_val_width, 4); // "0.02"
    }

    #[test]
    fn format_fuel_suffix_none() {
        let step = RouteStepBuilder::new().build();
        assert!(format_fuel_suffix(&step).is_none());
    }

    #[test]
    fn format_fuel_suffix_some() {
        let step = RouteStepBuilder::new()
            .with_fuel_projection(evefrontier_lib::FuelProjection {
                hop_cost: 3.5,
                cumulative: 3.5,
                remaining: Some(96.2),
                warning: None,
            })
            .build();
        let s = format_fuel_suffix(&step).unwrap();
        assert!(s.contains("fuel: 4"));
        assert!(s.contains("remaining: 97"));
    }

    #[test]
    fn build_box_unicode() {
        let box_str = build_estimation_warning_box("🛈 INFO", "msg", true);
        assert!(box_str.contains("┌") && box_str.contains("┘"));
        assert!(box_str.contains("🛈 INFO"));
        assert!(box_str.contains("msg"));
    }

    #[test]
    fn build_box_ascii() {
        let box_str = build_estimation_warning_box("INFO", "m", false);
        assert!(box_str.contains("+") && box_str.contains("-"));
        assert!(box_str.contains("INFO"));
        assert!(box_str.contains("m"));
    }

    #[test]
    fn get_temp_circle_tests() {
        let p = ColorPalette::plain();
        assert_eq!(get_temp_circle(10.0, &p), "●");
        let p2 = ColorPalette::colored();
        let warm = get_temp_circle(30.0, &p2);
        assert!(warm.contains('●'));
        assert!(warm.contains(p2.orange));
        let hot = get_temp_circle(80.0, &p2);
        assert!(hot.contains(p2.red));
    }

    #[test]
    fn build_min_segment_black_hole() {
        let p = ColorPalette::plain();
        let step = RouteStep {
            index: 1,
            id: 30000002,
            name: None,
            distance: None,
            method: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: None,
        };
        let seg = build_min_segment(&step, &p);
        assert!(seg.contains("Black Hole"));
    }

    #[test]
    fn build_fuel_and_remaining_segment() {
        let p = ColorPalette::plain();
        let step = RouteStep {
            index: 2,
            id: 42,
            name: None,
            distance: None,
            method: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: Some(evefrontier_lib::FuelProjection {
                hop_cost: 3.5,
                cumulative: 3.5,
                remaining: Some(96.5),
                warning: None,
            }),
            heat: None,
        };

        let widths = ColumnWidths {
            fuel_val_width: 1,
            rem_val_width: 2,
            ..Default::default()
        };

        let s = build_fuel_segment(&step, &widths, &p).expect("fuel seg");
        assert!(s.contains("fuel 4"));
        assert!(s.contains("(rem 97)"));
    }

    #[test]
    fn build_heat_segment_small() {
        let p = ColorPalette::plain();
        let step = RouteStep {
            index: 2,
            id: 42,
            name: None,
            distance: None,
            method: None,
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

        let widths = ColumnWidths {
            heat_val_width: 5,
            ..Default::default()
        };
        let s = build_heat_segment(&step, &widths, &p).expect("heat seg");
        assert!(s.contains("<0.01"));
    }

    #[test]
    fn build_segments_include_warning_tags() {
        let p = ColorPalette::plain();
        let step = RouteStep {
            index: 2,
            id: 42,
            name: None,
            distance: None,
            method: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: Some(evefrontier_lib::FuelProjection {
                hop_cost: 3.5,
                cumulative: 3.5,
                remaining: None,
                warning: Some("REFUEL".to_string()),
            }),
            heat: Some(evefrontier_lib::ship::HeatProjection {
                hop_heat: 0.1,
                warning: Some("OVERHEATED".to_string()),
                wait_time_seconds: None,
                residual_heat: None,
                can_proceed: false,
            }),
        };

        let widths = ColumnWidths {
            fuel_val_width: 5,
            heat_val_width: 5,
            ..Default::default()
        };

        let f = build_fuel_segment(&step, &widths, &p).expect("fuel seg");
        assert!(f.contains("REFUEL"));

        let h = build_heat_segment(&step, &widths, &p).expect("heat seg");
        assert!(h.contains("OVERHEATED"));
        // Tag should now come BEFORE the result of any padding or cooldown part
        // (though in this test wait_time is None)
    }

    #[test]
    fn build_heat_segment_alignment() {
        let p = ColorPalette::plain();
        let step = RouteStep {
            index: 2,
            id: 42,
            name: None,
            distance: None,
            method: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: Some(evefrontier_lib::ship::HeatProjection {
                hop_heat: 100.0,
                warning: Some("OVERHEATED".to_string()),
                wait_time_seconds: Some(60.0),
                residual_heat: None,
                can_proceed: true,
            }),
        };

        let widths = ColumnWidths {
            heat_val_width: 6,     // "100.00"
            cooldown_val_width: 4, // "1m0s"
            ..Default::default()
        };

        let s = build_heat_segment(&step, &widths, &p).expect("heat seg");
        let s_clean = strip_ansi_to_string(&s);
        // Desired: "heat 100.00 OVERHEATED (1m0s to cool)" (with spaces)
        assert!(s_clean.contains("OVERHEATED (1m0s to cool)"));
    }

    #[test]
    fn test_format_cooldown_duration() {
        assert_eq!(format_cooldown_duration(0.0), "0s");
        assert_eq!(format_cooldown_duration(-5.0), "0s");
        assert_eq!(format_cooldown_duration(45.0), "45s");
        assert_eq!(format_cooldown_duration(60.0), "1m0s");
        assert_eq!(format_cooldown_duration(124.0), "2m4s");
        assert_eq!(format_cooldown_duration(3600.0), "60m0s");
    }
}
