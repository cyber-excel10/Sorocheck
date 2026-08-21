use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context as _;
use clap::Parser;

use sorocheck_core::report::Severity;
use sorocheck_core::{check_project, Finding, Report};

/// Command-line interface for sorocheck.
///
/// Usage: `sorocheck [PATH]`
/// - If PATH is not provided, checks current directory
/// - Returns exit code 0 if no errors found
/// - Returns exit code 1 if any errors found
#[derive(Debug, Parser)]
#[command(name = "sorocheck", version, about)]
struct Cli {
    // Path to the Soroban contract project to check
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let report = check_project(&cli.path)
        .with_context(|| format!("failed to check project at {}", cli.path.display()))?;

    print_report(&report);

    if report.has_errors() {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print each finding in a simple, readable format.
fn print_report(report: &Report) {
    if report.is_clean() {
        println!("No findings. Project looks good.");
        return;
    }

    for finding in &report.findings {
        print_finding(finding);
    }
}

fn print_finding(finding: &Finding) {
    let severity = severity_label(finding.severity);
    println!("[{severity}] {}: {}", finding.check_id, finding.message);

    if let Some(note) = &finding.note {
        println!("    note: {note}");
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "INFO",
        Severity::Warning => "WARN",
        Severity::Error => "ERROR",
    }
}