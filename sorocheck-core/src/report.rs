//! Structured reporting types produced by checks and returned to callers.

use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

/// How serious a [`Finding`] is.
///
/// Ordered from least to most severe; derives `PartialOrd`/`Ord` so
/// findings can be sorted or filtered by severity threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational only; does not indicate a problem.
    Info,
    /// Worth reviewing, but not necessarily incorrect.
    Warning,
    /// A real problem that should block a Soroban build/deploy.
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

/// A single issue or informational result produced by a check.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable identifier for the check that produced this finding
    /// (e.g. "toolchain.rust_version", "std_scan.disallowed_import").
    pub check_id: &'static str,
    /// How serious this finding is.
    pub severity: Severity,
    /// Short, human-readable summary of the finding.
    pub message: String,
    /// File this finding relates to, if any (e.g. not applicable for
    /// toolchain checks, but always present for std_scan findings).
    pub file: Option<PathBuf>,
    /// 1-based line number within `file`, if known.
    pub line: Option<usize>,
    /// 1-based column number within `line`, if known.
    pub column: Option<usize>,
    /// Optional extra detail / suggestion for fixing the issue.
    pub note: Option<String>,
}

impl Finding {
    /// Construct a minimal finding with no source location.
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

    /// Attach a source file location to this finding.
    pub fn with_location(mut self, file: PathBuf, line: usize, column: usize) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Attach an explanatory note / suggestion to this finding.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// The aggregate result of running all checks against a project.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    /// All findings produced across all checks, in the order they were run.
    pub findings: Vec<Finding>,
}

impl Report {
    /// Create an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a finding to the report.
    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// True if any finding has [`Severity::Error`].
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    /// True if there are no findings at all.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// Iterate over findings at or above a given severity.
    pub fn findings_at_least(&self, min: Severity) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |f| f.severity >= min)
    }

    /// Count findings at exactly the given severity (not "at least").
    /// Used to build the CLI's error/warning/info summary footer.
    pub fn count(&self, severity: Severity) -> usize {
        self.findings.iter().filter(|f| f.severity == severity).count()
    }
}