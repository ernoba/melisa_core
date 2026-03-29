// ============================================================================
// src/cli/color.rs
//
// ANSI terminal color constants and formatted print helpers.
// All display output goes through these constants to ensure consistent
// styling across the entire MELISA CLI.
// ============================================================================

/// ANSI escape code for green text (success state).
pub const GREEN: &str = "\x1b[32m";

/// ANSI escape code for red text (error / danger state).
pub const RED: &str = "\x1b[31m";

/// ANSI escape code for cyan text (informational / deployment state).
pub const CYAN: &str = "\x1b[36m";

/// ANSI escape code for bold bright-blue text (section headers).
pub const BLUE: &str = "\x1b[34;1m";

/// ANSI escape code for yellow text (warning / cache / skip state).
pub const YELLOW: &str = "\x1b[33m";

/// ANSI escape code for bold text (emphasis).
pub const BOLD: &str = "\x1b[1m";

/// ANSI escape code that resets all formatting to terminal default.
pub const RESET: &str = "\x1b[0m";

// ── Formatted print helpers ─────────────────────────────────────────────────

/// Prints a formatted error message to `stderr`.
///
/// Renders as: `error: <message>`  (red label, reset body)
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!("{}error:{} {}", RED, RESET, message);
}

/// Prints a formatted success message to `stdout`.
///
/// Renders as: `success: <message>`  (green label, reset body)
#[allow(dead_code)]
pub fn print_success(message: &str) {
    println!("{}success:{} {}", GREEN, RESET, message);
}

/// Prints a formatted warning message to `stdout`.
///
/// Renders as: `[WARNING] <message>` (yellow label, reset body)
#[allow(dead_code)]
pub fn print_warning(message: &str) {
    println!("{}[WARNING]{} {}", YELLOW, RESET, message);
}

/// Prints a formatted informational message to `stdout`.
///
/// Renders as: `[INFO] <message>` (bold label, reset body)
#[allow(dead_code)]
pub fn print_info(message: &str) {
    println!("{}[INFO]{} {}", BOLD, RESET, message);
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants_are_valid_ansi_sequences() {
        // Every color constant must start with the ESC character (\x1b)
        // and end with 'm' — the standard ANSI SGR terminator.
        for (name, code) in &[
            ("GREEN", GREEN),
            ("RED", RED),
            ("CYAN", CYAN),
            ("BLUE", BLUE),
            ("YELLOW", YELLOW),
            ("BOLD", BOLD),
        ] {
            assert!(
                code.starts_with("\x1b["),
                "Color constant '{}' must start with ESC sequence \\x1b[",
                name
            );
            assert!(
                code.ends_with('m'),
                "Color constant '{}' must end with ANSI SGR terminator 'm'",
                name
            );
        }
    }

    #[test]
    fn test_reset_constant_is_correct_ansi_reset() {
        assert_eq!(
            RESET, "\x1b[0m",
            "RESET must be the standard ANSI SGR reset sequence"
        );
    }

    #[test]
    fn test_color_constants_are_distinct() {
        let codes = [GREEN, RED, CYAN, BLUE, YELLOW, BOLD];
        for i in 0..codes.len() {
            for j in (i + 1)..codes.len() {
                assert_ne!(
                    codes[i], codes[j],
                    "Color constants at index {} and {} must not be identical",
                    i, j
                );
            }
        }
    }
}