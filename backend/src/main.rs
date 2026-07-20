mod config;
mod db;
mod http;

use anyhow::Context;
use config::Settings;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "farminsight_backend=info,tower_http=info".into()),
        )
        .init();

    let settings = Settings::from_env()?;
    settings.log_summary();
    let db = db::connect(&settings).await?;
    db::migrate(&db).await?;

    let app = http::router(settings.frontend_dist_dir.clone(), db);
    let bind_addr = settings.bind_addr();
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind server to {bind_addr}"))?;

    info!(
        address = %listener.local_addr().context("failed to read local listener address")?,
        frontend_dist_dir = %settings.frontend_dist_dir.display(),
        "server listening"
    );

    axum::serve(listener, app)
        .await
        .context("server terminated unexpectedly")
}
