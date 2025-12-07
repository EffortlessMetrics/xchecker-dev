//! Schema versioning and backward compatibility tests
//!
//! These tests ensure that v1 schemas remain stable and backward compatible:
//! - Required fields are never removed (breaking change for consumers)
//! - New required fields are never added (breaks forward compatibility)
//! - Unknown fields are always ignored (additionalProperties: true)
//!
//! If any test fails, it indicates a potential breaking change. Either:
//! 1. Revert the change to maintain v1 compatibility
//! 2. Create a new schema version (v2) for breaking changes
//! 3. Update the baseline (only if this is intentional for v1.0 release)

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Baseline required fields for receipt.v1.json
/// These fields MUST remain in all future v1.x versions.
/// Update this only if intentionally breaking backward compatibility.
const RECEIPT_V1_REQUIRED_BASELINE: &[&str] = &[
    "schema_version",
    "emitted_at",
    "spec_id",
    "phase",
    "xchecker_version",
    "claude_cli_version",
    "model_full_name",
    "canonicalization_version",
    "canonicalization_backend",
    "flags",
    "runner",
    "packet",
    "outputs",
    "exit_code",
    "warnings",
];

/// Baseline required fields for status.v1.json
const STATUS_V1_REQUIRED_BASELINE: &[&str] = &[
    "schema_version",
    "emitted_at",
    "runner",
    "fallback_used",
    "canonicalization_version",
    "canonicalization_backend",
    "artifacts",
    "last_receipt_path",
    "effective_config",
];

/// Baseline required fields for doctor.v1.json
const DOCTOR_V1_REQUIRED_BASELINE: &[&str] = &["schema_version", "emitted_at", "ok", "checks"];

#[cfg(test)]
mod tests {
    use super::*;

    fn load_schema(path: &str) -> Value {
        let content =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read schema: {}", path));
        serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse schema: {}", path))
    }

