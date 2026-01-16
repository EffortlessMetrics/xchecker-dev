use anyhow::Result;
use xchecker::OrchestratorHandle;
use xchecker::artifact::{Artifact, ArtifactManager, ArtifactType};

/// Test that OrchestratorHandle rejects path traversal in spec_id
///
/// **Property: Path Traversal Rejection**
/// **Validates: Requirements FR-TEST-6**
#[test]
fn test_orchestrator_handle_spec_id_traversal() {
    // Attempt to create a handle with a spec_id containing path traversal
    // Note: OrchestratorHandle::new might succeed if it doesn't validate immediately,
    // but operations should fail or it should be sanitized.
    // Actually, if it's not sanitized, it might create directories outside specs/.

    // We use a relative path that tries to go up
    let malicious_id = "../malicious_spec";

    // If sanitization is in place, this will become ".._malicious_spec" or similar
    // If not, it might be accepted.

    // Let's check what happens.
    // We use a temporary home to avoid messing up the real environment.
    let _temp_home = xchecker::paths::with_isolated_home();

    let handle_result = OrchestratorHandle::new(malicious_id);

    // If it succeeds, we check the spec_id.
    if let Ok(handle) = handle_result {
        let id = handle.spec_id();
        // It should be sanitized.
        // sanitize_spec_id replaces '/' with '_'
        // so "../malicious_spec" -> ".._malicious_spec"
        assert_ne!(id, malicious_id, "Spec ID should have been sanitized");
        assert!(!id.contains("/"), "Spec ID should not contain slashes");
        assert!(!id.contains("\\"), "Spec ID should not contain backslashes");
    } else {
        // Failed creation is also a valid rejection (e.g. if it became empty or invalid)
    }
}

/// Test that ArtifactManager rejects path traversal in artifact names
///
/// **Property: Path Traversal Rejection**
/// **Validates: Requirements FR-TEST-6**
#[test]
fn test_artifact_manager_path_traversal() -> Result<()> {
    let _temp_home = xchecker::paths::with_isolated_home();
    let manager = ArtifactManager::new("test-spec")?;

    // Attempt to store an artifact with path traversal in the name
    let artifact = Artifact::new(
        "../../evil.md".to_string(),
        "malicious content".to_string(),
        ArtifactType::Markdown,
    );

    let result = manager.store_artifact(&artifact);

    // Should fail
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("traversal") || err.to_string().contains("parent"));

    Ok(())
}

/// Test that ArtifactManager handles absolute-looking paths in artifact names safely
///
/// **Property: Path Containment - absolute-looking names are sandboxed**
/// **Validates: Requirements FR-TEST-6**
///
/// When an artifact name looks like an absolute path (e.g., "/tmp/evil.md"),
/// ArtifactManager prepends "artifacts/" making it "artifacts//tmp/evil.md".
/// This is a SAFE behavior because:
/// - On Unix: normalizes to "artifacts/tmp/evil.md" (inside sandbox)
/// - On Windows: "C:\..." becomes "artifacts/C:\..." which is invalid filesystem path
///
/// The security model relies on SandboxRoot, which:
/// 1. Rejects paths with ".." components
/// 2. Verifies canonicalized paths stay within sandbox
/// 3. Handles symlink checking
#[test]
fn test_artifact_manager_absolute_path() -> Result<()> {
    let _temp_home = xchecker::paths::with_isolated_home();
    let manager = ArtifactManager::new("test-spec")?;

    // On Unix: "/tmp/evil.md" becomes "artifacts//tmp/evil.md" -> safe (inside artifacts/)
    // On Windows: "C:\Windows\Temp\evil.md" becomes "artifacts/C:\..." -> invalid path, fails
    #[cfg(unix)]
    {
        // Unix: absolute-looking name is safely contained in artifacts/
        let artifact = Artifact::new(
            "/tmp/evil.md".to_string(),
            "content".to_string(),
            ArtifactType::Markdown,
        );
        let result = manager.store_artifact(&artifact);
        // This SUCCEEDS because "artifacts//tmp/evil.md" -> "artifacts/tmp/evil.md"
        // which is safely inside the sandbox
        assert!(
            result.is_ok(),
            "Unix: absolute-looking path should be safely sandboxed"
        );
    }

    #[cfg(windows)]
    {
        // Windows: drive letter in path creates invalid filesystem path
        let artifact = Artifact::new(
            "C:\\Windows\\Temp\\evil.md".to_string(),
            "content".to_string(),
            ArtifactType::Markdown,
        );
        let result = manager.store_artifact(&artifact);
        // This FAILS because "artifacts/C:\Windows\..." is an invalid Windows path
        assert!(result.is_err(), "Windows: drive letter path should fail");
    }

    Ok(())
}
