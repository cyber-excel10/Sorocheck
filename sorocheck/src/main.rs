//! sorocheck CLI.
//!
//! Thin wrapper only: parse arguments, call sorocheck_core, print results,
//! set exit code. No check logic lives here.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context as _;
use clap::{Parser, ValueEnum};

use sorocheck_core::report::Severity;
use sorocheck_core::{check_project, Finding, Report};

/// Pre-build checker for Soroban smart contracts.
#[derive(Debug, Parser)]
#[command(name = "sorocheck", version, about)]
struct Cli {
    /// Path to the project to check.
    #[arg(default_value = ".")]
    path: PathBuf,
    
    /// Only print findings with Error severity.
    /// Warnings and info still counted in summary footer.
    /// Ignored when --format json is used.
    #[arg(long)]
    errors_only: bool,
    
    /// Output format for the report.
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

/// Supported `--format` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Human-readable findings list plus a summary footer (default).
    Text,
    /// The full report, unfiltered, as JSON on stdout.
    Json,
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let report = check_project(&cli.path)
        .with_context(|| format!("failed to check project at {}", cli.path.display()))?;

    match cli.format {
        OutputFormat::Text => print_report(&report, cli.errors_only),
        OutputFormat::Json => print_json(&report)?,
    }

    if report.has_errors() {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print findings, then summary footer with counts.
/// If errors_only is true, only show errors in list.
fn print_report(report: &Report, errors_only: bool) {
    if report.is_clean() {
        println!("No findings. Project looks good.");
        return;
    }

    for finding in findings_to_print(report, errors_only) {
        print_finding(finding);
    }

    println!();
    print_summary(report);
}

/// Select findings to list based on --errors-only flag.
fn findings_to_print(report: &Report, errors_only: bool) -> impl Iterator<Item = &Finding> {
    report.findings.iter().filter(move |f| !errors_only || f.severity == Severity::Error)
}

fn print_finding(finding: &Finding) {
    let severity = severity_label(finding.severity);

    match location_label(finding) {
        Some(location) => println!(
            "[{severity}] {location} {}: {}",
            finding.check_id, finding.message
        ),
        None => println!("[{severity}] {}: {}", finding.check_id, finding.message),
    }

    if let Some(note) = &finding.note {
        println!("    note: {note}");
    }
}

/// Show location as file:line:col if available.
fn location_label(finding: &Finding) -> Option<String> {
    let file = finding.file.as_ref()?;
    let line = finding.line?;
    let column = finding.column?;
    Some(format!("{}:{line}:{column}", file.display()))
}

/// One-line footer with error/warning/info counts.
fn print_summary(report: &Report) {
    let errors = report.count(Severity::Error);
    let warnings = report.count(Severity::Warning);
    let infos = report.count(Severity::Info);

    println!("Summary: {errors} error(s), {warnings} warning(s), {infos} info");
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "INFO",
        Severity::Warning => "WARN",
        Severity::Error => "ERROR",
    }
}

/// Serialize the full report as pretty-printed JSON.
/// --errors-only has no effect here (JSON always includes all findings).
fn print_json(report: &Report) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(report)
        .context("failed to serialize report as JSON")?;
    println!("{json}");
    Ok(())
}