    fn get_required_fields(schema: &Value) -> HashSet<String> {
        schema["required"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    // ==========================================================================
    // Receipt v1 Schema Tests
    // ==========================================================================

    #[test]
    fn test_receipt_v1_schema_exists() {
        let path = Path::new("schemas/receipt.v1.json");
        assert!(
            path.exists(),
            "receipt.v1.json must exist at schemas/receipt.v1.json"
        );
    }

    #[test]
    fn test_receipt_v1_required_fields_not_removed() {
        let schema = load_schema("schemas/receipt.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = RECEIPT_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &baseline {
            assert!(
                current_required.contains(*field),
                "BREAKING CHANGE: Required field '{}' was removed from receipt.v1.json. \
                 This breaks backward compatibility for existing consumers. \n\
                 Options:\n\
                 1. Restore the field to maintain v1 compatibility\n\
                 2. Create receipt.v2.json for breaking changes\n\
                 3. Update RECEIPT_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                field
            );
        }
    }

    #[test]
    fn test_receipt_v1_no_new_required_fields() {
        let schema = load_schema("schemas/receipt.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = RECEIPT_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &current_required {
            if !baseline.contains(field.as_str()) {
                panic!(
                    "BREAKING CHANGE: New required field '{}' was added to receipt.v1.json. \
                     This breaks forward compatibility (old code can't read new receipts).\n\
                     Options:\n\
                     1. Make the field optional (remove from 'required' array)\n\
                     2. Create receipt.v2.json for breaking changes\n\
                     3. Update RECEIPT_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                    field
                );
            }
        }
    }

    #[test]
    fn test_receipt_v1_allows_additional_properties() {
        let schema = load_schema("schemas/receipt.v1.json");

        // Default for JSON Schema draft-07 is true if not specified
        let additional_props = schema.get("additionalProperties");

        if let Some(val) = additional_props {
            assert_ne!(
                val,
                &Value::Bool(false),
                "receipt.v1.json MUST allow additional properties for forward compatibility. \
                 Setting additionalProperties: false breaks consumers when new optional fields are added."
            );
        }
        // If not specified, default is true which is fine
    }

    // ==========================================================================
    // Status v1 Schema Tests
    // ==========================================================================

    #[test]
    fn test_status_v1_schema_exists() {
        let path = Path::new("schemas/status.v1.json");
        assert!(
            path.exists(),
            "status.v1.json must exist at schemas/status.v1.json"
        );
    }

    #[test]
    fn test_status_v1_required_fields_not_removed() {
        let schema = load_schema("schemas/status.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = STATUS_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &baseline {
            assert!(
                current_required.contains(*field),
                "BREAKING CHANGE: Required field '{}' was removed from status.v1.json. \
                 This breaks backward compatibility for existing consumers.\n\
                 Options:\n\
                 1. Restore the field to maintain v1 compatibility\n\
                 2. Create status.v2.json for breaking changes\n\
                 3. Update STATUS_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                field
            );
        }
    }

    #[test]
    fn test_status_v1_no_new_required_fields() {
        let schema = load_schema("schemas/status.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = STATUS_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &current_required {
            if !baseline.contains(field.as_str()) {
                panic!(
                    "BREAKING CHANGE: New required field '{}' was added to status.v1.json. \
                     This breaks forward compatibility (old code can't read new status output).\n\
                     Options:\n\
                     1. Make the field optional (remove from 'required' array)\n\
                     2. Create status.v2.json for breaking changes\n\
                     3. Update STATUS_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                    field
                );
            }
        }
    }

    #[test]
    fn test_status_v1_allows_additional_properties() {
        let schema = load_schema("schemas/status.v1.json");
        let additional_props = schema.get("additionalProperties");

        if let Some(val) = additional_props {
            assert_ne!(
                val,
                &Value::Bool(false),
                "status.v1.json MUST allow additional properties for forward compatibility."
            );
        }
    }

    // ==========================================================================
    // Doctor v1 Schema Tests
    // ==========================================================================

    #[test]
    fn test_doctor_v1_schema_exists() {
        let path = Path::new("schemas/doctor.v1.json");
        assert!(
            path.exists(),
            "doctor.v1.json must exist at schemas/doctor.v1.json"
        );
    }

    #[test]
    fn test_doctor_v1_required_fields_not_removed() {
        let schema = load_schema("schemas/doctor.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = DOCTOR_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &baseline {
            assert!(
                current_required.contains(*field),
                "BREAKING CHANGE: Required field '{}' was removed from doctor.v1.json. \
                 This breaks backward compatibility for existing consumers.\n\
                 Options:\n\
                 1. Restore the field to maintain v1 compatibility\n\
                 2. Create doctor.v2.json for breaking changes\n\
                 3. Update DOCTOR_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                field
            );
        }
    }

    #[test]
    fn test_doctor_v1_no_new_required_fields() {
        let schema = load_schema("schemas/doctor.v1.json");
        let current_required = get_required_fields(&schema);
        let baseline: HashSet<&str> = DOCTOR_V1_REQUIRED_BASELINE.iter().copied().collect();

        for field in &current_required {
            if !baseline.contains(field.as_str()) {
                panic!(
                    "BREAKING CHANGE: New required field '{}' was added to doctor.v1.json. \
                     This breaks forward compatibility.\n\
                     Options:\n\
                     1. Make the field optional (remove from 'required' array)\n\
                     2. Create doctor.v2.json for breaking changes\n\
                     3. Update DOCTOR_V1_REQUIRED_BASELINE if this is intentional for v1.0 release",
                    field
                );
            }
        }
    }

    #[test]
    fn test_doctor_v1_allows_additional_properties() {
        let schema = load_schema("schemas/doctor.v1.json");
        let additional_props = schema.get("additionalProperties");

        if let Some(val) = additional_props {
            assert_ne!(
                val,
                &Value::Bool(false),
                "doctor.v1.json MUST allow additional properties for forward compatibility."
            );
        }
    }

    // ==========================================================================
    // Cross-Schema Version Consistency
    // ==========================================================================

    #[test]
    fn test_all_v1_schemas_have_schema_version_field() {
        let schemas = vec![
            "schemas/receipt.v1.json",
            "schemas/status.v1.json",
            "schemas/doctor.v1.json",
        ];

        for path in schemas {
            let schema = load_schema(path);
            let required = get_required_fields(&schema);

            assert!(
                required.contains("schema_version"),
                "{} must have 'schema_version' as a required field for versioning",
                path
            );
        }
    }

    #[test]
    fn test_schema_version_field_is_string() {
        let schemas = vec![
            "schemas/receipt.v1.json",
            "schemas/status.v1.json",
            "schemas/doctor.v1.json",
        ];

        for path in schemas {
            let schema = load_schema(path);
            let props = schema
                .get("properties")
                .expect("Schema should have properties");
            let version_field = props.get("schema_version");

            assert!(
                version_field.is_some(),
                "{} must define 'schema_version' property",
                path
            );

            let version_type = version_field.unwrap().get("type");
            assert_eq!(
                version_type,
                Some(&Value::String("string".to_string())),
                "{} schema_version should be a string type",
                path
            );
        }
    }
}
