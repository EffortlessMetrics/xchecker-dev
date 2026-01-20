//! Doctor command implementation
//!
//! Handles `xchecker doctor` command for environment health checks.

use anyhow::{Context, Result};

use crate::{emit_jcs, Config};

/// Execute the doctor command for environment health checks
pub fn execute_doctor_command(json: bool, strict_exit: bool, config: &Config) -> Result<()> {
    use crate::doctor::DoctorCommand;

    // Create and run doctor command (wired through Doctor::run)
    let mut doctor = DoctorCommand::new(config.clone());
    let output = doctor
        .run_with_options_strict(strict_exit)
        .context("Failed to run doctor checks")?;

    if json {
        // Emit as canonical JSON (JCS) for stable diffs (FR-CLI-6)
        // Use emit_jcs for consistent canonicalization with receipts/status
        let json_output = emit_jcs(&output).context("Failed to emit doctor JSON")?;
        println!("{json_output}");
    } else {
        // Use log_doctor_report for human-readable output (wired into logging)
        crate::logging::log_doctor_report(&output);

        if !output.ok {
            println!();
            if strict_exit {
                println!(
                    "Some checks failed or warned (strict mode). Please address the issues above."
                );
            } else {
                println!(
                    "Some checks failed. Please address the issues above before using xchecker."
                );
            }
        }
    }

    // Exit with non-zero code if any check failed (R5.6)
    // In strict mode, warnings also cause non-zero exit
    if !output.ok {
        std::process::exit(1);
    }

    Ok(())
}
