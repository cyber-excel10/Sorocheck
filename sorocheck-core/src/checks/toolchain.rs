//! Toolchain compatibility checks.
//!
//! Verifies that the development environment is properly set up for
//! Soroban contract development.

use std::process::Command;

use semver::Version;

use crate::checks::Check;
use crate::context::Context;
use crate::error::Error;
use crate::report::{Finding, Severity};

/// Check IDs used in reports
const CHECK_ID_RUST_VERSION: &str = "toolchain.rust_version";
const CHECK_ID_WASM_TARGET: &str = "toolchain.wasm32v1_none_target";
const CHECK_ID_STELLAR_CLI: &str = "toolchain.stellar_cli";

/// WebAssembly target required for Soroban contracts
const REQUIRED_TARGET: &str = "wasm32v1-none";

/// Minimum Rust version required
fn min_rust_version() -> Version {
    Version::new(1, 84, 0)
}

/// Checks the local Rust/Soroban toolchain for compatibility.
///
/// Verifies:
/// 1. Rust version is >= 1.84.0
/// 2. `wasm32v1-none` target is installed
/// 3. `stellar` CLI is available (optional, warning only)
pub struct ToolchainCheck;

impl Check for ToolchainCheck {
    fn name(&self) -> &'static str {
        "toolchain"
    }

    fn run(&self, _ctx: &Context) -> Result<Vec<Finding>, Error> {
        Ok(vec![
            check_rust_version(),
            check_wasm_target(),
            check_stellar_cli(),
        ])
    }
}

/// Verify `rustc` is present and its version is >= the minimum required.
fn check_rust_version() -> Finding {
    let min = min_rust_version();

    match detect_rust_version() {
        Some(version) if version >= min => Finding::new(
            CHECK_ID_RUST_VERSION,
            Severity::Info,
            format!("Rust version {version} meets the minimum required ({min})"),
        ),
        Some(version) => Finding::new(
            CHECK_ID_RUST_VERSION,
            Severity::Error,
            format!("Rust version {version} is below the minimum required ({min})"),
        )
        .with_note("Run `rustup update` to install a newer toolchain."),
        None => Finding::new(
            CHECK_ID_RUST_VERSION,
            Severity::Error,
            "Could not determine the installed Rust version",
        )
        .with_note("Ensure `rustc` is installed and available on PATH."),
    }
}

/// Run `rustc --version` and parse the semver from its output.
fn detect_rust_version() -> Option<Version> {
    let output = Command::new("rustc").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_rustc_version(&stdout)
}

/// Parse the version number out of `rustc --version` output, e.g.
/// "rustc 1.84.0 (9fc6b4312 2025-01-07)" -> 1.84.0
fn parse_rustc_version(stdout: &str) -> Option<Version> {
    let version_str = stdout.split_whitespace().nth(1)?;
    Version::parse(version_str).ok()
}

/// Verify the `wasm32v1-none` target is installed via rustup.
fn check_wasm_target() -> Finding {
    match wasm_target_installed() {
        Some(true) => Finding::new(
            CHECK_ID_WASM_TARGET,
            Severity::Info,
            format!("Target `{REQUIRED_TARGET}` is installed"),
        ),
        Some(false) => Finding::new(
            CHECK_ID_WASM_TARGET,
            Severity::Error,
            format!("Target `{REQUIRED_TARGET}` is not installed"),
        )
        .with_note(format!("Run `rustup target add {REQUIRED_TARGET}`.")),
        None => Finding::new(
            CHECK_ID_WASM_TARGET,
            Severity::Error,
            format!("Could not determine whether target `{REQUIRED_TARGET}` is installed"),
        )
        .with_note("Ensure `rustup` is installed and available on PATH."),
    }
}

/// Run `rustup target list --installed` and check for the required target.
fn wasm_target_installed() -> Option<bool> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().any(|line| line.trim() == REQUIRED_TARGET))
}

/// Check whether the `stellar` CLI is reachable on PATH. Missing CLI is a
/// warning, not an error, since it is not required to build a contract.
fn check_stellar_cli() -> Finding {
    if stellar_cli_available() {
        Finding::new(
            CHECK_ID_STELLAR_CLI,
            Severity::Info,
            "`stellar` CLI is available",
        )
    } else {
        Finding::new(
            CHECK_ID_STELLAR_CLI,
            Severity::Warning,
            "`stellar` CLI was not found on PATH",
        )
        .with_note("Install it with `cargo install --locked stellar-cli`.")
    }
}

fn stellar_cli_available() -> bool {
    Command::new("stellar")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_rustc_version_output() {
        let stdout = "rustc 1.84.0 (9fc6b4312 2025-01-07)\n";
        let version = parse_rustc_version(stdout).expect("should parse");
        assert_eq!(version, Version::new(1, 84, 0));
    }

    #[test]
    fn rejects_malformed_rustc_version_output() {
        assert!(parse_rustc_version("not a version string").is_none());
    }
}