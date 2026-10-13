use std::{str::FromStr, time::Duration};

use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::{errors::app_error::AppError, utils::extract_env::extract_env};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // Server
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_environment")]
    pub environment: Environment,

    #[serde(default = "default_body_limit_megabytes")]
    pub body_limit_megabytes: usize,

    // Storage
    #[serde(default = "default_upload_path")]
    pub upload_path: String,

    #[serde(default = "default_export_path")]
    pub export_path: String,

    // CORS
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,

    // Email
    pub email_provider: EmailProvider,
    pub email_from_address: String,
    pub email_from_name: String,

    // Email (SMTP, only read when email_provider is Smtp)
    pub smtp_host: SecretString,
    pub smtp_port: u16,
    pub smtp_username: SecretString,
    pub smtp_password: SecretString,
    pub smtp_tls_mode: SmtpTlsMode,

    // Email (Mailtrap API, only read when email_provider is Mailtrap)
    pub mailtrap_api_token: SecretString,
    pub mailtrap_base_url: String,

    // Database
    pub database_url: SecretString,
    pub max_db_connections: u32,
    pub redis_url: SecretString,

    #[serde(default = "default_requests_time_out")]
    pub requests_time_out_secs: Duration,

    // Rate limiting
    #[serde(default = "default_rate_limit_requests_per_second")]
    pub rate_limit_requests_per_second: u32,

    #[serde(default = "default_rate_limit_burst")]
    pub rate_limit_burst: u32,

    #[serde(default = "default_rate_limit_requests_per_minute")]
    pub rate_limit_requests_per_minute: u32,

    #[serde(default = "default_rate_limit_max_keys")]
    pub rate_limit_max_keys: usize,

    #[serde(default = "default_rate_limit_auth_per_minute")]
    pub rate_limit_auth_per_minute: u32,

    #[serde(default = "default_rate_limit_auth_per_hour")]
    pub rate_limit_auth_per_hour: u32,

    #[serde(default = "default_rate_limit_otp_per_minute")]
    pub rate_limit_otp_per_minute: u32,

    #[serde(default = "default_rate_limit_otp_per_hour")]
    pub rate_limit_otp_per_hour: u32,

    // imagekit
    pub imagekit_public_key: SecretString,
    pub imagekit_private_key: SecretString,
    #[serde(default = "default_imagekit_upload_url")]
    pub imagekit_upload_url: String,

    // payment providers
    pub paystack_api_key: SecretString,
    pub paystack_api_secret: SecretString,
    pub paystack_base_url: String,
    #[serde(default = "default_paystack_verify_base_url")]
    pub paystack_verify_base_url: String,

    // Token TTLs
    pub access_token_ttl_secs: Duration,
    pub refresh_token_ttl_secs: Duration,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    // Frontend
    #[serde(default = "default_frontend_url")]
    pub frontend_url: String,

    // Console
    #[serde(default = "default_console_url")]
    pub console_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        tracing::info!("Loading application configuration...");

        let database_url = extract_env::<String>("DATABASE_URL")?;
        let database_url = database_url.into_boxed_str();

        let paystack_api_key = extract_env::<String>("PAYSTACK_API_KEY")?;
        let paystack_api_secret = extract_env::<String>("PAYSTACK_API_SECRET")?;
        let paystack_base_url = extract_env::<String>("PAYSTACK_BASE_URL")?;

        let requests_time_out = extract_env::<u64>("REQUESTS_TIME_OUT_SECS")
            .unwrap_or_else(|_| default_requests_time_out().as_secs());

        let environment =
            extract_env::<Environment>("ENVIRONMENT").unwrap_or_else(|_| default_environment());

        let email_provider =
            extract_env("EMAIL_PROVIDER").unwrap_or_else(|_| default_email_provider());

        let smtp_tls_mode =
            extract_env("SMTP_TLS_MODE").unwrap_or_else(|_| default_smtp_tls_mode());

        // Only the selected provider's credentials are required, so a deployment
        // that only sets SMTP_* still boots on the API path, and vice versa. An
        // absent key reads as empty rather than as an error.
        let smtp_host = extract_optional_env::<String>("SMTP_HOST").unwrap_or_default();
        let smtp_username =
            extract_optional_env::<String>("SMTP_AUTH_USERNAME").unwrap_or_default();
        let smtp_password =
            extract_optional_env::<String>("SMTP_AUTH_PASSWORD").unwrap_or_default();
        let mailtrap_api_token =
            extract_optional_env::<String>("MAILTRAP_API_TOKEN").unwrap_or_default();

        // Validate only what the selected provider will actually dial, so a
        // half-configured deployment fails at boot with a message naming the
        // missing key rather than at the first send.
        match email_provider {
            EmailProvider::Smtp => {
                if smtp_host.trim().is_empty() {
                    return Err(AppError::EnvError(
                        "SMTP_HOST is required when EMAIL_PROVIDER=smtp".to_string(),
                    ));
                }

                if environment == Environment::Production && smtp_tls_mode == SmtpTlsMode::None {
                    return Err(AppError::EnvError(
                        "SMTP_TLS_MODE=none is not allowed when ENVIRONMENT=production".to_string(),
                    ));
                }
            }
            EmailProvider::Mailtrap => {
                if mailtrap_api_token.trim().is_empty() {
                    return Err(AppError::EnvError(
                        "MAILTRAP_API_TOKEN is required when EMAIL_PROVIDER=mailtrap".to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            // Server
            port: extract_env("PORT").unwrap_or_else(|_| default_port()),
            environment,
            body_limit_megabytes: extract_env("BODY_LIMIT_MEGA_BYTES")
                .unwrap_or_else(|_| default_body_limit_megabytes()),

            // Storage
            upload_path: extract_env("UPLOAD_PATH").unwrap_or_else(|_| default_upload_path()),
            export_path: extract_env("EXPORT_PATH").unwrap_or_else(|_| default_export_path()),

            // payment providers
            paystack_api_key: SecretString::from(paystack_api_key),
            paystack_api_secret: SecretString::from(paystack_api_secret),
            paystack_base_url,
            paystack_verify_base_url: extract_env("PAYSTACK_VERIFY_BASE_URL")
                .unwrap_or_else(|_| default_paystack_verify_base_url()),

            // CORS
            allowed_origins: extract_env::<String>("ALLOWED_ORIGINS")?
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect(),

            // Email
            email_provider,
            email_from_address: extract_env("EMAIL_FROM_ADDRESS")
                .unwrap_or_else(|_| default_email_from_address()),
            email_from_name: extract_env("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| default_email_from_name()),

            // Email (SMTP)
            smtp_host: SecretString::from(smtp_host),
            smtp_port: extract_env("SMTP_PORT").unwrap_or(587),
            smtp_username: SecretString::from(smtp_username),
            smtp_password: SecretString::from(smtp_password),
            smtp_tls_mode,

            // Email (Mailtrap API)
            mailtrap_api_token: SecretString::from(mailtrap_api_token),
            mailtrap_base_url: extract_env("MAILTRAP_BASE_URL")
                .unwrap_or_else(|_| default_mailtrap_base_url()),

            // Database
            database_url: SecretString::new(database_url),
            max_db_connections: extract_env("MAX_DB_CONNECTIONS")?,

            requests_time_out_secs: Duration::from_secs(requests_time_out),

            // Rate limiting
            rate_limit_requests_per_second: extract_env("RATE_LIMIT_RPS")
                .unwrap_or_else(|_| default_rate_limit_requests_per_second()),
            rate_limit_burst: extract_env("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| default_rate_limit_burst()),
            rate_limit_requests_per_minute: extract_env("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| default_rate_limit_requests_per_minute()),
            rate_limit_max_keys: extract_env("RATE_LIMIT_MAX_KEYS")
                .unwrap_or_else(|_| default_rate_limit_max_keys()),
            rate_limit_auth_per_minute: extract_env("RATE_LIMIT_AUTH_PER_MINUTE")
                .unwrap_or_else(|_| default_rate_limit_auth_per_minute()),
            rate_limit_auth_per_hour: extract_env("RATE_LIMIT_AUTH_PER_HOUR")
                .unwrap_or_else(|_| default_rate_limit_auth_per_hour()),
            rate_limit_otp_per_minute: extract_env("RATE_LIMIT_OTP_PER_MINUTE")
                .unwrap_or_else(|_| default_rate_limit_otp_per_minute()),
            rate_limit_otp_per_hour: extract_env("RATE_LIMIT_OTP_PER_HOUR")
                .unwrap_or_else(|_| default_rate_limit_otp_per_hour()),

            imagekit_private_key: SecretString::from(extract_env::<String>(
                "IMAGEKIT_PRIVATE_KEY",
            )?),

            imagekit_public_key: SecretString::from(extract_env::<String>("IMAGEKIT_PUBLIC_KEY")?),
            imagekit_upload_url: extract_env::<String>("IMAGEKIT_UPLOAD_URL")
                .unwrap_or_else(|_| default_imagekit_upload_url()),
            redis_url: SecretString::from(extract_env::<String>("REDIS_CONNECTION_URL")?),

            // Token TTLs
            access_token_ttl_secs: Duration::from_secs(
                extract_env::<u64>("ACCESS_TOKEN_TTL_IN_MINUTES").unwrap_or(10) * 60,
            ),
            refresh_token_ttl_secs: Duration::from_secs(
                extract_env::<u64>("REFRESH_TOKEN_TTL_IN_MINUTES").unwrap_or(420) * 60,
            ),
            cache_ttl: extract_env::<u64>("CACHE_TTL_IN_SECONDS")
                .unwrap_or_else(|_| default_cache_ttl()),

            // Frontend
            frontend_url: extract_env("FRONTEND_URL").unwrap_or_else(|_| default_frontend_url()),

            // Console
            console_url: extract_env("CONSOLE_URL").unwrap_or_else(|_| default_console_url()),
        })
    }

    pub fn current_env() -> Environment {
        match std::env::var("ENVIRONMENT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "development" | "dev" => Environment::Development,
            "production" | "prod" => Environment::Production,
            "test" | "testing" => Environment::Test,
            _ => Environment::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_test(&self) -> bool {
        matches!(self.environment, Environment::Test)
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }
}

pub fn load_config() -> Result<AppConfig, AppError> {
    AppConfig::from_env()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        self == &Environment::Production
    }
}
impl FromStr for Environment {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            "test" | "testing" => Ok(Self::Test),
            _ => Err(format!("Invalid environment: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    #[default]
    Smtp,
    Mailtrap,
}

impl FromStr for EmailProvider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "smtp" | "lettre" => Ok(Self::Smtp),
            "mailtrap" | "api" => Ok(Self::Mailtrap),
            _ => Err(format!("Invalid email provider: {value}")),
        }
    }
}

impl std::fmt::Display for EmailProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smtp => f.write_str("smtp"),
            Self::Mailtrap => f.write_str("mailtrap"),
        }
    }
}

/// `none` is plaintext and only valid outside production; it exists so local
/// development can point at a plain SMTP sink such as maildev.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmtpTlsMode {
    /// No TLS at all. Rejected when `ENVIRONMENT=production`.
    None,

    /// Upgrade with STARTTLS, failing if the server does not support it.
    #[default]
    Starttls,

    /// Upgrade with STARTTLS when offered, silently falling back to plaintext.
    Opportunistic,

    /// Implicit TLS (SMTPS) from the first byte.
    Implicit,
}

