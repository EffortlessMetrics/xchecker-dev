//! JSON emission for gate results
//!
//! This module provides functions for emitting gate evaluation results as JSON.

use anyhow::Context;
use xchecker_utils::canonicalization::emit_jcs;

use crate::types::GateResult;

/// Emit gate result as canonical JSON using JCS (RFC 8785)
///
/// This function emits gate evaluation result in canonical JSON form
/// for deterministic output and stable diffs.
pub fn emit_gate_json(result: &GateResult) -> anyhow::Result<String> {
    emit_jcs(result).context("Failed to emit gate JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GateCondition, GateResult};

    #[test]
    fn test_emit_gate_json() {
        let result = GateResult {
            passed: true,
            summary: "Spec passed all checks".to_string(),
            conditions: vec![GateCondition {
                name: "Test condition".to_string(),
                description: "A test condition".to_string(),
                passed: true,
                actual: Some("actual".to_string()),
                expected: Some("expected".to_string()),
            }],
            failure_reasons: vec![],
        };

        let json = emit_gate_json(&result);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["passed"], true);
        assert_eq!(parsed["summary"], "Spec passed all checks");
        assert_eq!(parsed["conditions"].as_array().unwrap().len(), 1);
        assert!(parsed["failure_reasons"].as_array().unwrap().is_empty());
    }
}
