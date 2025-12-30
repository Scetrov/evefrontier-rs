//! Terminal styling and color utilities.
//!
//! This module provides ANSI escape code definitions and color detection
//! for terminal output formatting. It abstracts terminal capabilities
//! and provides a consistent interface for styled text output.

/// ANSI escape codes for text styling and colors.
///
/// All constants use raw ANSI escape sequences for maximum compatibility.
pub mod colors {
    // Reset
    /// Reset all styling.
    pub const RESET: &str = "\x1b[0m";

    // Tag colors (bold reverse video for high visibility badges)
    /// Bold reverse green for STRT tags.
    pub const TAG_START: &str = "\x1b[1;7;32m";
    /// Bold reverse cyan for GATE tags.
    pub const TAG_GATE: &str = "\x1b[1;7;36m";
    /// Bold reverse yellow for JUMP tags.
    pub const TAG_JUMP: &str = "\x1b[1;7;33m";
    /// Bold reverse magenta for GOAL tags.
    pub const TAG_GOAL: &str = "\x1b[1;7;35m";

    // Text colors
    /// Bright bold white for emphasis (system names).
    pub const WHITE_BOLD: &str = "\x1b[1;97m";
    /// Gray for secondary elements (tree lines, decorations).
    pub const GRAY: &str = "\x1b[90m";
    /// Cyan for temperature values.
    pub const CYAN: &str = "\x1b[36m";
    /// Green for planet counts and gate distances.
    pub const GREEN: &str = "\x1b[32m";
    /// Blue for moon counts.
    pub const BLUE: &str = "\x1b[34m";
    /// Orange (256-color) for warm systems (>20K).
    pub const ORANGE: &str = "\x1b[38;5;208m";
    /// Red for hot systems (>50K).
    pub const RED: &str = "\x1b[31m";
}

/// A collection of resolved color codes, either actual ANSI sequences
/// or empty strings when color is disabled.
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    pub reset: &'static str,
    pub tag_start: &'static str,
    pub tag_gate: &'static str,
    pub tag_jump: &'static str,
    pub tag_goal: &'static str,
    pub white_bold: &'static str,
    pub gray: &'static str,
    pub cyan: &'static str,
    pub green: &'static str,
    pub blue: &'static str,
    pub orange: &'static str,
    pub red: &'static str,
}

impl ColorPalette {
    /// Create a palette with actual ANSI color codes.
    #[must_use]
    pub const fn colored() -> Self {
        Self {
            reset: colors::RESET,
            tag_start: colors::TAG_START,
            tag_gate: colors::TAG_GATE,
            tag_jump: colors::TAG_JUMP,
            tag_goal: colors::TAG_GOAL,
            white_bold: colors::WHITE_BOLD,
            gray: colors::GRAY,
            cyan: colors::CYAN,
            green: colors::GREEN,
            blue: colors::BLUE,
            orange: colors::ORANGE,
            red: colors::RED,
        }
    }

    /// Create a palette with no colors (empty strings).
    #[must_use]
    pub const fn plain() -> Self {
        Self {
            reset: "",
            tag_start: "",
            tag_gate: "",
            tag_jump: "",
            tag_goal: "",
            white_bold: "",
            gray: "",
            cyan: "",
            green: "",
            blue: "",
            orange: "",
            red: "",
        }
    }

