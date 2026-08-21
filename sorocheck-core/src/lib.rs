//! sorocheck-core
//!
//! All real logic for sorocheck lives here. The `sorocheck` CLI crate is a
//! thin wrapper that calls [`check_project`] and prints the result.

pub mod checks;
pub mod context;
pub mod error;
pub mod report;

use std::path::Path;

use checks::std_scan::StdScanCheck;
use checks::toolchain::ToolchainCheck;
use checks::Check;
use context::Context;

pub use error::Error;
pub use report::{Finding, Report};

/// Run all checks against the project rooted at `path` and return a report.
///
/// This is the single public entry point into sorocheck-core. The CLI
/// (and any other consumer) should not need to call anything else.
pub fn check_project(path: &Path) -> Result<Report, Error> {
    let ctx = Context::new(path);
    let findings = run_all_checks(&ctx)?;

    let mut report = Report::new();
    for finding in findings {
        report.push(finding);
    }
    Ok(report)
}

/// Run every registered check against `ctx` and return their combined
/// findings. Adding a new check later means adding one entry here.
fn run_all_checks(ctx: &Context) -> Result<Vec<Finding>, Error> {
    let checks: Vec<Box<dyn Check>> = vec![Box::new(ToolchainCheck), Box::new(StdScanCheck)];

    let mut findings = Vec::new();
    for check in &checks {
        findings.extend(check.run(ctx)?);
    }
    Ok(findings)
}