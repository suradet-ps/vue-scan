//! End-to-end integration tests.
//!
//! Tests are split into focused submodules so a failure points at the
//! layer that regressed:
//!
//! * `common`  - a thin fluent wrapper around the compiled `vuer` binary
//!   so each test reads as a one-liner: `vuer().input(...).format("json")
//!   .run()`.
//! * `audit`   - per-rule regression tests. Each rule has at least one
//!   fixture that should fire it and at least one that should not.
//! * `snapshots` - byte-exact output tests for the JSON, SARIF, and pretty
//!   output formats. Adding a new output field will diff against the
//!   committed snapshot, so the change is intentional.

mod audit;
mod common;
mod snapshots;
