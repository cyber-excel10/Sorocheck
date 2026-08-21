# sorocheck  

Pre-build checker for **Soroban (Stellar) smart contracts**.

`sorocheck` catches common environment and restricted-Rust mistakes **before** you waste time on confusing build failures.

> This is a focused developer tool for Soroban contract crates — not a general Rust linter and not a full security audit tool.

## What it checks

1. **Toolchain**
   - Rust version (requires 1.84+)
   - `wasm32v1-none` target installed
   - `stellar` CLI availability

2. **Restricted environment (`std` usage)**
   - Flags dangerous `std::` usage outside test code
   - Ignores item-level `#[cfg(test)]`
   - Ignores whole files with `#![cfg(test)]`
   - Skips `build.rs` by default

## Quick start

```bash
# From this repo
cargo install --path sorocheck

# Check a Soroban contract crate
sorocheck /path/to/contracts

# Check current directory
sorocheck .
```

**Tip:** point it at the contract package (for example `./contracts`), not a full monorepo with frontend/backend apps.

## Example output

```
[INFO] toolchain.rust_version: Rust version 1.97.1 meets the minimum required (1.84.0)
[INFO] toolchain.wasm32v1_none_target: Target `wasm32v1-none` is installed
[INFO] toolchain.stellar_cli: `stellar` CLI is available
[WARN] std_scan.disallowed_std_usage: could not parse src/test.rs: cannot parse string into token stream
```

- **Exit code 0** → no error-level findings
- **Exit code 1** → one or more error-level findings

## Current limitations (Phase 1)

- Designed for Soroban contract code, not general Rust CLIs/apps
- Only exact `cfg(test)` is treated as test-only (not full `cfg(any(...))` support yet)
- Output is simple text (JSON/CI helpers come later)

## Project structure

```
sorocheck/
├── sorocheck-core/   # library: checks + report logic
└── sorocheck/        # thin CLI
```

## Development

```bash
cargo test -p sorocheck-core
cargo run -p sorocheck -- ./path/to/contract
```

### Adding a new check

1. Add a module under `sorocheck-core/src/checks/`
2. Implement the `Check` trait
3. Register it in `run_all_checks()` in `sorocheck-core/src/lib.rs`
4. Add fixtures/tests

## License

MIT