//! Template command implementation
//!
//! Handles `xchecker template` subcommands.

use anyhow::Result;

use crate::cli::args::TemplateCommands;
use crate::error::ConfigError;
use crate::spec_id::sanitize_spec_id;
use crate::XCheckerError;

/// Execute template management commands
/// Per FR-TEMPLATES (Requirements 4.7.1, 4.7.2, 4.7.3)
pub fn execute_template_command(cmd: TemplateCommands) -> Result<()> {
    use crate::template;

    match cmd {
        TemplateCommands::List => {
            println!("Available templates:\n");

            for t in template::list_templates() {
                println!("  {}", t.id);
                println!("    Name: {}", t.name);
                println!("    Description: {}", t.description);
                println!("    Use case: {}", t.use_case);
                if !t.prerequisites.is_empty() {
                    println!("    Prerequisites: {}", t.prerequisites.join(", "));
                }
                println!();
            }

            println!("To initialize a spec from a template:");
            println!("  xchecker template init <template> <spec-id>");

            Ok(())
        }
        TemplateCommands::Init { template, spec_id } => {
            // Sanitize spec ID
            let sanitized_id = sanitize_spec_id(&spec_id).map_err(|e| {
                XCheckerError::Config(ConfigError::InvalidValue {
                    key: "spec_id".to_string(),
                    value: format!("{e}"),
                })
            })?;

            // Validate template
            if !template::is_valid_template(&template) {
                let valid_templates = template::BUILT_IN_TEMPLATES.join(", ");
                return Err(XCheckerError::Config(ConfigError::InvalidValue {
                    key: "template".to_string(),
                    value: format!(
                        "Unknown template '{}'. Valid templates: {}",
                        template, valid_templates
                    ),
                })
                .into());
            }

            // Initialize from template
            template::init_from_template(&template, &sanitized_id)?;

            // Get template info for display
            let template_info = template::get_template(&template).unwrap();

            println!(
                "✓ Initialized spec '{}' from template '{}'",
                sanitized_id, template
            );
            println!();
            println!("Template: {}", template_info.name);
            println!("Description: {}", template_info.description);
            println!();
            println!("Created files:");
            println!(
                "  - .xchecker/specs/{}/context/problem-statement.md",
                sanitized_id
            );
            println!("  - .xchecker/specs/{}/README.md", sanitized_id);
            println!();
            println!("Next steps:");
            println!("  1. Review the problem statement:");
            println!(
                "     cat .xchecker/specs/{}/context/problem-statement.md",
                sanitized_id
            );
            println!("  2. Customize the problem statement for your needs");
            println!("  3. Run the requirements phase:");
            println!("     xchecker resume {} --phase requirements", sanitized_id);

            Ok(())
        }
    }
}
