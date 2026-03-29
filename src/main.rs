// ============================================================================
// src/main.rs
//
// MELISA server entry point.
//
// Bootstraps the Tokio async runtime, performs privilege escalation via
// the SUID binary, and launches the REPL loop.
// ============================================================================

// ── Lint enforcement (see CODING_STANDARD.md §10) ────────────────────────────
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod cli;
pub mod core;
pub mod deployment;
pub mod distros;

use std::env;
use std::process;

#[tokio::main]
async fn main() {
    // Privilege escalation: if we are not running as root, re-exec via sudo.
    if !is_running_as_root() {
        re_exec_as_root();
    }

    let current_user = env::var("SUDO_USER")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string());

    let home_dir = env::var("HOME").unwrap_or_else(|_| format!("/home/{}", current_user));

    // Launch the REPL (defined in cli/melisa_cli.rs).
    cli::repl::start_repl(&current_user, &home_dir).await;
}

/// Returns `true` if the process is running with effective UID 0 (root).
fn is_running_as_root() -> bool {
    // SAFETY: getuid() is always safe to call.
    unsafe { libc::geteuid() == 0 }
}

/// Re-executes the current binary via `sudo` and exits the current process.
fn re_exec_as_root() {
    let current_binary = env::current_exe().unwrap_or_else(|_| {
        eprintln!("MELISA: Failed to resolve the current executable path.");
        process::exit(1);
    });

    let args: Vec<String> = env::args().skip(1).collect();

    let mut sudo_cmd = process::Command::new("sudo");
    sudo_cmd.arg("-E");
    sudo_cmd.arg(current_binary);
    sudo_cmd.args(&args);

    let status = sudo_cmd.status().unwrap_or_else(|err| {
        eprintln!("MELISA: Failed to re-exec via sudo: {}", err);
        process::exit(1);
    });

    process::exit(status.code().unwrap_or(1));
}