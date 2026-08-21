//! Report structures for sorocheck findings.
//!
//! Contains the data structures used to report issues found during checking.

use std::fmt;
use std::path::PathBuf;

/// Severity level of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Informational message (not a problem)
    Info,
    /// Warning (potential issue)
    Warning,
    /// Error (definite problem that should be fixed)
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "{s}")
    }
}

/// A single issue found by a check.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Which check found this issue (e.g., "toolchain.rust_version")
    pub check_id: &'static str,
    /// How serious this issue is
    pub severity: Severity,
    /// Human-readable description of the issue
    pub message: String,
    /// File where the issue was found (if applicable)
    pub file: Option<PathBuf>,
    /// Line number in the file (if applicable)
    pub line: Option<usize>,
    /// Column number in the file (if applicable)
    pub column: Option<usize>,
    /// Additional helpful information or fix suggestions
    pub note: Option<String>,
}

impl Finding {
    pub fn new(check_id: &'static str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            check_id,
            severity,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            note: None,
        }
    }

    pub fn with_location(mut self, file: impl Into<PathBuf>, line: usize, column: usize) -> Self {
    self.file = Some(file.into());
    self.line = Some(line);
    self.column = Some(column);
    self
}

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn findings_at_least(&self, min: Severity) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |f| f.severity >= min)
    }
}