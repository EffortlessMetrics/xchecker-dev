//! LLM integration helpers for `PhaseOrchestrator`.
//!
//! This module contains LLM-related code extracted from mod.rs.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;

use crate::config::{
    ClaudeConfig, Config, Defaults, LlmConfig, PhaseConfig, PhasesConfig, RunnerConfig, Selectors,
};
use crate::error::XCheckerError;
use crate::hooks::HooksConfig;
use crate::llm::{LlmBackend, LlmInvocation, LlmResult, Message};
use crate::types::PhaseId;

use super::{OrchestratorConfig, PhaseOrchestrator};

/// Metadata from Claude CLI execution for receipt generation.
///
/// Internal type used to track LLM execution details that get written to receipts.
/// This type is specific to the Claude CLI backend and will be generalized in future versions.
#[derive(Debug, Clone)]
pub(crate) struct ClaudeExecutionMetadata {
    pub model_alias: Option<String>,
    pub model_full_name: String,
    pub claude_cli_version: String,
    pub fallback_used: bool,
    pub runner: String,
    pub runner_distro: Option<String>,
    pub stderr_tail: Option<String>,
}

impl PhaseOrchestrator {
    /// Build a minimal Config from `OrchestratorConfig` for LLM backend construction.
    ///
    /// Internal helper for V11 that extracts the necessary configuration
    /// from `OrchestratorConfig`. Future versions may pass full `Config` to orchestrator.
    ///
    /// This is not part of the public API.
    pub(crate) fn config_from_orchestrator_config(
        &self,
        orc_config: &OrchestratorConfig,
    ) -> Config {
        // Extract values from OrchestratorConfig
        let model = orc_config.config.get("model").cloned();
        let phase_timeout = orc_config
            .config
            .get("phase_timeout")
            .and_then(|s| s.parse::<u64>().ok());
        let runner_mode = orc_config.config.get("runner_mode").cloned();
        let runner_distro = orc_config.config.get("runner_distro").cloned();
        let claude_path = orc_config.config.get("claude_path").cloned();
        let llm_provider = orc_config.config.get("llm_provider").cloned();
        let llm_claude_binary = orc_config.config.get("llm_claude_binary").cloned();

        // Helper to build PhaseConfig from OrchestratorConfig keys
        let build_phase_config = |phase_name: &str| -> Option<PhaseConfig> {
            let model = orc_config
                .config
                .get(&format!("phases.{phase_name}.model"))
                .cloned();
            let max_turns = orc_config
                .config
                .get(&format!("phases.{phase_name}.max_turns"))
                .and_then(|s| s.parse::<u32>().ok());
            let phase_timeout = orc_config
                .config
                .get(&format!("phases.{phase_name}.phase_timeout"))
                .and_then(|s| s.parse::<u64>().ok());

            if model.is_some() || max_turns.is_some() || phase_timeout.is_some() {
                Some(PhaseConfig {
                    model,
                    max_turns,
                    phase_timeout,
                })
            } else {
                None
            }
        };

        // Build phases config from OrchestratorConfig
        let phases = PhasesConfig {
            requirements: build_phase_config("requirements"),
            design: build_phase_config("design"),
            tasks: build_phase_config("tasks"),
            review: build_phase_config("review"),
            fixup: build_phase_config("fixup"),
            final_: build_phase_config("final"),
        };

        // Build Config
        Config {
            defaults: Defaults {
                model,
                phase_timeout,
                ..Defaults::default()
            },
            selectors: Selectors::default(),
            runner: RunnerConfig {
                mode: runner_mode,
                distro: runner_distro,
                claude_path,
            },
            llm: LlmConfig {
                provider: llm_provider,
                fallback_provider: None, // Fallback provider not supported in orchestrator minimal config yet
                claude: llm_claude_binary.map(|binary| ClaudeConfig {
                    binary: Some(binary),
                }),
                gemini: None, // Gemini config not supported in orchestrator minimal config yet
                openrouter: None, // OpenRouter config not supported in orchestrator minimal config yet
                anthropic: None, // Anthropic config not supported in orchestrator minimal config yet
                execution_strategy: None, // Will be set by Config::discover
                prompt_template: None, // Will use default template
            },
            phases,
            hooks: HooksConfig::default(),
            source_attribution: HashMap::new(),
        }
    }

