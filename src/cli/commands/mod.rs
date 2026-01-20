//! CLI command implementations (facade).
//!
//! This module re-exports the stable command surface used by `run.rs` and CLI tests.
//! Implementations live in `commands/*`.

// Allow unused imports for public API surface - these are intentionally exported
// for external use and tests, even if not used within the CLI module itself.
#![allow(unused_imports)]

mod benchmark;
mod clean;
mod common;
mod doctor;
mod gate;
mod init;
mod json_emit;
mod project;
mod resume;
mod spec;
mod status;
mod template;
mod test_cmd;

// Re-export command handlers
pub use benchmark::execute_benchmark_command;
pub use clean::execute_clean_command;
pub use doctor::execute_doctor_command;
pub use gate::execute_gate_command;
pub use init::execute_init_command;
pub use project::{
    derive_spec_status, execute_project_command, execute_project_history_command,
    execute_project_status_command, execute_project_tui_command,
};
pub use resume::{execute_resume_command, execute_resume_json_command};
pub use spec::{execute_spec_command, execute_spec_json_command};
pub use status::{check_and_display_fixup_targets, execute_status_command};
pub use template::execute_template_command;
pub use test_cmd::execute_test_command;

// Re-export common helpers
pub use common::{
    build_orchestrator_config, check_lockfile_drift, count_pending_fixups,
    count_pending_fixups_for_spec, create_default_config, detect_claude_cli_version,
    generate_next_steps_hint,
};

// Re-export JSON emit functions
pub use json_emit::{
    emit_resume_json, emit_spec_json, emit_status_json, emit_workspace_history_json,
    emit_workspace_status_json,
};
