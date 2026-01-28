//! Orchestrator for executing spec generation workflows
//!
//! This crate provides the core orchestration logic that wires together
//! Phase trait, `ArtifactManager`, and Receipt system to execute
//! phases end-to-end with proper error handling and state management.
//!
//! # Architecture
//!
//! The orchestrator module provides:
//! - **PhaseOrchestrator**: Internal orchestration with phase execution logic
//! - **OrchestratorHandle**: Stable facade for external consumers (CLI, Kiro, MCP tools)
//! - **OrchestratorConfig**: Configuration for phase execution
//! - **ExecutionResult**: Result type for phase execution
//!
//! # Module Organization
//!
//! The orchestrator module is organized into sub-modules:
//! - `handle.rs`: Stable facade API for external consumers
//! - `phase_exec.rs`: Single-phase execution with timeout handling
//! - `workflow.rs`: Multi-phase workflow execution with rewind support
//! - `llm.rs`: LLM backend integration and invocation
//!
//! # Integration Rule
//!
//! **Outside this crate, use `OrchestratorHandle` for all production scenarios.**
//! Direct `PhaseOrchestrator` usage is reserved for tests and orchestrator internals.
//!
//! # Public API
//!
//! ## OrchestratorHandle
//!
//! The primary public API for embedding xchecker. Use this for:
//! - Creating and managing specs programmatically
//! - Executing individual phases or full workflow
//! - Querying spec status and artifacts
//! - Configuring execution options
//!
//! ## Example
//!
//! ```rust,no_run
//! use xchecker_orchestrator::OrchestratorHandle;
//! use xchecker_utils::types::PhaseId;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut handle = OrchestratorHandle::new("my-spec")?;
//!
//!     // Run a single phase
//!     handle.run_phase(PhaseId::Requirements).await?;
//!
//!     // Check status
//!     let status = handle.status()?;
//!     println!("Artifacts: {}", status.artifacts.len());
//!
//!     Ok(())
//! }
//! ```

// Declare modules
mod handle;
mod phase_exec;
mod workflow;
mod llm;

// Re-export orchestrator module contents
pub use self::handle::OrchestratorHandle;
pub use self::phase_exec::ExecutionResult;

// Internal types for workflow execution are not re-exported
pub(crate) use self::handle::OrchestratorConfig;
pub(crate) use self::workflow::{PhaseExecution, PhaseExecutionResult, WorkflowResult};
