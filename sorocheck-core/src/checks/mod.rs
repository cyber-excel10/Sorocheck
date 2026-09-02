// this checks implementations for sorocheck.
//
// Each check implements the `Check` trait and scans for specific issues.
// New checks can be added by creating a new module and implementing the trait.

use crate::context::Context;
use crate::error::Error;
use crate::report::Finding;

pub mod toolchain;
pub mod std_scan;

/// Trait for implementing new checks.
///
/// To add a check:
/// 1. Create a new module in this directory
/// 2. Implement this trait for your check struct
/// 3. Add your check to `run_all_checks()` in lib.rs
pub trait Check {
    /// Unique identifier for this check
    fn name(&self) -> &'static str;

    /// Run the check against a project context
    fn run(&self, ctx: &Context) -> Result<Vec<Finding>, Error>;
}