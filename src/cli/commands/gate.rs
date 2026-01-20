//! Gate command implementation
//!
//! Handles `xchecker gate` command for policy-based spec validation.

use anyhow::{Context, Result};

use crate::error::ConfigError;
use crate::XCheckerError;

/// Execute the gate command for policy-based spec validation
/// Per FR-GATE (Requirements 4.5.1, 4.5.2, 4.5.3, 4.5.4)
pub fn execute_gate_command(
    spec_id: &str,
    min_phase: &str,
    fail_on_pending_fixups: bool,
    max_phase_age: Option<&str>,
    json: bool,
) -> Result<()> {
    use crate::gate::{emit_gate_json, parse_duration, parse_phase, GateCommand, GatePolicy};

    // Parse min_phase
    let min_phase_id = parse_phase(min_phase).map_err(|e| {
        XCheckerError::Config(ConfigError::InvalidValue {
            key: "min_phase".to_string(),
            value: e.to_string(),
        })
    })?;

    // Parse max_phase_age if provided
    let max_age = if let Some(age_str) = max_phase_age {
        Some(parse_duration(age_str).map_err(|e| {
            XCheckerError::Config(ConfigError::InvalidValue {
                key: "max_phase_age".to_string(),
                value: e.to_string(),
            })
        })?)
    } else {
        None
    };

    // Build policy
    let policy = GatePolicy {
        min_phase: min_phase_id,
        fail_on_pending_fixups,
        max_phase_age: max_age,
    };

    // Execute gate evaluation
    let gate = GateCommand::new(spec_id.to_string(), policy);
    let result = gate
        .execute()
        .with_context(|| format!("Failed to evaluate gate for spec: {spec_id}"))?;

    // Output results
    if json {
        let json_output = emit_gate_json(&result).with_context(|| "Failed to emit gate JSON")?;
        println!("{json_output}");
    } else {
        // Human-friendly output
        if result.passed {
            println!("✓ {}", result.summary);
        } else {
            println!("✗ {}", result.summary);
        }

        println!();
        println!("Conditions evaluated:");
        for condition in &result.conditions {
            let status = if condition.passed { "✓" } else { "✗" };
            println!("  {} {}: {}", status, condition.name, condition.description);
            if let Some(actual) = &condition.actual {
                println!("      Actual: {}", actual);
            }
            if let Some(expected) = &condition.expected {
                println!("      Expected: {}", expected);
            }
        }

        if !result.failure_reasons.is_empty() {
            println!();
            println!("Failure reasons:");
            for reason in &result.failure_reasons {
                println!("  - {}", reason);
            }
        }
    }

    // Exit with appropriate code
    if result.passed {
        Ok(())
    } else {
        std::process::exit(crate::gate::exit_codes::POLICY_VIOLATION);
    }
}