    /// Create a palette based on terminal capabilities.
    ///
    /// Returns `colored()` if the terminal supports ANSI colors,
    /// otherwise returns `plain()`.
    #[must_use]
    pub fn detect() -> Self {
        if supports_color() {
            Self::colored()
        } else {
            Self::plain()
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::detect()
    }
}

/// Check if the terminal supports ANSI color codes.
///
/// This function respects:
/// - The `NO_COLOR` environment variable (https://no-color.org/)
/// - The `TERM=dumb` convention for non-capable terminals
///
/// # Returns
///
/// `true` if color output should be used, `false` otherwise.
#[must_use]
pub fn supports_color() -> bool {
    // Respect NO_COLOR convention
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    // Respect TERM=dumb convention
    if let Ok(term) = std::env::var("TERM") {
        if term.eq_ignore_ascii_case("dumb") {
            return false;
        }
    }
    true
}

/// Check if the terminal supports Unicode characters.
///
/// This function checks for explicit Unicode support hints in
/// environment variables (`LANG`, `LC_ALL`).
///
/// # Returns
///
/// `true` if Unicode output should be used, `false` otherwise.
#[must_use]
pub fn supports_unicode() -> bool {
    // Check for explicit Unicode support hints
    if let Ok(lang) = std::env::var("LANG") {
        if lang.to_uppercase().contains("UTF") {
            return true;
        }
    }
    if let Ok(lc_all) = std::env::var("LC_ALL") {
        if lc_all.to_uppercase().contains("UTF") {
            return true;
        }
    }
    // On Windows, assume Unicode support unless TERM suggests otherwise
    #[cfg(windows)]
    {
        if let Ok(term) = std::env::var("TERM") {
            // Some legacy Windows terminals don't support Unicode
            return !term.eq_ignore_ascii_case("dumb");
        }
        return true;
    }
    // On Unix-like systems, default to false unless explicitly set
    #[cfg(not(windows))]
    {
        false
    }
}

/// Format a number with thousand separators (commas).
///
/// # Arguments
///
/// * `n` - The number to format
///
/// # Returns
///
/// A string with thousand separators, e.g., `1,234,567`.
///
/// # Examples
///
/// ```
/// # use evefrontier_cli::terminal::format_with_separators;
/// assert_eq!(format_with_separators(999), "999");
/// assert_eq!(format_with_separators(1000), "1,000");
/// assert_eq!(format_with_separators(1234567), "1,234,567");
/// ```
#[must_use]
pub fn format_with_separators(n: u64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_separators_small() {
        assert_eq!(format_with_separators(0), "0");
        assert_eq!(format_with_separators(1), "1");
        assert_eq!(format_with_separators(999), "999");
    }

    #[test]
    fn test_format_with_separators_thousands() {
        assert_eq!(format_with_separators(1000), "1,000");
        assert_eq!(format_with_separators(9999), "9,999");
    }

    #[test]
    fn test_format_with_separators_millions() {
        assert_eq!(format_with_separators(1_000_000), "1,000,000");
        assert_eq!(format_with_separators(1_234_567), "1,234,567");
    }

    #[test]
    fn test_format_with_separators_large() {
        assert_eq!(format_with_separators(1_000_000_000), "1,000,000,000");
        assert_eq!(
            format_with_separators(u64::MAX),
            "18,446,744,073,709,551,615"
        );
    }

    #[test]
    fn test_color_palette_colored() {
        let p = ColorPalette::colored();
        assert!(!p.reset.is_empty());
        assert!(!p.tag_start.is_empty());
        assert!(!p.cyan.is_empty());
    }

    #[test]
    fn test_color_palette_plain() {
        let p = ColorPalette::plain();
        assert!(p.reset.is_empty());
        assert!(p.tag_start.is_empty());
        assert!(p.cyan.is_empty());
    }

    // Note: Testing supports_color() and supports_unicode() directly is challenging
    // because they read environment variables which are global process state.
    // We use a static mutex to serialize all env-modifying tests to avoid race conditions.

    use std::sync::Mutex;

    /// Global mutex to serialize tests that modify environment variables.
    /// Environment variables are process-global, so tests modifying them must not run in parallel.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    mod supports_color_tests {
        use super::*;
        use std::env;

        /// Helper to run a test with temporary environment variable changes.
        /// Acquires the ENV_MUTEX to prevent parallel execution with other env-modifying tests.
        fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
        where
            F: FnOnce() -> R,
        {
            let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

            // Save original values
            let saved: Vec<_> = vars.iter().map(|(k, _)| (*k, env::var_os(k))).collect();

            // Set test values
            for (key, value) in vars {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }

            let result = f();

            // Restore original values
            for (key, value) in saved {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }

            result
        }

        #[test]
        fn test_supports_color_no_color_set() {
            with_env_vars(&[("NO_COLOR", Some("1")), ("TERM", None)], || {
                assert!(!supports_color(), "NO_COLOR=1 should disable colors");
            });
        }

        #[test]
        fn test_supports_color_term_dumb() {
            with_env_vars(&[("NO_COLOR", None), ("TERM", Some("dumb"))], || {
                assert!(!supports_color(), "TERM=dumb should disable colors");
            });
        }

        #[test]
        fn test_supports_color_default() {
            with_env_vars(
                &[("NO_COLOR", None), ("TERM", Some("xterm-256color"))],
                || {
                    assert!(supports_color(), "Normal terminal should support colors");
                },
            );
        }
    }

    mod supports_unicode_tests {
        use super::*;
        use std::env;

        /// Helper to run a test with temporary environment variable changes.
        /// Acquires the ENV_MUTEX to prevent parallel execution with other env-modifying tests.
        fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
        where
            F: FnOnce() -> R,
        {
            let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

            // Save original values
            let saved: Vec<_> = vars.iter().map(|(k, _)| (*k, env::var_os(k))).collect();

            // Set test values
            for (key, value) in vars {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }

            let result = f();

            // Restore original values
            for (key, value) in saved {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }

            result
        }

        #[test]
        fn test_supports_unicode_lang_utf8() {
            with_env_vars(&[("LANG", Some("en_US.UTF-8")), ("LC_ALL", None)], || {
                assert!(supports_unicode(), "LANG=en_US.UTF-8 should enable Unicode");
            });
        }

        #[test]
        fn test_supports_unicode_lc_all_utf8() {
            with_env_vars(&[("LANG", None), ("LC_ALL", Some("C.UTF-8"))], || {
                assert!(supports_unicode(), "LC_ALL=C.UTF-8 should enable Unicode");
            });
        }

        #[test]
        #[cfg(not(windows))]
        fn test_supports_unicode_no_utf_hint() {
            with_env_vars(&[("LANG", Some("C")), ("LC_ALL", None)], || {
                assert!(
                    !supports_unicode(),
                    "Non-UTF locale should disable Unicode on Unix"
                );
            });
        }
    }
}
