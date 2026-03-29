// ============================================================================
// src/cli/prompt.rs
//
// Dynamic prompt string builder and command-history management.
//
// [`Prompt`] resolves the executing user's identity and home directory at
// construction time and exposes a [`Prompt::build`] method that formats the
// interactive shell prompt string for Rustyline.
//
// [`reset_history`] atomically purges the in-memory history buffer and the
// on-disk history file, then re-initialises an empty file with 0600 perms.
// ============================================================================

use std::env;
use std::io::ErrorKind;
use rustyline::Editor;
use rustyline::history::FileHistory;
use tokio::fs;

use crate::cli::color::{BLUE, BOLD, GREEN, RED, RESET, YELLOW};
use crate::cli::helper::MelisaHelper;

// ── Prompt ────────────────────────────────────────────────────────────────────

/// Dynamically constructs the interactive terminal prompt string.
///
/// Adapts to the execution context (Standard User vs. Sudo / Root) by reading
/// `SUDO_USER` before `USER` so that the prompt never falsely claims "root"
/// when an admin escalates privileges.
pub struct Prompt {
    /// The resolved login username shown in the prompt.
    pub user: String,
    /// The resolved home directory used for path abbreviation (`~`).
    pub home: String,
}

impl Prompt {
    /// Initialises a new `Prompt` by resolving the executing user's environment.
    pub fn new() -> Self {
        let user = env::var("SUDO_USER")
            .or_else(|_| env::var("USER"))
            .or_else(|_| env::var("LOGNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        // Resolve the home directory to enable path abbreviation
        // (e.g., replacing `/home/alice` with `~`).
        let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());

        Self { user, home }
    }

    /// Compiles and formats the final prompt string injected into the Rustyline editor.
    ///
    /// # Returns
    /// A coloured prompt of the form `melisa@<user>:<cwd>> `.
    pub fn build(&self) -> String {
        let curr_path = env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .replace(&self.home, "~");

        // Output format: melisa@username:~/current/path>
        format!("{BOLD}{GREEN}melisa@{}{RESET}:{BLUE}{}{RESET}> ", self.user, curr_path)
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new()
    }
}

// ── History management ────────────────────────────────────────────────────────

/// Safely purges the command history from both the in-memory buffer and disk.
///
/// Steps performed:
/// 1. Clear the Rustyline in-memory history buffer.
/// 2. Delete the physical history file (TOCTOU-safe: delete then handle error).
/// 3. Re-initialise an empty file so Rustyline does not crash on exit.
/// 4. Enforce 0600 permissions on the new file (`#[cfg(unix)]`).
///
/// # Arguments
/// * `rl`           - The active Rustyline editor instance.
/// * `history_path` - Absolute path to the session history file.
pub async fn reset_history(rl: &mut Editor<MelisaHelper, FileHistory>, history_path: &str) {
    // 1. Purge the in-memory history buffer (application-level, atomic).
    let _ = rl.clear_history();

    // 2. Attempt physical deletion without prior metadata checks.
    //    This strictly prevents TOCTOU race conditions.
    match fs::remove_file(history_path).await {
        Ok(_) => {
            println!(
                "{}[SUCCESS]{} Local history file has been permanently deleted.",
                GREEN, RESET
            );
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                println!(
                    "{}[INFO]{} History file does not exist or was already removed.",
                    YELLOW, RESET
                );
            }
            ErrorKind::PermissionDenied => {
                eprintln!(
                    "{}[ERROR]{} Cannot delete history: Permission denied. Escalation required.",
                    RED, RESET
                );
            }
            _ => {
                eprintln!(
                    "{}[ERROR]{} Unexpected I/O error during history reset: {}",
                    RED, RESET, e
                );
            }
        },
    }

    // 3. Re-initialise a clean, empty history file to synchronise editor state.
    //    This prevents Rustyline from crashing on exit when it saves the session.
    if let Err(e) = rl.save_history(history_path) {
        eprintln!(
            "{}[WARNING]{} Failed to re-initialise an empty history file: {}",
            YELLOW, RESET, e
        );
    } else {
        // 4. Enforce strict 0600 (owner read/write only) on the new file.
        //    Command history files can contain sensitive parameters.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // std::fs is used here because permissions must be set immediately
            // after the synchronous rustyline save_history call.
            if let Ok(mut perms) = std::fs::metadata(history_path).map(|m| m.permissions()) {
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(history_path, perms);
            }
        }
    }

    println!(
        "{}[DONE]{} Command history purge sequence completed successfully.",
        GREEN, RESET
    );
}