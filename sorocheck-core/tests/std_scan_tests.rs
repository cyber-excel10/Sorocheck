//! Integration tests for the cfg-aware `std::` scanner.
//!
//! These call `sorocheck_core::checks::std_scan::scan_source` directly on
//! each fixture's contents, rather than `check_project`, so each fixture's
//! findings can be asserted in isolation (running `check_project` on the
//! whole `fixtures/` directory would mix all three files' findings
//! together).

use std::fs;
use std::path::{Path, PathBuf};

use sorocheck_core::checks::std_scan::scan_source;
use sorocheck_core::report::Severity;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    let path = fixture_path(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("failed to read fixture {}: {}", path.display(), e);
    })
}

/// Count findings at Error severity (i.e. actual disallowed std:: usage,
/// as opposed to a Warning-level parse/read failure).
fn std_usage_findings(name: &str) -> usize {
    let path = fixture_path(name);
    let content = read_fixture(name);
    scan_source(&path, &content)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .count()
}

/// plain_contract.rs has no std:: usage at all, so the scanner must report
/// zero findings.
#[test]
fn plain_contract_has_no_std_findings() {
    assert_eq!(std_usage_findings("plain_contract.rs"), 0);
}

/// with_test_module.rs uses std::collections::HashMap, but only inside a
/// #[cfg(test)] module, so the scanner must ignore it entirely.
#[test]
fn test_module_std_usage_is_ignored() {
    assert_eq!(std_usage_findings("with_test_module.rs"), 0);
}

/// with_alias.rs imports std::collections::HashMap under an alias (Map),
/// outside of any cfg(test) code, so the scanner must still catch it.
#[test]
fn alias_case_is_detected() {
    assert!(std_usage_findings("with_alias.rs") >= 1);
}

#[test]
fn whole_file_cfg_test_attribute_is_ignored() {
    assert_eq!(std_usage_findings("whole_file_cfg_test.rs"), 0);
}

/// whole_file_cfg_test.rs has `#![cfg(test)]` at the crate level, so the
/// whole file is test-only and must be ignored entirely, even though it
/// contains std::collections::HashMap and println! usage.
#[test]
fn whole_file_cfg_test_is_ignored() {
    assert_eq!(std_usage_findings("whole_file_cfg_test.rs"), 0);
}