//! Standard library usage scanner.
//!
//! This check scans Rust source files for dangerous `std::` usage outside of
//! `#[cfg(test)]` code blocks. Soroban contracts typically need `#![no_std]`,
//! but test code can safely use the standard library.
//!
//! The scanner:
//! 1. Discovers all `.rs` files in the project
//! 2. Skips files with `#![cfg(test)]` at the top
//! 3. Parses each file with `syn`
//! 4. Walks the AST looking for dangerous std usage
//! 5. Reports findings with exact line/column locations

use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use walkdir::WalkDir;

use crate::checks::Check;
use crate::context::Context;
use crate::error::Error;
use crate::report::{Finding, Severity};

const CHECK_ID: &str = "std_scan.disallowed_std_usage";

/// `std` paths that are not allowed outside of test code, expressed as
/// segment lists. A usage matches if its segments start with one of these
/// (so `std::fs::read_to_string` matches the `std::fs` entry).
const DANGEROUS_PATHS: &[&[&str]] = &[
    &["std", "collections", "HashMap"],
    &["std", "collections", "HashSet"],
    &["std", "fs"],
    &["std", "net"],
    &["std", "thread"],
    &["std", "process"],
];

/// Macro names that are not allowed outside of test code.
const DANGEROUS_MACROS: &[&str] = &["println", "print", "eprintln"];

/// File names skipped during discovery regardless of their content.
/// `build.rs` is a Cargo build script, not contract code, and commonly
/// uses std:: freely and legitimately.
const SKIPPED_FILE_NAMES: &[&str] = &["build.rs"];

/// Scans project source files for disallowed `std::` usage outside of
/// `#[cfg(test)]` code.
pub struct StdScanCheck;

impl Check for StdScanCheck {
    fn name(&self) -> &'static str {
        "std_scan"
    }

    fn run(&self, ctx: &Context) -> Result<Vec<Finding>, Error> {
        let mut findings = Vec::new();
        for path in discover_rust_files(ctx.project_root()) {
            scan_file(&path, &mut findings);
        }
        Ok(findings)
    }
}

/// Find all `.rs` files under `root`, skipping `target/` and hidden
/// directories (but never skipping `root` itself, even if its own name
/// looks hidden, e.g. `.`), and skipping files in `SKIPPED_FILE_NAMES`
/// (e.g. `build.rs`) wherever they appear.
fn discover_rust_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !is_skipped_dir(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .filter(|entry| !is_skipped_file(entry))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn is_skipped_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name == "target" || name.starts_with('.')
}

fn is_skipped_file(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    SKIPPED_FILE_NAMES.contains(&name.as_ref())
}

/// Read and scan a single file, appending any findings (or a parse/read
/// warning) to `findings`. One unreadable/unparseable file must not stop
/// the rest of the scan.
fn scan_file(path: &Path, findings: &mut Vec<Finding>) {
    match fs::read_to_string(path) {
        Ok(content) => findings.extend(scan_source(path, &content)),
        Err(err) => findings.push(Finding::new(
            CHECK_ID,
            Severity::Warning,
            format!("could not read {}: {err}", path.display()),
        )),
    }
}

/// Parse `content` (the contents of `file`) and return findings for any
/// disallowed `std::` usage outside `#[cfg(test)]` code.
///
/// A parse failure produces a single `Warning` finding rather than an
/// `Error`, since a syntax error in one file is a hygiene issue, not
/// itself dangerous std usage.
///
/// If the file carries a crate-level `#![cfg(test)]` inner attribute, the
/// whole file is treated as test-only and skipped entirely (no findings),
/// the same way an item-level `#[cfg(test)]` module is skipped.
pub fn scan_source(file: &Path, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    match syn::parse_file(content) {
        Ok(parsed) => {
            if is_cfg_test(&parsed.attrs) {
                return findings;
            }

            let mut visitor = StdUsageVisitor {
                file,
                findings: &mut findings,
            };
            for item in &parsed.items {
                visitor.visit_item(item);
            }
        }
        Err(err) => findings.push(Finding::new(
            CHECK_ID,
            Severity::Warning,
            format!("could not parse {}: {err}", file.display()),
        )),
    }

    findings
}

/// Walks the AST of a single file, skipping any subtree attributed with
/// `#[cfg(test)]` and flagging disallowed `std::` paths, `use` imports,
/// and macros everywhere else.
struct StdUsageVisitor<'a> {
    file: &'a Path,
    findings: &'a mut Vec<Finding>,
}

impl<'a> StdUsageVisitor<'a> {
    fn push_finding(&mut self, subject: impl Into<String>, span: proc_macro2::Span) {
        let start = span.start();
        let finding = Finding::new(
            CHECK_ID,
            Severity::Error,
            format!("disallowed `{}` usage", subject.into()),
        )
        .with_location(self.file.to_path_buf(), start.line, start.column + 1);
        self.findings.push(finding);
    }

