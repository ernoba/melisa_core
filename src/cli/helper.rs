// ============================================================================
// src/cli/helper.rs
//
// FIXES APPLIED:
//  1. Hapus `CmdKind` dari import — tidak ada di rustyline 13.
//     Signature `highlight_char` di rustyline 13 pakai `forced: bool`, bukan CmdKind.
//  2. Menyelesaikan konflik nama antara Trait dan Derive Macro. Kita mengimpor
//     Trait dari submodul masing-masing, dan memanggil derive macro menggunakan
//     path eksplisit `rustyline::NamaMacro` di dalam `#[derive(...)]`.
// ============================================================================

use std::collections::HashSet;
use std::borrow::Cow;

// 1. Impor Traits dari submodul spesifik (dibutuhkan untuk `impl`)
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{Validator, MatchingBracketValidator};
use rustyline::history::SearchDirection;
use rustyline::Context;

/// MELISA CLI Helper
/// Integrates intelligent history-based autocompletion, file path completion,
/// and syntax highlighting for an advanced REPL experience.
// 2. Gunakan `rustyline::` agar compiler tahu kita memanggil Derive Macro, bukan Trait
#[derive(rustyline::Helper, rustyline::Validator, rustyline::Hinter)]
pub struct MelisaHelper {
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,

    // Note: #[rustyline(Highlighter)] is intentionally omitted here
    // so we can manually implement the trait to customize hint colors.
    pub highlighter: MatchingBracketHighlighter,

    #[rustyline(Validator)]
    pub validator: MatchingBracketValidator,

    pub file_completer: FilenameCompleter,
}

impl Highlighter for MelisaHelper {
    /// Delegates standard syntax highlighting (like matching brackets) to the default highlighter.
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    /// Handles character-level highlighting events as the user types.
    ///
    /// FIX: rustyline 13 mengubah parameter ketiga dari `kind: CmdKind` menjadi `forced: bool`.
    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }

    /// Customizes the visual appearance of history hints.
    /// Wraps the suggested hint in ANSI escape codes to render it in a sleek, muted gray.
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Completer for MelisaHelper {
    type Candidate = Pair;

    /// Provides dynamic autocompletion based on context (File paths vs. Command History).
    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {

        // 1. CONTEXTUAL ROUTING: File Path Completion
        // If the user is navigating directories or typing a file path, route to the native file completer.
        if line.starts_with("cd ") || line[..pos].contains('/') {
            return self.file_completer.complete(line, pos, ctx);
        }

        // 2. HISTORY-BASED AUTOCOMPLETION
        // Capture the exact phrase the user has typed up to the cursor's current position.
        let prefix = &line[..pos];

        // Prevent duplicate suggestions from cluttering the autocomplete menu.
        let mut seen = HashSet::new();
        let mut suggest = Vec::new();
        let history = ctx.history();

        // 3. REVERSE HISTORY TRAVERSAL
        // Iterate backwards through the history to prioritize the most recently executed commands.
        for i in (0..history.len()).rev() {
            // Retrieve the history entry safely
            if let Ok(Some(entry)) = history.get(i, SearchDirection::Forward) {
                let cmd_string = &entry.entry;

                // Check if the history entry starts with the user's current input
                if cmd_string.starts_with(prefix) {
                    // Only allocate and insert if we haven't seen this exact command before
                    if seen.insert(cmd_string.to_string()) {
                        suggest.push(Pair {
                            display: cmd_string.to_string(),
                            replacement: cmd_string.to_string(),
                        });
                    }
                }
            }

            // Cap the suggestions at 10 to keep the terminal UI clean and responsive
            if suggest.len() >= 10 {
                break;
            }
        }

        // 4. RESOLUTION & FALLBACK
        if !suggest.is_empty() {
            // CRITICAL: Return index 0. This instructs rustyline to replace the *entire* line
            // with the chosen suggestion, rather than just appending to the last typed word.
            Ok((0, suggest))
        } else {
            // Fallback: If no history matches are found, default to the standard file completer
            // to ensure the user always gets some form of useful assistance.
            self.file_completer.complete(line, pos, ctx)
        }
    }
}