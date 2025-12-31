use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - MUST redirect to stderr to avoid stdout protocol corruption
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("evefrontier_mcp=info".parse()?),
        )
        .init();

    info!("MCP Server initialized (stub)");
    info!("This is placeholder code to be replaced in Phase 2");

    Ok(())
}
