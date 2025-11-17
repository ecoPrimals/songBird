//! Execution agent binary

use songbird_execution_agent::{init_agent, AgentConfig};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "songbird_execution_agent=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Songbird Execution Agent starting...");

    // Load configuration (for now, use defaults - can add config file support later)
    let config = AgentConfig::default();

    // Initialize and start the agent
    let server = init_agent(config).await?;
    server.serve().await?;

    Ok(())
}
