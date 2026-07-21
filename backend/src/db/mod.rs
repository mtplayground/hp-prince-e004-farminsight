use std::{path::Path, str::FromStr, time::Duration};

use anyhow::{Context, Result};
use sqlx::{
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

use crate::config::{DatabaseSslMode, Settings};

pub async fn connect(settings: &Settings) -> Result<PgPool> {
    let mut options = PgConnectOptions::from_str(&settings.database_url)
        .context("DATABASE_URL must be a valid PostgreSQL connection string")?
        .statement_cache_capacity(0);
    if let Some(database_ssl_mode) = settings.database_ssl_mode {
        options = options.ssl_mode(pg_ssl_mode(database_ssl_mode));
    }

    PgPoolOptions::new()
        .max_connections(settings.database_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
        .context("failed to connect to PostgreSQL")
}

pub async fn migrate(pool: &PgPool, migrations_dir: &Path) -> Result<()> {
    let migrator = Migrator::new(migrations_dir)
        .await
        .context("failed to load PostgreSQL migrations")?;

    migrator
        .run(pool)
        .await
        .context("failed to run PostgreSQL migrations")
}

fn pg_ssl_mode(mode: DatabaseSslMode) -> PgSslMode {
    match mode {
        DatabaseSslMode::Disable => PgSslMode::Disable,
        DatabaseSslMode::Prefer => PgSslMode::Prefer,
        DatabaseSslMode::Require => PgSslMode::Require,
    }
}