impl FromStr for SmtpTlsMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "none" | "plain" | "plaintext" => Ok(Self::None),
            "starttls" | "required" => Ok(Self::Starttls),
            "opportunistic" => Ok(Self::Opportunistic),
            "implicit" | "wrapper" | "tls" => Ok(Self::Implicit),
            _ => Err(format!("Invalid SMTP TLS mode: {value}")),
        }
    }
}

fn default_smtp_tls_mode() -> SmtpTlsMode {
    SmtpTlsMode::default()
}

fn default_email_provider() -> EmailProvider {
    EmailProvider::default()
}

fn default_email_from_address() -> String {
    "accounts@asterisk.app".to_string()
}

fn default_email_from_name() -> String {
    "asterisk Accounts".to_string()
}

fn default_mailtrap_base_url() -> String {
    "https://send.api.mailtrap.io/api/send".to_string()
}

fn extract_optional_env<T: FromStr>(env_key: &str) -> Option<T> {
    std::env::var(env_key).ok().and_then(|raw| raw.parse().ok())
}

fn default_port() -> u16 {
    8080
}

fn default_environment() -> Environment {
    Environment::Development
}

fn default_body_limit_megabytes() -> usize {
    10
}

fn default_upload_path() -> String {
    "/tmp/upload".into()
}

