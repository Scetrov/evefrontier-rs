use anyhow::{Context, Result};
use evefrontier_lib::{ensure_dataset, DatasetRelease};
use evefrontier_mcp::server::McpServerState;
use serde_json::json;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::select;
use tokio::signal;

use crate::GlobalOptions;

/// Configure tracing to write only to stderr.
pub fn configure_tracing(log_level: Option<&str>) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let env_filter = if let Some(level) = log_level {
        EnvFilter::new(level)
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let subscriber = fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    Ok(())
}

/// Resolve dataset path from CLI globals and env vars.
pub fn resolve_dataset_path(global: &GlobalOptions) -> Result<Option<PathBuf>> {
    if let Some(path) = global.data_dir.clone() {
        return Ok(Some(path));
    }

    if let Ok(env_path) = std::env::var("EVEFRONTIER_DATA_DIR") {
        return Ok(Some(PathBuf::from(env_path)));
    }

    Ok(None)
}

/// Stdio transport using tokio async I/O.
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    // Using Stdout directly for now
    writer: tokio::io::Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: stdout(),
        }
    }

    /// Read a single JSON-RPC message. Returns Ok(None) on EOF.
    pub async fn read_message(&mut self) -> Result<Option<Value>> {
        let mut line = String::new();
        let bytes = self
            .reader
            .read_line(&mut line)
            .await
            .context("failed to read line")?;
        if bytes == 0 {
            return Ok(None);
        }
        let v: Value = serde_json::from_str(line.trim_end()).context("invalid json")?;
        Ok(Some(v))
    }

    // Helper to map std::io::Result into anyhow::Result while preserving BrokenPipe as io::Error
    fn check_io<T>(res: std::io::Result<T>) -> Result<T> {
        match res {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Client disconnected",
            )
            .into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_message(&mut self, msg: &Value) -> Result<()> {
        let s = serde_json::to_string(msg)?;

        // Write JSON, newline and flush while mapping IO errors consistently
        Self::check_io(self.writer.write_all(s.as_bytes()).await)?;
        Self::check_io(self.writer.write_all(b"\n").await)?;
        Self::check_io(self.writer.flush().await)?;

        Ok(())
    }
}

/// Run the server loop: read messages from stdin and respond on stdout.
pub async fn run_server_loop(mut transport: StdioTransport, server: McpServerState) -> Result<()> {
    tracing::info!("MCP server initialized, waiting for requests...");

    // Initialize server state (warm up caches)
    server
        .initialize()
        .await
        .context("failed to initialize MCP server state")?;

    loop {
        select! {
           _ = signal::ctrl_c() => {
               tracing::info!("Received shutdown signal, exiting gracefully");
               break;
           }

           msg = transport.read_message() => {
               match msg {
                   Ok(Some(val)) => {
                        // Expect object with jsonrpc, id, method
                        let method = val.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        let id = val.get("id").cloned();

                       if method == "initialize" {
                           let result = json!({
                               "protocolVersion": "2024-11-05",
                               "serverInfo": {"name":"evefrontier", "version": env!("CARGO_PKG_VERSION")},
                               "capabilities": {"tools": {}, "resources": {}, "prompts": {} }
                           });

                           let response = json!({"jsonrpc": "2.0", "id": id, "result": result});
                           if let Err(e) = transport.write_message(&response).await {
                               // Prefer inspecting the underlying IO error instead of string matching
                               if e.downcast_ref::<std::io::Error>().map(|ioe| ioe.kind() == std::io::ErrorKind::BrokenPipe).unwrap_or(false) {
                                   tracing::info!("Client disconnected (broken pipe)");
                                   break;
                               }
                               return Err(e);
                           }
                       } else {
                           // Method not found
                           let error = json!({"code": -32601, "message": format!("Unknown method: {}", method)});
                           let response = json!({"jsonrpc": "2.0", "id": id, "error": error});
                           if let Err(e) = transport.write_message(&response).await {
                               if e.downcast_ref::<std::io::Error>().map(|ioe| ioe.kind() == std::io::ErrorKind::BrokenPipe).unwrap_or(false) {
                                   tracing::info!("Client disconnected (broken pipe)");
                                   break;
                               }
                               return Err(e);
                           }
                       }
                   }
                   Ok(None) => {
                       tracing::info!("Client disconnected (EOF)");
                       break;
                   }
                   Err(e) => {
                       tracing::error!("Transport error: {}", e);
                       return Err(e);
                   }
                }
            }
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

/// Public entrypoint orchestrating the MCP server lifecycle
pub async fn run_mcp_server(global: &GlobalOptions, log_level: Option<&str>) -> Result<()> {
    // 1. Configure tracing
    configure_tracing(log_level)?;

    // 2. Resolve dataset path and initialize the MCP server (deduplicated)
    let target = resolve_dataset_path(global)?;
    tracing::info!("Resolving dataset path...");

    let server = (|| -> Result<McpServerState> {
        if let Some(ref p) = target {
            if p.exists() {
                tracing::info!("Using explicit dataset path {}", p.display());
                return McpServerState::with_path(p)
                    .context("Failed to initialize MCP server state from database");
            }
            tracing::info!(
                "Dataset not found at {}, attempting to ensure/download",
                p.display()
            );
            let paths = ensure_dataset(Some(p), DatasetRelease::latest())
                .context("Failed to locate or download dataset")?;
            let start = Instant::now();
            let s = McpServerState::with_path(&paths.database)
                .context("Failed to initialize MCP server state from database")?;
            tracing::info!("Dataset loaded in {:?}", start.elapsed());
            return Ok(s);
        }

        let paths = ensure_dataset(None, DatasetRelease::latest())
            .context("Failed to locate or download dataset")?;
        let start = Instant::now();
        let s = McpServerState::with_path(&paths.database)
            .context("Failed to initialize MCP server state from database")?;
        tracing::info!("Dataset loaded in {:?}", start.elapsed());
        Ok(s)
    })()?;

    // 3. Create transport and run server loop
    let transport = StdioTransport::new();
    run_server_loop(transport, server).await
}