    /// Recursively walk a `use` tree, tracking the path segments seen so
    /// far, and flag any leaf whose full path matches a dangerous path.
    fn check_use_tree(&mut self, tree: &syn::UseTree, mut prefix: Vec<String>) {
        match tree {
            syn::UseTree::Path(p) => {
                prefix.push(p.ident.to_string());
                self.check_use_tree(&p.tree, prefix);
            }
            syn::UseTree::Name(n) => {
                let mut full = prefix;
                full.push(n.ident.to_string());
                if let Some(name) = matches_dangerous(&full) {
                    self.push_finding(name, n.ident.span());
                }
            }
            syn::UseTree::Rename(r) => {
                // Flag the *original* name (e.g. `HashMap`), not the alias,
                // so `use std::collections::HashMap as Map;` is still caught.
                let mut full = prefix;
                full.push(r.ident.to_string());
                if let Some(name) = matches_dangerous(&full) {
                    self.push_finding(name, r.ident.span());
                }
            }
            syn::UseTree::Glob(g) => {
                if let Some(name) = matches_dangerous(&prefix) {
                    self.push_finding(name, g.span());
                }
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.check_use_tree(item, prefix.clone());
                }
            }
        }
    }

    fn check_path(&mut self, path: &syn::Path) {
        let Some(first) = path.segments.first() else {
            return;
        };
        if first.ident != "std" {
            return;
        }
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        if let Some(name) = matches_dangerous(&segments) {
            self.push_finding(name, first.ident.span());
        }
    }

    fn check_macro(&mut self, mac: &syn::Macro) {
        let Some(last) = mac.path.segments.last() else {
            return;
        };
        let name = last.ident.to_string();
        if DANGEROUS_MACROS.contains(&name.as_str()) {
            self.push_finding(format!("{name}!"), last.ident.span());
        }
    }
}

impl<'a, 'ast> Visit<'ast> for StdUsageVisitor<'a> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if is_cfg_test(item_attrs(item)) {
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        if is_cfg_test(impl_item_attrs(item)) {
            return;
        }
        visit::visit_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        if is_cfg_test(trait_item_attrs(item)) {
            return;
        }
        visit::visit_trait_item(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.check_use_tree(&item.tree, Vec::new());
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.check_path(path);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.check_macro(mac);
        visit::visit_macro(self, mac);
    }
}

/// True if `attrs` contains an exact `#[cfg(test)]` (or, for a file's
/// inner attributes, `#![cfg(test)]`).
///
/// Phase 1 limitation: compound predicates such as `#[cfg(any(test, ...))]`
/// are not recognized as test-only and will still be scanned.
fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

/// True if `segments` matches or is more specific than one of the
/// `DANGEROUS_PATHS` entries, returning the dangerous path's display name.
fn matches_dangerous(segments: &[String]) -> Option<String> {
    DANGEROUS_PATHS
        .iter()
        .find(|dangerous| {
            segments.len() >= dangerous.len()
                && segments.iter().zip(dangerous.iter()).all(|(a, b)| a == b)
        })
        .map(|dangerous| dangerous.join("::"))
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    use syn::Item::*;
    match item {
        Const(i) => &i.attrs,
        Enum(i) => &i.attrs,
        ExternCrate(i) => &i.attrs,
        Fn(i) => &i.attrs,
        ForeignMod(i) => &i.attrs,
        Impl(i) => &i.attrs,
        Macro(i) => &i.attrs,
        Mod(i) => &i.attrs,
        Static(i) => &i.attrs,
        Struct(i) => &i.attrs,
        Trait(i) => &i.attrs,
        TraitAlias(i) => &i.attrs,
        Type(i) => &i.attrs,
        Union(i) => &i.attrs,
        Use(i) => &i.attrs,
        _ => &[],
    }
}

fn impl_item_attrs(item: &syn::ImplItem) -> &[syn::Attribute] {
    use syn::ImplItem::*;
    match item {
        Const(i) => &i.attrs,
        Fn(i) => &i.attrs,
        Type(i) => &i.attrs,
        Macro(i) => &i.attrs,
        _ => &[],
    }
}

fn trait_item_attrs(item: &syn::TraitItem) -> &[syn::Attribute] {
    use syn::TraitItem::*;
    match item {
        Const(i) => &i.attrs,
        Fn(i) => &i.attrs,
        Type(i) => &i.attrs,
        Macro(i) => &i.attrs,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// discover_rust_files must skip build.rs while still picking up
    /// ordinary .rs files, including ones that sit alongside it.
    #[test]
    fn discovery_skips_build_rs() {
        let dir = unique_temp_dir("sorocheck_discovery_skips_build_rs");
        fs::create_dir_all(dir.join("src")).expect("create src dir");

        fs::write(dir.join("build.rs"), "fn main() {}").expect("write build.rs");
        fs::write(dir.join("src").join("lib.rs"), "fn main() {}").expect("write lib.rs");

        let found = discover_rust_files(&dir);

        assert!(
            found.iter().all(|p| p.file_name().unwrap() != "build.rs"),
            "build.rs should never be discovered: {found:?}"
        );
        assert!(
            found.iter().any(|p| p.file_name().unwrap() == "lib.rs"),
            "lib.rs should still be discovered: {found:?}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{label}_{}_{nanos}", std::process::id()));
        fs::create_dir_all(&dir).expect("create unique temp dir");
        dir
    }
}