//! Minimal no_std contract-shaped crate used as a clean path for sorocheck CI.
#![no_std]

/// Placeholder entry used only so the crate has a public API surface.
pub fn ping() -> u32 {
    1
}
