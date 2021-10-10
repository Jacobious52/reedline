use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::{core_editor::LineBuffer, Completer, CompletionActionHandler, DefaultCompleter, Span};

/// A simple handler that will do a cycle-based rotation through the options given by the Completer
pub struct CmdCompletionHandler {
    completer: Box<dyn Completer>,
    command: Command,
}

impl CmdCompletionHandler {
    /// Build a `CmdCompletionHandler` configured to use a specific completer
    ///
    /// # Arguments
    ///
    /// * `completer`    The completion logic to use
    ///
    /// # Example
    /// ```
    /// use reedline::{CmdCompletionHandler, DefaultCompleter, Completer, Span};
    ///
    /// let mut completer = DefaultCompleter::default();
    /// completer.insert(vec!["test-hyphen","test_underscore"].iter().map(|s| s.to_string()).collect());
    /// assert_eq!(
    ///     completer.complete("te",2),
    ///     vec![(Span { start: 0, end: 2 }, "test".into())]);
    ///
    /// let mut completions = CmdCompletionHandler::default().with_completer(Box::new(completer));
    /// ```
    pub fn with_completer(mut self, completer: Box<dyn Completer>) -> CmdCompletionHandler {
        self.completer = completer;
        self
    }

    pub fn with_command(mut self, command: Command) -> CmdCompletionHandler {
        self.command = command;
        self
    }
}

impl Default for CmdCompletionHandler {
    fn default() -> Self {
        CmdCompletionHandler {
            completer: Box::new(DefaultCompleter::default()),
            command: Command::new("fzf"),
        }
    }
}

impl CompletionActionHandler for CmdCompletionHandler {
    // With this function we handle the tab events.
    //
    // If completions vector is not empty we proceed to replace
    //  in the line_buffer only the specified range of characters.
    // If internal index is 0 it means that is the first tab event pressed.
    // If internal index is greater than completions vector, we bring it back to 0.
    fn handle(&mut self, present_buffer: &mut LineBuffer) {
        let completions = self
            .completer
            .complete(present_buffer.get_buffer(), present_buffer.offset());

        let mut cmd = self
            .command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let mut stdin = cmd.stdin.take().expect("Failed to open stdin");
            stdin
                .write_all(
                    completions
                        .iter()
                        .map(|c| c.1.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                        .as_bytes(),
                )
                .unwrap();
            stdin.flush().unwrap();
        }

        let output = cmd.wait_with_output().expect("Failed to read stdout");
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.is_empty() {
                return;
            }
            present_buffer.clear();
            present_buffer.insert_str(&result.trim_end());
        }
    }
}
