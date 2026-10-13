use crate::errors::CliError;
use crate::generator::{ModuleConfig, backend, migration};
use crate::logging::LogMessage;

#[derive(Debug, Clone)]
pub struct CreateModuleParams {
    pub name: String,
    pub description: String,
}

pub fn generate_module(params: &CreateModuleParams) -> Result<(), CliError> {
    let cfg = ModuleConfig::new(&params.name, &params.description);

    LogMessage::info("Summary:");
    LogMessage::step(&format!("Module: {} ({})", cfg.name_pascal, cfg.name));
    LogMessage::step(&format!("Description: {}", cfg.description));

    LogMessage::info("Generating backend files...");
    backend::generate(&cfg)?;

    LogMessage::info("Generating migration...");
    migration::generate(&cfg)?;

    LogMessage::success(&format!(
        "Module '{}' generated successfully!",
        cfg.name_pascal
    ));

    Ok(())
}

pub fn run() -> Result<(), CliError> {
    use dialoguer::Input;
    use dialoguer::theme::ColorfulTheme;

    LogMessage::info("Create Module Generator");
    println!();

    // 1. Module name
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Module name (snake_case)")
        .validate_with(|input: &String| {
            if input.is_empty() {
                return Err("Module name cannot be empty");
            }
            if input.contains(' ') {
                return Err("Module name cannot contain spaces");
            }
            Ok(())
        })
        .interact_text()
        .map_err(|e| CliError::ParseError(format!("Failed to read module name: {e}")))?;

    let name = name.trim().to_lowercase();
    let cfg_for_desc = ModuleConfig::new(&name, "");

    // 2. Description
    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Description for {}", cfg_for_desc.name_pascal))
        .default(format!("{} management", cfg_for_desc.name_pascal))
        .interact_text()
        .map_err(|e| CliError::ParseError(format!("Failed to read description: {e}")))?;

    let params = CreateModuleParams { name, description };
    generate_module(&params)?;

    println!();
    LogMessage::info("Next steps:");
    let cfg = ModuleConfig::new(&params.name, &params.description);
    LogMessage::step(&format!(
        "Run `sea-orm-cli migrate generate create_{}_table` if migration didn't auto-generate",
        cfg.name
    ));
    LogMessage::step("Run `just generate-entities` to regenerate SeaORM models");
    LogMessage::step("Run `cargo check --workspace` to verify compilation");

    Ok(())
}
