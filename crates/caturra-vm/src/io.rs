//! Console IO for the running program.
//!
//! `System.out.println` and friends need somewhere to go, and `Scanner`
//! / `System.in` need somewhere to come from. The VM writes through the
//! [`ConsoleIo`] trait; the WASM boundary implements it by forwarding to
//! JavaScript callbacks, and tests use [`BufferedConsole`].

/// How a bounded wait for the window ended ([`ConsoleIo::ui_poll_event`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiPoll {
    /// The user did something: the event's payload.
    Event(String),
    /// The time given ran out with nothing from the window.
    TimedOut,
    /// The host ended the UI session (Stop, or there is no interactive host).
    Closed,
}

/// Host hooks for the running program's standard streams.
pub trait ConsoleIo {
    /// Milliseconds since the Unix epoch (`System.currentTimeMillis`).
    /// Hosts without a clock (tests) use a fixed origin.
    fn now_millis(&mut self) -> i64 {
        0
    }

    /// Seconds to add to UTC for the host's own zone, right now
    /// (`LocalDate.now()` has to know what "today" is, and that is a zone
    /// question). The browser answers from its own IANA data; a host without
    /// a zone — the tests, the CLI — is UTC, which is why a program that asks
    /// what today is cannot be compared against a JDK any more than one that
    /// asks for a random number can.
    fn zone_offset_seconds(&mut self) -> i32 {
        0
    }

    /// Wait up to `millis` milliseconds — what `Thread.sleep` asks of the
    /// host — and return how many actually passed. A host may return early
    /// (the VM asks again for the rest); it must not return MORE than it
    /// waited, or a sleep would end before its time.
    ///
    /// The browser worker really waits (`Atomics.wait` with a timeout). A
    /// host without a clock of its own passes the time instantly; one that
    /// answers `now_millis` should move that clock by what it returns, so a
    /// program that times its own sleep sees the pause it asked for — see
    /// [`BufferedConsole`], whose clock is VIRTUAL for exactly that reason.
    /// (See `specs/CONCURRENCY.md`, "The host contract".)
    fn wait_millis(&mut self, millis: u32) -> u32 {
        millis
    }

    /// Write bytes to standard out.
    fn stdout(&mut self, bytes: &[u8]);

    /// Write bytes to standard error.
    fn stderr(&mut self, bytes: &[u8]);

    /// Read one line from standard in, without the trailing newline.
    /// Returns `None` at end of input.
    fn read_line(&mut self) -> Option<String>;

    /// `Scanner.close()` on a `System.in` scanner closes the underlying
    /// stream, as the JDK's does: `read_line` then reports end of input
    /// forever, so a later `new Scanner(System.in)` reads nothing. A host
    /// with no closeable stdin may ignore this.
    fn close_stdin(&mut self) {}

    /// Swing event pump (see the bundled `__SwingRuntime`): present the
    /// current component tree (`tree`, a JSON string), then block until
    /// the next UI event and return its payload — the clicked component's
    /// id followed by newline-separated `id=value` field states. `None`
    /// ends the event loop (window closed, or no interactive host). Hosts
    /// without a UI (tests) use the default, which ends the loop at once.
    fn ui_await_event(&mut self, _tree: &str) -> Option<String> {
        None
    }

    /// The event-dispatch thread's wait (specs/CONCURRENCY.md, phase 3):
    /// present `tree`, then wait for the next UI event for at most
    /// `timeout_millis` — `None` for as long as it takes, `Some(0)` to only
    /// look. A host that answers [`ConsoleIo::now_millis`] from a clock of its
    /// own should move that clock by the time a [`UiPoll::TimedOut`] waited.
    /// The default cannot wait on time: it waits as [`ConsoleIo::ui_await_event`]
    /// does, whatever the timeout.
    fn ui_poll_event(&mut self, tree: &str, _timeout_millis: Option<u32>) -> UiPoll {
        match self.ui_await_event(tree) {
            Some(payload) => UiPoll::Event(payload),
            None => UiPoll::Closed,
        }
    }

