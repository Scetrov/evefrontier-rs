//! Enhanced renderer extracted from `output.rs` to reduce module size.

use crate::terminal::{format_with_separators, ColorPalette};
use evefrontier_lib::{RouteStep, RouteSummary};

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
        let widths = crate::output_helpers::compute_details_column_widths(&summary.steps);

        for (i, step) in summary.steps.iter().enumerate() {
            let is_last = i + 1 == len;
            self.render_step(step, i == 0, is_last);
            self.render_step_details(step, &widths);
        }

        // Render footer via helper to keep this file smaller
        let lines = crate::output_helpers::build_enhanced_footer(summary, base_url, p);
        println!();
        for line in lines {
            println!("{}", line);
        }

        if summary.fuel.is_some() || summary.heat.is_some() {
            println!();
            crate::output_helpers::print_estimation_warning_box_with_palette(&self.palette);
        }
    }

    fn render_step(&self, step: &RouteStep, is_first: bool, is_last: bool) {
        println!("{}", self.build_step_header_line(step, is_first, is_last));
    }

    pub(crate) fn build_step_header_line(
        &self,
        step: &RouteStep,
        is_first: bool,
        is_last: bool,
    ) -> String {
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
                    crate::output_helpers::get_temp_circle(
                        step.min_external_temp.unwrap_or(0.0),
                        p
                    ),
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
                    crate::output_helpers::get_temp_circle(
                        step.min_external_temp.unwrap_or(0.0),
                        p
                    ),
                    p.white_bold,
                    name,
                    p.reset,
                    dist_str
                )
            };
            // Append planets/moons if present using helper to keep this file smaller
            let tokens = crate::output_helpers::build_planet_moon_tokens(step, p);
            if !tokens.is_empty() {
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
                crate::output_helpers::get_temp_circle(step.min_external_temp.unwrap_or(0.0), p),
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

    #[cfg(test)]
    pub fn get_temp_circle(&self, temp: f64) -> String {
        crate::output_helpers::get_temp_circle(temp, &self.palette)
    }

    // Palette accessor was only used in tests previously; removed to avoid dead_code warnings.

    // removed duplicated wrapper; `get_temp_circle` is public

    fn render_step_details(&self, step: &RouteStep, widths: &crate::output_helpers::ColumnWidths) {
        if let Some(line) = self.build_step_details_line(step, widths) {
            println!("{}", line);
        }
    }

    pub(crate) fn build_step_details_line(
        &self,
        step: &RouteStep,
        widths: &crate::output_helpers::ColumnWidths,
    ) -> Option<String> {
        let p = &self.palette;
        let is_black_hole = matches!(step.id, 30000001..=30000003);

        // Delegate to helpers
        let min_seg = crate::output_helpers::build_min_segment(step, p);
        let fuel_seg_opt = crate::output_helpers::build_fuel_segment(step, widths, p);
        let heat_seg_opt = crate::output_helpers::build_heat_segment(step, widths, p);
        let tags_opt = crate::output_helpers::build_tags_segment(step, p);

        let has_fuel = step.fuel.is_some();
        let has_heat = step.heat.is_some();
        if !is_black_hole && !has_fuel && !has_heat && step.min_external_temp.is_none() {
            return None;
        }

        let mut segments = Vec::new();
        segments.push(min_seg);
        if let Some(s) = fuel_seg_opt {
            segments.push(s);
        }
        if let Some(s) = heat_seg_opt {
            segments.push(s);
        }
        if let Some(s) = tags_opt {
            segments.push(s);
        }

        let joined = segments.join(", ");
        Some(format!("       {}│{} {}", p.gray, p.reset, joined))
    }
}
