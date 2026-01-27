/// Generate status output for a spec.
///
/// This is the primary status API used by the CLI `xchecker status` command.
///
/// # Arguments
///
/// * `spec_id` - The spec ID to generate status for
/// * `effective_config` - Optional effective configuration (for programmatic use)
/// * `lock_drift` - Optional lock drift status (for debugging)
/// * `pending_fixups` - Optional pending fixups (for debugging)
///
/// # Returns
///
/// * `Ok(StatusOutput)` - The generated status output
/// * `Err(XCheckerError)` - Status generation failed
///
/// # Example
///
/// ```rust,no_run
/// use xchecker_engine::status::generate_status;
/// use xchecker_engine::types::SpecId;
///
/// let status = generate_status("my-spec", None, None, None)?;
/// println!("Artifacts: {}", status.artifacts.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Spec ID is invalid
/// - Status generation fails
pub fn generate_status(
    spec_id: &str,
    effective_config: Option<&std::collections::BTreeMap<String, (String, String)>>,
    _lock_drift: Option<&str>,
    _pending_fixups: Option<&str>,
) -> Result<crate::types::StatusOutput, crate::error::XCheckerError> {
    use std::collections::BTreeMap;
    use camino::Utf8PathBuf;

    // Build effective config from orchestrator config
    let mut effective_config_map: BTreeMap<String, crate::types::ConfigValue> = BTreeMap::new();
    if let Some(config) = effective_config {
        for (key, (value, source)) in config {
            effective_config_map.insert(
                key.clone(),
                crate::types::ConfigValue {
                    value: serde_json::json!(value),
                    source: match source.as_str() {
                        "cli" => crate::types::ConfigSource::Cli,
                        "env" => crate::types::ConfigSource::Env,
                        "config" => crate::types::ConfigSource::Config,
                        "programmatic" => crate::types::ConfigSource::Programmatic,
                        _ => crate::types::ConfigSource::Default,
                    },
                },
            );
        }
    }

    // Get artifact manager
    let artifact_manager = crate::artifact::ArtifactManager::new(spec_id)
        .map_err(|e| crate::error::XCheckerError::Config(crate::error::ConfigError::DiscoveryFailed {
                reason: format!("Failed to create artifact manager: {}", e),
            }))?;

    // Get artifacts
    let artifacts = artifact_manager.list_artifacts()
        .map_err(|e| crate::error::XCheckerError::Config(crate::error::ConfigError::DiscoveryFailed {
                reason: format!("Failed to list artifacts: {}", e),
            }))?
        .iter()
        .map(|artifact_path| {
            let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(artifact_path.clone()))
                .unwrap_or_else(|_| Utf8PathBuf::from(artifact_path));
            crate::types::ArtifactInfo {
                path: path.to_string(),
                blake3_first8: String::new(),
            }
        })
        .collect::<Vec<_>>();

    // Get effective configuration
    let effective_config_output = if artifacts.is_empty() {
        std::collections::BTreeMap::new()
    } else {
        effective_config_map
    };

    Ok(crate::types::StatusOutput {
        schema_version: "1".to_string(),
        emitted_at: chrono::Utc::now(),
        runner: "native".to_string(),
        runner_distro: None,
        fallback_used: false,
        canonicalization_version: "yaml-v1,md-v1".to_string(),
        canonicalization_backend: "jcs-rfc8785".to_string(),
        artifacts,
        last_receipt_path: "receipts/latest.json".to_string(),
        effective_config: effective_config_output,
        lock_drift: None,
        pending_fixups: None,
    })
}
