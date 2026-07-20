use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub frontend_dist_dir: PathBuf,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_ssl_mode: Option<DatabaseSslMode>,
}

#[derive(Debug, Clone, Copy)]
pub enum DatabaseSslMode {
    Disable,
    Prefer,
    Require,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()
            .context("PORT must be a valid TCP port")?;
        let frontend_dist_dir = env::var("FRONTEND_DIST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("frontend/dist"));
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_owned())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a positive integer")?;
        if database_max_connections == 0 {
            bail!("DATABASE_MAX_CONNECTIONS must be greater than zero");
        }
        let database_ssl_mode = DatabaseSslMode::from_env()?;

        Ok(Self {
            host,
            port,
            frontend_dist_dir,
            database_url,
            database_max_connections,
            database_ssl_mode,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
