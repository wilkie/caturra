//! Console IO for the running program.
//!
//! `System.out.println` and friends need somewhere to go, and `Scanner`
//! / `System.in` need somewhere to come from. The VM writes through the
//! [`ConsoleIo`] trait; the WASM boundary implements it by forwarding to
//! JavaScript callbacks, and tests use [`BufferedConsole`].

/// Host hooks for the running program's standard streams.
pub trait ConsoleIo {
    /// Write bytes to standard out.
    fn stdout(&mut self, bytes: &[u8]);

    /// Write bytes to standard error.
    fn stderr(&mut self, bytes: &[u8]);

    /// Read one line from standard in, without the trailing newline.
    /// Returns `None` at end of input.
    fn read_line(&mut self) -> Option<String>;
}

/// An in-memory console: collects output and serves scripted input.
/// Used by tests and by batch (non-interactive) runs.
#[derive(Debug, Default)]
pub struct BufferedConsole {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    input: Vec<String>,
    next_input: usize,
}

impl BufferedConsole {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A console pre-loaded with lines of standard input.
    #[must_use]
    pub fn with_input(lines: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            input: lines.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Everything the program wrote to standard out, as UTF-8.
    #[must_use]
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// Everything the program wrote to standard error, as UTF-8.
    #[must_use]
    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

impl ConsoleIo for BufferedConsole {
    fn stdout(&mut self, bytes: &[u8]) {
        self.stdout.extend_from_slice(bytes);
    }

    fn stderr(&mut self, bytes: &[u8]) {
        self.stderr.extend_from_slice(bytes);
    }

    fn read_line(&mut self) -> Option<String> {
        let line = self.input.get(self.next_input)?.clone();
        self.next_input += 1;
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_console_collects_output() {
        let mut console = BufferedConsole::new();
        console.stdout(b"Hello, ");
        console.stdout(b"World!\n");
        console.stderr(b"warning\n");
        assert_eq!(console.stdout_text(), "Hello, World!\n");
        assert_eq!(console.stderr_text(), "warning\n");
    }

    #[test]
    fn buffered_console_serves_scripted_input() {
        let mut console = BufferedConsole::with_input(["first", "second"]);
        assert_eq!(console.read_line().as_deref(), Some("first"));
        assert_eq!(console.read_line().as_deref(), Some("second"));
        assert_eq!(console.read_line(), None);
    }
}
