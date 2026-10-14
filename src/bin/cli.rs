use bcrypt::{DEFAULT_COST, hash};
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Password, Select};
use lib_asterisk::config::db::init_db_pool;
use lib_asterisk::config::env::load_config;
use lib_asterisk::errors::app_error::AppError;
use lib_asterisk::models::users;
use lib_asterisk::repositories::app_config_repository::AppConfigRepositoryExt;
use lib_asterisk::repositories::user_roles_repository::UserRolesRepositoryExt;
use lib_asterisk::state::Application;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use ulid::Ulid;

#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Parse Error: {0}")]
    ParseError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Parser)]
#[command(
    name = "asterisk CLI",
    version = env!("CARGO_PKG_VERSION"),
    about = "Command line interface for asterisk application",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Asterisk {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the asterisk application
    Init,

    /// Create a new user account
    CreateUser,
}

impl From<AppError> for CliError {
    fn from(value: AppError) -> Self {
        Self::OperationFailed(value.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let app_config = load_config()?;

    let db = init_db_pool(&app_config).await?;
    let app = Application::new(&db).await?;

    let cli = Asterisk::parse();
    parse_commands(cli, &app).await?;
    Ok(())
}

async fn parse_commands(cli: Asterisk, app: &Application) -> Result<(), CliError> {
    match cli.command {
        Commands::Init => init(app).await?,
        Commands::CreateUser => create_user(app).await?,
    }

    Ok(())
}

async fn init(app: &Application) -> Result<(), CliError> {
    let existing = app
        .repositories
        .app_config
        .find_app_config()
        .await
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if existing.is_some() {
        tracing::warn!("App config already initialized, skipping.");
        return Ok(());
    }

    let app_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Application name")
        .default("asterisk".into())
        .interact_text()
        .map_err(|e| AppError::OperationFailed(e.to_string()))?;

    let email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Support email")
        .validate_with(|input: &String| {
            if input.contains('@') {
                Ok(())
            } else {
                Err("Invalid email format")
            }
        })
        .interact_text()
        .map_err(|e| CliError::ConfigError(e.to_string()))?;

    let config_ulid = Ulid::new().to_string();

    app.repositories
        .app_config
        .create_app_config(&config_ulid, Some(app_name), Some(email), None, None, None)
        .await
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    tracing::info!("Application initialized successfully.");
    Ok(())
}

struct RoleIdentifier(String);

impl RoleIdentifier {
    fn new(identifier: String) -> Self {
        Self(identifier)
    }

    fn expose(&self) -> String {
        self.0.to_string()
    }
}

async fn select_role(app: &Application) -> Result<RoleIdentifier, CliError> {
    let mut roles = app
        .repositories
        .user_roles
        .fetch_all_roles()
        .await
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if roles.is_empty() {
        return Err(CliError::OperationFailed(
            "No active roles are available".to_string(),
        ));
    }

    roles.sort_by(|a, b| a.name.cmp(&b.name));

    let role_names = roles
        .iter()
        .map(|role| role.name.as_str())
        .collect::<Vec<_>>();
    let default_index = roles
        .iter()
        .position(|role| role.name == "super_admin")
        .unwrap_or(0);

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a role")
        .default(default_index)
        .items(&role_names)
        .interact()
        .map_err(|e| CliError::ParseError(format!("Failed to select role: {}", e)))?;

    Ok(RoleIdentifier::new(
        roles[selected_index].identifier.clone(),
    ))
}

async fn create_user(app: &Application) -> Result<(), CliError> {
    let selected_role_id = select_role(app).await?;

    let first_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("First name")
        .interact()
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    let last_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Last name")
        .interact()
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    let admin_email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("user email")
        .interact_text()
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    let admin_password: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("User password")
        .interact()
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    let two_factor_enabled: bool = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable two-factor authentication (email OTP)?")
        .default(true)
        .interact()
        .map_err(|e| CliError::ParseError(e.to_string()))?;

    let existing = users::Entity::find()
        .filter(users::Column::Email.eq(&admin_email))
        .one(&app.db)
        .await
        .map_err(|e| CliError::OperationFailed(e.to_string()))?;

    if existing.is_some() {
        println!("User with email '{}' already exists", admin_email);
        return Ok(());
    }

    let new_admin_id = Ulid::new().to_string();
    let hashed_password = hash(admin_password.trim(), DEFAULT_COST)
        .map_err(|err| CliError::ParseError(err.to_string()))?;

    users::ActiveModel {
        identifier: Set(new_admin_id),
        role_identifier: Set(Some(selected_role_id.expose())),
        email: Set(admin_email.clone()),
        password: Set(hashed_password),
        is_active: Set(true),
        first_name: Set(first_name.into()),
        last_name: Set(last_name.into()),
        two_factor_enabled: Set(two_factor_enabled),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .map_err(|err| CliError::OperationFailed(err.to_string()))?;

    println!(
        "User created successfully with email '{}' (two-factor {})",
        admin_email,
        if two_factor_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    Ok(())
}