fn default_imagekit_upload_url() -> String {
    "https://upload.imagekit.io/api/v1/files/upload".into()
}

fn default_export_path() -> String {
    "/tmp/export".into()
}

fn default_allowed_origins() -> Vec<String> {
    vec![
        "http://localhost:3000".into(),
        "http://localhost:5173".into(),
    ]
}

fn default_requests_time_out() -> Duration {
    Duration::from_secs(10)
}

fn default_rate_limit_requests_per_second() -> u32 {
    5
}

fn default_rate_limit_burst() -> u32 {
    10
}

fn default_rate_limit_requests_per_minute() -> u32 {
    50
}

fn default_rate_limit_max_keys() -> usize {
    100_000
}

fn default_rate_limit_auth_per_minute() -> u32 {
    5
}

fn default_rate_limit_auth_per_hour() -> u32 {
    30
}

fn default_rate_limit_otp_per_minute() -> u32 {
    2
}

fn default_rate_limit_otp_per_hour() -> u32 {
    10
}

pub const CACHE_TTL_SECS: u64 = 10 * 60;

pub fn default_cache_ttl() -> u64 {
    CACHE_TTL_SECS
}

fn default_frontend_url() -> String {
    match AppConfig::current_env() {
        Environment::Production => "https://asterisk.app".into(),
        Environment::Development | Environment::Test => "http://localhost:7008".into(),
    }
}

fn default_paystack_verify_base_url() -> String {
    "https://api.paystack.co/transaction/verify".into()
}

fn default_console_url() -> String {
    match AppConfig::current_env() {
        Environment::Production => "https://backoffice-rho-ten.vercel.app/".into(),
        Environment::Development | Environment::Test => "http://localhost:5600".into(),
    }
}