    /// Construct LLM backend based on configuration.
    ///
    /// Internal helper that creates the appropriate LLM backend.
    /// In V11, only `ClaudeCliBackend` is supported. Future versions will support
    /// multiple providers based on config.
    ///
    /// This is not part of the public API.
    pub(crate) fn make_llm_backend(
        &self,
        orc_config: &OrchestratorConfig,
    ) -> Result<Box<dyn LlmBackend>, XCheckerError> {
        // Build a Config from OrchestratorConfig
        let cfg = self.config_from_orchestrator_config(orc_config);

        // Use the factory function to construct the appropriate backend
        crate::llm::from_config(&cfg).map_err(XCheckerError::Llm)
    }

    /// Build `LlmInvocation` from packet and phase context.
    ///
    /// Internal helper that constructs an invocation with model, timeout, and messages.
    /// Uses per-phase model configuration with precedence:
    /// 1. Phase-specific override (`[phases.<phase>].model`)
    /// 2. Global default (`[defaults].model`)
    /// 3. Hard default: `"haiku"`
    ///
    /// This is not part of the public API.
    pub(crate) fn build_llm_invocation(
        &self,
        phase_id: PhaseId,
        prompt: &str,
        orc_config: &OrchestratorConfig,
    ) -> LlmInvocation {
        // Build Config from OrchestratorConfig to use model_for_phase
        let cfg = self.config_from_orchestrator_config(orc_config);

        // Get model for this specific phase using the precedence rules
        let model = cfg.model_for_phase(phase_id);

        // Get timeout from config (default 600 seconds)
        let timeout_secs = orc_config
            .config
            .get("phase_timeout")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(600);
        let timeout = Duration::from_secs(timeout_secs);

        // Build messages array
        // For now, we use a simple user message with the prompt content
        // This preserves the existing prompt-building logic from execute_claude_cli
        let messages = vec![Message::user(prompt)];

        // Create invocation
        LlmInvocation::new(&self.spec_id, phase_id.as_str(), model, timeout, messages)
    }

    /// Execute LLM invocation using the backend abstraction.
    ///
    /// Internal helper that invokes the LLM backend and converts results to the format
    /// expected by the orchestrator's execution flow.
    ///
    /// Returns `(response_text, exit_code, metadata, llm_result)` tuple compatible with existing code.
    ///
    /// This is not part of the public API.
    pub(crate) async fn run_llm_invocation(
        &self,
        prompt: &str,
        phase_id: PhaseId,
        config: &OrchestratorConfig,
    ) -> Result<(
        String,
        i32,
        Option<ClaudeExecutionMetadata>,
        Option<LlmResult>,
    )> {
        // Build LLM invocation
        let invocation = self.build_llm_invocation(phase_id, prompt, config);

        // Get backend
        let backend = self.make_llm_backend(config)?;

        // Invoke LLM
        let llm_result = backend
            .invoke(invocation)
            .await
            .map_err(XCheckerError::Llm)?;

        // For V11, we need to convert LlmResult back to the format expected by existing code
        // This maintains compatibility while using the new abstraction
        let metadata = ClaudeExecutionMetadata {
            model_alias: None, // LlmResult doesn't track alias yet
            model_full_name: llm_result.model_used.clone(),
            claude_cli_version: "0.8.1".to_string(), // TODO: Extract from extensions if available
            fallback_used: false,                    // Not tracked in V11
            runner: "native".to_string(),            // TODO: Extract from extensions if available
            runner_distro: None,
            stderr_tail: llm_result
                .extensions
                .get("stderr")
                .and_then(|v| v.as_str().map(String::from)),
        };

        // Exit code is 0 for success (we got a result)
        // Errors are handled via XCheckerError::Llm mapping
        Ok((
            llm_result.raw_response.clone(),
            0,
            Some(metadata),
            Some(llm_result),
        ))
    }
}
