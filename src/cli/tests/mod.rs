//! CLI tests module (manifest).
//!
//! Split by concern under `src/cli/tests/*` to keep files small and localize
//! global env/CWD mutation patterns.

// Allow unused imports and clippy lints for test code
#![allow(clippy::items_after_test_module)]
#![allow(clippy::await_holding_lock)]

// Test support utilities
mod support;

// Test modules by command/feature
mod benchmark;
mod default_config;
mod project;
mod resume_json;
mod spec_exec;
mod spec_json;
mod status;
mod status_json;
