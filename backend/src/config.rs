use std::{env, path::PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub frontend_dist_dir: PathBuf,
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

        Ok(Self {
            host,
            port,
            frontend_dist_dir,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
