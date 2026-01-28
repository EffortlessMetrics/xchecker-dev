//! Phase trait system for orchestrating spec generation workflows
//!
//! This module re-exports the Phase trait and related types from
//! xchecker-phase-api, providing a stable facade for engine modules.

// Re-export all Phase trait types from xchecker-phase-api
pub use xchecker_phase_api::{NextStep, Phase, PhaseContext, PhaseId, PhaseMetadata, PhaseResult};

// Re-export artifact types from status module
pub use crate::status::artifact::Artifact;
