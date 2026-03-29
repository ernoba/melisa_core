// ============================================================================
// src/cli/melisa_cli.rs
//
// Primary entry point for the MELISA interactive shell (REPL).
//
// Initializes the Rustyline editor, injects the MelisaHelper, loads session
// history, and drives the main input loop.  Each command is dispatched to
// `executor::execute_command` which returns an [`ExecResult`] that controls
// loop continuation.
// ============================================================================

use tokio::fs;
use rustyline::{Editor, Config};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::completion::FilenameCompleter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;

use crate::cli::color::{RED, YELLOW, GREEN, RESET, BOLD};
use crate::cli::executor::{execute_command, ExecResult};
use crate::cli::helper::MelisaHelper;
use crate::cli::prompt::{Prompt, reset_history};

// ── REPL entry point ──────────────────────────────────────────────────────────

/// Launches the MELISA interactive shell.
///
/// Resolves user identity and home directory via [`Prompt::new`], configures
/// the Rustyline editor with history and autocompletion, then drives the main
/// read-eval-print loop until the user exits.
pub async fn melisa() {
    // 1. Initialize context & path resolution.
    let p_info = Prompt::new();

    // Align the history storage with the system directories established by
    // the bash installer.
    let melisa_dir  = format!("{}/.local/share/melisa", p_info.home);
    let history_path = format!("{}/history.txt", melisa_dir);

    // Ensure the global MELISA directory exists asynchronously.
    let _ = fs::create_dir_all(&melisa_dir).await;

    // 2. Configure the Rustyline editor.
    let config = Config::builder()
        .history_ignore_dups(true).unwrap_or_default()
        .auto_add_history(false) // History is managed manually for security.
        .build();

    let mut rl: Editor<MelisaHelper, FileHistory> = Editor::with_config(config)
        .expect("FATAL: Failed to allocate terminal interface for Rustyline Editor");

    // Inject the custom MELISA helper (autocompletion, highlighting, validation).
    rl.set_helper(Some(MelisaHelper {
        hinter:         HistoryHinter {},
        highlighter:    MatchingBracketHighlighter::new(),
        validator:      MatchingBracketValidator::new(),
        file_completer: FilenameCompleter::new(),
    }));

    // Load previous session history if it exists.
    let _ = rl.load_history(&history_path);

    println!("{}Authenticated as {}. Secure session granted.{}", BOLD, p_info.user, RESET);

    // 3. Core execution loop.
    loop {
        let prompt_str = p_info.build();

        // Note: rustyline::readline blocks the current thread while waiting for
        // user input.  This is necessary for raw terminal mode handling.
        match rl.readline(&prompt_str) {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() { continue; }

                match execute_command(input, &p_info.user, &p_info.home).await {
                    ExecResult::ResetHistory => {
                        // Delegate history reset authority back to the main loop.
                        reset_history(&mut rl, &history_path).await;
                    }
                    ExecResult::Continue => {
                        // Register command to RAM then sync delta to disk.
                        let _ = rl.add_history_entry(input);
                        // append_history writes only the new line, saving I/O.
                        let _ = rl.append_history(&history_path);
                    }
                    ExecResult::Break => {
                        // Terminal exit requested.  Save state then break.
                        let _ = rl.add_history_entry(input);
                        let _ = rl.append_history(&history_path);
                        break;
                    }
                    ExecResult::Error(e) => {
                        // Log to history so the user can press Up to fix typos.
                        let _ = rl.add_history_entry(input);
                        let _ = rl.append_history(&history_path);
                        eprintln!("{}[ERROR]{} {}", RED, RESET, e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C: cancel current line but keep the session alive.
                println!("{}[CTRL+C] Operation aborted.{}", YELLOW, RESET);
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D: gracefully terminate the session.
                println!("{}[EXIT] Secure session terminated.{}", GREEN, RESET);
                break;
            }
            Err(err) => {
                eprintln!("{}[FATAL]{} Readline encountered a critical error: {:?}", RED, RESET, err);
                break;
            }
        }
    }
}