    /// Show a modal dialog (bundled `JOptionPane`): `kind` is `message`,
    /// `confirm:<optionType>`, or `input`; `message` is its text. Blocks
    /// until the user answers and returns the response — a numeric option
    /// code, the entered text, or `None` if dismissed. Default: no host, so
    /// dismissed.
    fn ui_dialog(&mut self, _kind: &str, _message: &str) -> Option<String> {
        None
    }

    /// A dialog's BOUNDED wait (specs/CONCURRENCY.md, phase 3): show the dialog
    /// (once — a later call for the same dialog only waits again), then wait
    /// at most `timeout_millis` for the answer. [`UiPoll::Event`] is the
    /// answer, [`UiPoll::Closed`] a dismissal (`null`), [`UiPoll::TimedOut`]
    /// no answer yet. The default cannot wait on time: it waits as
    /// [`ConsoleIo::ui_dialog`] does.
    fn ui_poll_dialog(
        &mut self,
        kind: &str,
        message: &str,
        _timeout_millis: Option<u32>,
    ) -> UiPoll {
        match self.ui_dialog(kind, message) {
            Some(answer) => UiPoll::Event(answer),
            None => UiPoll::Closed,
        }
    }

    /// Start capturing standard-out messages (for `SystemOutTestRunner`,
    /// which runs the student's `main` and inspects what it printed). While
    /// capturing, standard out is redirected here (`System.setOut` semantics)
    /// — one message per `print`/`println` call, matching javabuilder's
    /// per-call `SYSTEM_OUT` messages. Default: no capture.
    fn begin_capture(&mut self) {}

    /// Whether a capture is active (a `print`/`println` should be recorded as
    /// a message rather than written out).
    fn capturing(&self) -> bool {
        false
    }

    /// Record one `print`/`println` call's text as a captured message.
    fn capture_message(&mut self, _text: &str) {}

    /// Stop capturing and return the recorded messages, in order.
    fn take_capture(&mut self) -> Vec<String> {
        Vec::new()
    }
}

/// An in-memory console: collects output and serves scripted input.
/// Used by tests and by batch (non-interactive) runs.
#[derive(Debug, Default)]
pub struct BufferedConsole {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    input: Vec<String>,
    next_input: usize,
    /// `Scanner.close()` closed standard in; reads report end of input.
    stdin_closed: bool,
    /// When `Some`, standard out is redirected into these per-call messages
    /// (`System.setOut` semantics for `SystemOutTestRunner`).
    capture: Option<Vec<String>>,
    /// A VIRTUAL clock, in milliseconds from zero: `Thread.sleep` advances it
    /// instantly and `currentTimeMillis`/`nanoTime` read it, so a sleeping
    /// program runs in no time and every timing it measures is exact — the
    /// same arrangement a fixed `Math.random` seed gives randomness.
    clock_millis: i64,
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
    fn now_millis(&mut self) -> i64 {
        self.clock_millis
    }

    fn wait_millis(&mut self, millis: u32) -> u32 {
        self.clock_millis += i64::from(millis);
        millis
    }

    fn stdout(&mut self, bytes: &[u8]) {
        self.stdout.extend_from_slice(bytes);
    }

    fn stderr(&mut self, bytes: &[u8]) {
        self.stderr.extend_from_slice(bytes);
    }

    fn begin_capture(&mut self) {
        self.capture = Some(Vec::new());
    }

    fn capturing(&self) -> bool {
        self.capture.is_some()
    }

    fn capture_message(&mut self, text: &str) {
        if let Some(messages) = &mut self.capture {
            messages.push(text.to_owned());
        }
    }

    fn take_capture(&mut self) -> Vec<String> {
        self.capture.take().unwrap_or_default()
    }

    fn read_line(&mut self) -> Option<String> {
        if self.stdin_closed {
            return None;
        }
        let line = self.input.get(self.next_input)?.clone();
        self.next_input += 1;
        Some(line)
    }

    fn close_stdin(&mut self) {
        self.stdin_closed = true;
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
