use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod db;
mod engines;
mod xdotcom;
mod utils;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_env_filter("info")
        .pretty()
        .init();

    // Load unified configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize database connection with config
    let db_pool = db::establish_connection(&config).await?;
    info!("Database connection established");

    // Run migrations
    db::run_migrations(&db_pool).await?;
    info!("Database migrations completed");

    // Initialize services with config
    let twitter_client = xdotcom::Client::new(&config)?;
    let openai_client = engines::ai::Client::new(&config)?;
    let eth_client = engines::wallet::Client::new(&config)?;

    info!("Starting nousflash agent...");
    
    // Main loop
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received, cleaning up...");
                break;
            }
            result = run_pipeline(
                &config,
                &db_pool,
                &twitter_client,
                &openai_client,
                &eth_client,
            ) => {
                if let Err(e) = result {
                    tracing::error!("Pipeline error: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn run_pipeline(
    config: &Config,
    pool: &db::Pool,
    twitter: &xdotcom::Client,
    ai: &engines::ai::Client,
    eth: &engines::wallet::Client,
) -> Result<()> {
    // Pipeline implementation
    todo!("Implement main agent pipeline")
}