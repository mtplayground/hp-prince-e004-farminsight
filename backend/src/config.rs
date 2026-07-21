use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};

#[derive(Clone)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub frontend_dist_dir: PathBuf,
    pub migrations_dir: PathBuf,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_ssl_mode: Option<DatabaseSslMode>,
    pub object_storage: Option<ObjectStorageSettings>,
    pub auth: Option<AuthSettings>,
    pub legacy_jwt_secret: Option<String>,
    pub email: Option<EmailSettings>,
    pub self_url: Option<String>,
    pub allowed_cors_origin: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum DatabaseSslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Clone)]
pub struct ObjectStorageSettings {
    pub endpoint_url: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub prefix: String,
}

#[derive(Clone)]
pub struct AuthSettings {
    pub auth_url: String,
    pub app_token: String,
    pub jwks_url: String,
}

#[derive(Clone)]
pub struct EmailSettings {
    pub email_url: String,
    pub app_token: String,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()
            .context("PORT must be a valid TCP port")?;
        let frontend_dist_dir = env::var("FRONTEND_DIST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("frontend/dist"));
        let migrations_dir = env::var("MIGRATIONS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("backend/migrations"));
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_owned())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a positive integer")?;
        if database_max_connections == 0 {
            bail!("DATABASE_MAX_CONNECTIONS must be greater than zero");
        }
        let database_ssl_mode = DatabaseSslMode::from_env()?;
        let object_storage = ObjectStorageSettings::from_env()?;
        let auth = AuthSettings::from_env()?;
        let legacy_jwt_secret = optional_var("JWT_SECRET")?;
        let email = EmailSettings::from_env()?;
        let self_url = optional_var("SELF_URL")?;
        let allowed_cors_origin = optional_var("ALLOWED_CORS_ORIGIN")?;

        Ok(Self {
            host,
            port,
            frontend_dist_dir,
            migrations_dir,
            database_url,
            database_max_connections,
            database_ssl_mode,
            object_storage,
            auth,
            legacy_jwt_secret,
            email,
            self_url,
            allowed_cors_origin,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn log_summary(&self) {
        let object_storage = self.object_storage.as_ref();
        let auth = self.auth.as_ref();
        let email = self.email.as_ref();

        tracing::info!(
            host = %self.host,
            port = self.port,
            frontend_dist_dir = %self.frontend_dist_dir.display(),
            migrations_dir = %self.migrations_dir.display(),
            database_configured = !self.database_url.is_empty(),
            database_max_connections = self.database_max_connections,
            database_ssl_mode_override = self.database_ssl_mode.is_some(),
            object_storage_configured = object_storage.is_some(),
            object_storage_endpoint_configured = object_storage
                .map(|settings| !settings.endpoint_url.is_empty())
                .unwrap_or(false),
            object_storage_region_configured = object_storage
                .map(|settings| !settings.region.is_empty())
                .unwrap_or(false),
            object_storage_bucket_configured = object_storage
                .map(|settings| !settings.bucket.is_empty())
                .unwrap_or(false),
            object_storage_access_key_configured = object_storage
                .map(|settings| !settings.access_key_id.is_empty())
                .unwrap_or(false),
            object_storage_secret_key_configured = object_storage
                .map(|settings| !settings.secret_access_key.is_empty())
                .unwrap_or(false),
            object_storage_prefix_configured = object_storage
                .map(|settings| !settings.prefix.is_empty())
                .unwrap_or(false),
            auth_configured = auth.is_some(),
            auth_url_configured = auth
                .map(|settings| !settings.auth_url.is_empty())
                .unwrap_or(false),
            auth_app_token_configured = auth
                .map(|settings| !settings.app_token.is_empty())
                .unwrap_or(false),
            auth_jwks_url_configured = auth
                .map(|settings| !settings.jwks_url.is_empty())
                .unwrap_or(false),
            legacy_jwt_secret_configured = self.legacy_jwt_secret.is_some(),
            email_configured = email.is_some(),
            email_url_configured = email
                .map(|settings| !settings.email_url.is_empty())
                .unwrap_or(false),
            email_app_token_configured = email
                .map(|settings| !settings.app_token.is_empty())
                .unwrap_or(false),
            self_url_configured = self.self_url.is_some(),
            allowed_cors_origin_configured = self.allowed_cors_origin.is_some(),
            "configuration loaded"
        );
    }
}

impl DatabaseSslMode {
    fn from_env() -> Result<Option<Self>> {
        let Ok(value) = env::var("DATABASE_SSL_MODE") else {
            return Ok(None);
        };

        match value.to_ascii_lowercase().as_str() {
            "disable" => Ok(Some(Self::Disable)),
            "prefer" => Ok(Some(Self::Prefer)),
            "require" => Ok(Some(Self::Require)),
            value => bail!(
                "DATABASE_SSL_MODE must be one of disable, prefer, or require; got {value}"
            ),
        }
    }
}

impl ObjectStorageSettings {
    fn from_env() -> Result<Option<Self>> {
        let endpoint_url = first_present(&["OBJECT_STORAGE_ENDPOINT_URL", "AWS_ENDPOINT_URL_S3"])?;
        let region = first_present(&["OBJECT_STORAGE_REGION", "AWS_REGION"])?;
        let bucket = first_present(&["OBJECT_STORAGE_BUCKET", "AWS_BUCKET", "S3_BUCKET"])?;
        let access_key_id =
            first_present(&["OBJECT_STORAGE_ACCESS_KEY_ID", "AWS_ACCESS_KEY_ID"])?;
        let secret_access_key =
            first_present(&["OBJECT_STORAGE_SECRET_ACCESS_KEY", "AWS_SECRET_ACCESS_KEY"])?;
        let prefix = optional_var("OBJECT_STORAGE_PREFIX")?.unwrap_or_default();

        if [
            endpoint_url.as_ref(),
            region.as_ref(),
            bucket.as_ref(),
            access_key_id.as_ref(),
            secret_access_key.as_ref(),
        ]
        .iter()
        .all(|value| value.is_none())
            && prefix.is_empty()
        {
            return Ok(None);
        }

        Ok(Some(Self {
            endpoint_url: require_group_value(
                "object storage",
                "OBJECT_STORAGE_ENDPOINT_URL or AWS_ENDPOINT_URL_S3",
                endpoint_url,
            )?,
            region: require_group_value(
                "object storage",
                "OBJECT_STORAGE_REGION or AWS_REGION",
                region,
            )?,
            bucket: require_group_value(
                "object storage",
                "OBJECT_STORAGE_BUCKET, AWS_BUCKET, or S3_BUCKET",
                bucket,
            )?,
            access_key_id: require_group_value(
                "object storage",
                "OBJECT_STORAGE_ACCESS_KEY_ID or AWS_ACCESS_KEY_ID",
                access_key_id,
            )?,
            secret_access_key: require_group_value(
                "object storage",
                "OBJECT_STORAGE_SECRET_ACCESS_KEY or AWS_SECRET_ACCESS_KEY",
                secret_access_key,
            )?,
            prefix,
        }))
    }
}

impl AuthSettings {
    fn from_env() -> Result<Option<Self>> {
        let auth_url = optional_var("MCTAI_AUTH_URL")?;
        let app_token = optional_var("MCTAI_AUTH_APP_TOKEN")?;
        let jwks_url = optional_var("MCTAI_AUTH_JWKS_URL")?;

        if [&auth_url, &app_token, &jwks_url]
            .iter()
            .all(|value| value.is_none())
        {
            return Ok(None);
        }

        Ok(Some(Self {
            auth_url: require_group_value("auth", "MCTAI_AUTH_URL", auth_url)?,
            app_token: require_group_value("auth", "MCTAI_AUTH_APP_TOKEN", app_token)?,
            jwks_url: require_group_value("auth", "MCTAI_AUTH_JWKS_URL", jwks_url)?,
        }))
    }
}

impl EmailSettings {
    fn from_env() -> Result<Option<Self>> {
        let email_url = optional_var("MCTAI_EMAIL_URL")?;
        let app_token = optional_var("MCTAI_EMAIL_APP_TOKEN")?;

        if [&email_url, &app_token]
            .iter()
            .all(|value| value.is_none())
        {
            return Ok(None);
        }

        Ok(Some(Self {
            email_url: require_group_value("email", "MCTAI_EMAIL_URL", email_url)?,
            app_token: require_group_value("email", "MCTAI_EMAIL_APP_TOKEN", app_token)?,
        }))
    }
}

fn optional_var(key: &str) -> Result<Option<String>> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => bail!("{key} cannot be empty when set"),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error).with_context(|| format!("{key} must contain valid Unicode")),
    }
}

fn first_present(keys: &[&str]) -> Result<Option<String>> {
    for key in keys {
        if let Some(value) = optional_var(key)? {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

fn require_group_value(group: &str, key: &str, value: Option<String>) -> Result<String> {
    value.with_context(|| {
        format!("{group} configuration requires {key} when any {group} variable is set")
    })
}
