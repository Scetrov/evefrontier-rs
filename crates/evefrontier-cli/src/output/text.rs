use std::io;
use std::io::Write;

use evefrontier_lib::{RouteRenderMode, RouteStep, RouteSummary};

use crate::output_helpers::{format_fuel_suffix, print_estimation_warning_box_gray_reset};
use crate::terminal::colors;
use crate::terminal::supports_color;

/// Render a route summary in text format.
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
            base_url,
            fmap_url,
            super::FMAP_TYPE_WIDTH_PARAM
        );
    }

    if summary.fuel.is_some() || summary.heat.is_some() {
        let (gray, reset) = if supports_color() {
            (colors::GRAY, colors::RESET)
        } else {
            ("", "")
        };
        print_estimation_warning_box_gray_reset(gray, reset);
    }
}

fn render_text_step(step: &RouteStep, show_temps: bool) {
    let name = step.name.as_deref().unwrap_or("<unknown>");
    let fuel_suffix = format_fuel_suffix(step);

    if let (Some(distance), Some(method)) = (step.distance, step.method.as_deref()) {
        if show_temps {
            if let Some(t) = step.min_external_temp {
                println!(
                    " - {} [min {:.2}K] ({:.0}ly via {}){}",
                    name,
                    t,
                    distance,
                    method,
                    fuel_suffix.as_deref().unwrap_or("")
                );
            } else {
                println!(
                    " - {} ({:.0}ly via {}){}",
                    name,
                    distance,
                    method,
                    fuel_suffix.as_deref().unwrap_or("")
                );
            }
        } else {
            println!(
                " - {} ({:.0}ly via {}){}",
                name,
                distance,
                method,
                fuel_suffix.as_deref().unwrap_or("")
            );
        }
    } else if show_temps {
        if let Some(t) = step.min_external_temp {
            println!(
                " - {} [min {:.2}K]{}",
                name,
                t,
                fuel_suffix.as_deref().unwrap_or("")
            );
        } else {
            println!(" - {}{}", name, fuel_suffix.as_deref().unwrap_or(""));
        }
    } else {
        println!(" - {}{}", name, fuel_suffix.as_deref().unwrap_or(""));
    }
}

/// Render a route summary in rich text format using the library's renderer.
pub fn render_rich(summary: &RouteSummary, show_temps: bool, _base_url: &str) {
    print!(
        "{}",
        summary.render_with(RouteRenderMode::RichText, show_temps)
    );

    if summary.fuel.is_some() || summary.heat.is_some() {
        let (gray, reset) = if supports_color() {
            (colors::GRAY, colors::RESET)
        } else {
            ("", "")
        };
        println!();
        print_estimation_warning_box_gray_reset(gray, reset);
    }
}

/// Render a route summary in JSON format.
pub fn render_json(summary: &RouteSummary) -> io::Result<()> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, summary).map_err(io::Error::other)?;
    stdout.write_all(b"\n").map_err(io::Error::other)
}

/// Render a route summary in basic path format.
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

    if summary.fuel.is_some() || summary.heat.is_some() {
        let (gray, reset) = if supports_color() {
            (colors::GRAY, colors::RESET)
        } else {
            ("", "")
        };
        println!();
        print_estimation_warning_box_gray_reset(gray, reset);
    }
}

/// Render a route summary in emoji format.
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
        let prefix = format!(" {} ", icon);
        // reuse text step rendering logic with prefix
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
    println!("\nTotal distance: {:.0}ly", summary.total_distance);
    println!("Total ly jumped: {:.0}ly", summary.jump_distance);

    if summary.fuel.is_some() || summary.heat.is_some() {
        let (gray, reset) = if supports_color() {
            (colors::GRAY, colors::RESET)
        } else {
            ("", "")
        };
        println!();
        print_estimation_warning_box_gray_reset(gray, reset);
    }
}

/// Render a route summary in notepad format.
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

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lib::{RouteAlgorithm, RouteEndpoint, RouteOutputKind, RouteSummary};

    fn minimal_summary() -> RouteSummary {
        RouteSummary {
            kind: RouteOutputKind::Route,
            algorithm: RouteAlgorithm::AStar,
            hops: 0,
            gates: 0,
            jumps: 0,
            total_distance: 0.0,
            jump_distance: 0.0,
            start: RouteEndpoint { id: 0, name: None },
            goal: RouteEndpoint { id: 0, name: None },
            steps: Vec::new(),
            fuel: None,
            heat: None,
            fmap_url: None,
        }
    }

    #[test]
    fn smoke_text_renderers() {
        let summary = minimal_summary();
        render_basic(&summary, false, "");
        render_emoji(&summary, false, "");
        render_note(&summary, "");
    }
}
