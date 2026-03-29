// ============================================================================
// src/cli/loading.rs
//
// Provides the `execute_with_spinner` helper that wraps any async operation
// in an animated progress indicator.  When `audit` mode is enabled the
// spinner is suppressed so that raw subprocess output is visible.
// ============================================================================

use std::future::Future;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

// ── Spinner style ────────────────────────────────────────────────────────────

/// Tick characters used to animate the spinner.
const SPINNER_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

/// Milliseconds between each spinner frame.
const SPINNER_TICK_MS: u64 = 100;

/// Template string for `indicatif` progress bar formatting.
const SPINNER_TEMPLATE: &str = "{spinner:.cyan} {msg}";

// ── Public API ───────────────────────────────────────────────────────────────

/// Executes an async `operation` while displaying a spinning progress indicator.
///
/// When `audit` is `true` the progress bar is hidden so that raw subprocess
/// output reaches the terminal unobstructed.
///
/// # Type Parameters
/// * `F`  - Closure type that accepts a `ProgressBar` reference and returns a `Future`.
/// * `Fut`- Future returned by `F`.
/// * `T`  - Output type produced by `Fut`.
///
/// # Arguments
/// * `label`     - Short description shown next to the spinner.
/// * `operation` - Closure containing the async work to perform.
/// * `audit`     - When `true`, spinner is hidden and raw output is inherited.
///
/// # Returns
/// The value produced by `operation`.
pub async fn execute_with_spinner<F, Fut, T>(label: &str, operation: F, audit: bool) -> T
where
    F: FnOnce(ProgressBar) -> Fut,
    Fut: Future<Output = T>,
{
    if audit {
        // In audit mode use a silent, non-drawing placeholder so the
        // caller still receives a ProgressBar it can call `.println()` on.
        let silent_bar = ProgressBar::hidden();
        return operation(silent_bar).await;
    }

    let spinner = build_spinner(label);
    let result = operation(spinner.clone()).await;
    spinner.finish_and_clear();
    result
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Constructs and starts a styled `ProgressBar` spinner.
fn build_spinner(label: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .tick_chars(SPINNER_CHARS)
            .template(SPINNER_TEMPLATE)
            .expect("Spinner template must be valid"),
    );
    bar.set_message(label.to_string());
    bar.enable_steady_tick(Duration::from_millis(SPINNER_TICK_MS));
    bar
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `execute_with_spinner` returns the correct value when
    /// the operation completes successfully in audit mode.
    #[tokio::test]
    async fn test_execute_with_spinner_returns_operation_result_in_audit_mode() {
        let expected = 42_u32;
        let actual = execute_with_spinner(
            "Computing answer to everything",
            |_pb| async move { expected },
            true, // audit = true → spinner is hidden
        )
        .await;

        assert_eq!(
            actual, expected,
            "execute_with_spinner must propagate the operation's return value unchanged"
        );
    }

    /// Verifies that `execute_with_spinner` returns the correct value when
    /// the operation completes successfully with the spinner visible.
    #[tokio::test]
    async fn test_execute_with_spinner_returns_operation_result_with_spinner() {
        let result = execute_with_spinner(
            "Running test workload",
            |_pb| async move { "hello" },
            false, // audit = false → spinner is shown
        )
        .await;

        assert_eq!(
            result, "hello",
            "execute_with_spinner must propagate the operation's return value when spinner is active"
        );
    }

    /// Verifies that `execute_with_spinner` can handle operations that produce
    /// an `Option<T>` result.
    #[tokio::test]
    async fn test_execute_with_spinner_handles_option_result() {
        let result: Option<String> = execute_with_spinner(
            "Looking up value",
            |_pb| async move { Some("found".to_string()) },
            true,
        )
        .await;

        assert!(
            result.is_some(),
            "execute_with_spinner must not drop the inner Option value"
        );
        assert_eq!(
            result.unwrap(),
            "found",
            "execute_with_spinner must preserve the inner Option value"
        );
    }
}