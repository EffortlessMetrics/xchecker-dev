use anyhow::Result;
use std::fs;
use xchecker::orchestrator::{OrchestratorConfig, OrchestratorHandle};
use xchecker::types::PhaseId;
use xchecker::paths::with_isolated_home;

/// Helper to create a unique spec ID
fn unique_spec_id(test_name: &str) -> String {
    format!("sec-test-{}-{}", test_name, std::process::id())
}

/// Test 33.1: Path traversal rejection via Spec ID
///
/// **Validates: Requirements FR-TEST-6**
#[test]
fn test_spec_id_path_traversal_sanitization() -> Result<()> {
    let _home = with_isolated_home();
    
    // Attempt to create a handle with a path traversal spec ID
    let malicious_id = "../../../etc/passwd";
    
    // The handle creation should succeed, but the ID should be sanitized
    let handle = OrchestratorHandle::new(malicious_id)?;
    
    // Verify the spec ID was sanitized
    assert_ne!(handle.spec_id(), malicious_id);
    assert!(!handle.spec_id().contains(".."));
    assert!(!handle.spec_id().contains("/"));
    
    // It should be sanitized to underscores
    assert!(handle.spec_id().contains("passwd"));
    
    Ok(())
}

/// Test 33.3: Secret redaction in receipts
///
/// **Validates: Requirements FR-TEST-8**
#[tokio::test]
async fn test_secret_redaction_in_receipts() -> Result<()> {
    let _home = with_isolated_home();
    let spec_id = unique_spec_id("secret-redaction");
    
    // 1. Create a config with a secret
    // We'll use a fake AWS key pattern: AKIA... (16 chars +)
    let secret = "AKIAIOSFODNN7EXAMPLE";
    let mut config = OrchestratorConfig {
        dry_run: true,
        ..OrchestratorConfig::default()
    };
    // Inject secret into a field that might be logged or put in error
    // "model" is a good candidate if we expect it to fail and report the model name
    config.config.insert("model".to_string(), secret.to_string());
    
    // We also need to ensure the redactor is configured to catch this.
    // The default redactor should catch AWS keys.
    // But `OrchestratorConfig` has a `redactor` field which is `Arc<SecretRedactor>`.
    // `OrchestratorHandle::with_config_and_force` uses the passed config.
    // We need to make sure the redactor in `config` is set up.
    // `OrchestratorConfig::default()` has a default redactor?
    // Let's check `src/orchestrator/mod.rs` for `Default` impl of `OrchestratorConfig`.
    // If not, we might need to create one.
    
    // Actually, `OrchestratorHandle::with_config_and_force` takes `OrchestratorConfig`.
    // We should construct it properly.
    
    // Let's use `OrchestratorHandle::new` and then `set_config`.
    // `OrchestratorHandle::new` initializes the redactor from the discovered config.
    
    let mut handle = OrchestratorHandle::new(&spec_id)?;
    handle.set_config("model", secret);
    handle.set_dry_run(true);
    
    // 2. Run a phase that will fail due to invalid model (or just run dry run)
    // In dry run, it might not fail on model name.
    // But `Requirements` phase might record the model name in the receipt.
    
    let result = handle.run_phase(PhaseId::Requirements).await?;
    
    // 3. Check the receipt for the secret
    // The receipt is written to disk.
    let receipt_path = result.receipt_path.ok_or_else(|| anyhow::anyhow!("No receipt path"))?;
    let receipt_content = fs::read_to_string(receipt_path)?;
    
    // The secret should NOT be present in the receipt content
    if receipt_content.contains(secret) {
        // If it's present, it's a failure.
        // But wait, if the secret is in the "config" section of the receipt, is it redacted?
        // The receipt schema has `config: HashMap<String, String>`.
        // If `ReceiptManager` doesn't redact config values, then this test will fail (and reveal a bug).
        // Let's assert that it IS redacted.
        panic!("Secret leaked in receipt: {}", receipt_content);
    }
    
    // Verify that we can find the redacted version
    // The default redaction replaces with "***" or "[REDACTED:pattern]"
    // For AWS key, it might be "***".
    // But we can't easily search for "***" as it might be common.
    // However, we know the secret is NOT there.
    
    Ok(())
}

/// Test 33.3: Secret redaction in status output
///
/// **Validates: Requirements FR-TEST-8**
#[test]
fn test_secret_redaction_in_status() -> Result<()> {
    let _home = with_isolated_home();
    let spec_id = unique_spec_id("status-redaction");
    
    let mut handle = OrchestratorHandle::new(&spec_id)?;
    let secret = "AKIAIOSFODNN7EXAMPLE";
    handle.set_config("model", secret);
    
    // Get status
    let status = handle.status()?;
    
    // Check effective config in status
    if let Some(val) = status.effective_config.get("model") {
        let val_str = val.value.as_str().unwrap_or("");
        if val_str.contains(secret) {
             // This confirms that status() leaks secrets if they are in config.
             // Since I cannot fix the code (I am writing tests), I should probably
             // expect this to fail if the code is buggy, or pass if it's secure.
             // Given the security requirements, it SHOULD be redacted.
             // If this test fails, it indicates a security vulnerability.
             panic!("Secret leaked in status output: {}", val_str);
        }
    }
    
    Ok(())
}
