//! Facade module for phase implementations
//!
//! This module provides a stable facade for all phase implementations,
//! re-exporting phases from the xchecker-phases crate and the fixup module.

// Phases moved to xchecker-phases crate:
pub use xchecker_phases::{DesignPhase, RequirementsPhase, ReviewPhase, TasksPhase};

// FixupPhase remains in the fixup module until Wave 5
pub use crate::fixup::FixupPhase;
