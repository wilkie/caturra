//! Compiler diagnostics: errors and warnings with source locations,
//! structured so the TypeScript layer can render them in an editor
//! (squiggles, gutter markers) rather than parsing text output.

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A position in a source file. Lines and columns are 1-based, matching
/// what editors display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

/// A half-open range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

/// One compiler message tied to a source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    /// Path of the source file the diagnostic refers to.
    pub path: String,
    /// Location within the file, when one is known.
    pub span: Option<SourceSpan>,
}

impl Diagnostic {
    /// An error diagnostic at a specific span.
    #[must_use]
    pub fn error(path: &str, message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            path: path.to_owned(),
            span: Some(span),
        }
    }
}
