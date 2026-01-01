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
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                // Preserve original io::Error context instead of constructing a new one.
                Err(e).context("Client disconnected")
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_message(&mut self, msg: &Value) -> Result<()> {
        let s = serde_json::to_string(msg)?;

        // Write JSON, newline and flush while mapping IO errors consistently.
        // Use instance helper to preserve error context and reduce duplication.
        Self::check_io(self.writer.write_all(s.as_bytes()).await)?;
        Self::check_io(self.writer.write_all(b"\n").await)?;
        Self::check_io(self.writer.flush().await)?;

        Ok(())
    }
}

// Helper to inspect anyhow::Error for BrokenPipe without duplicating downcast logic.
fn is_broken_pipe_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<std::io::Error>()
        .map(|ioe| ioe.kind() == std::io::ErrorKind::BrokenPipe)
        .unwrap_or(false)
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

                       // Prepare response based on method.
                       let response = if method == "initialize" {
                           // We do not advertise non-implemented capabilities to avoid misleading clients.
                           let result = json!({
                               "protocolVersion": "2024-11-05",
                               "serverInfo": {"name": "evefrontier", "version": env!("CARGO_PKG_VERSION")},
                               // No tools/resources/prompts are currently advertised here.
                               "capabilities": {}
                           });

                           json!({"jsonrpc": "2.0", "id": id, "result": result})
                       } else if method == "tools/list" {
                           // Return a simple description of available tools.
                           let result = json!({
                               "tools": [
                                   {"name": "route_plan", "description": "Plan a route between two solar systems."},
                                   {"name": "system_info", "description": "Get information about a solar system."},
                                   {"name": "systems_nearby", "description": "List systems within a number of jumps."},
                                   {"name": "gates_from", "description": "List outbound stargates from a system."}
                               ]
                           });

                           json!({"jsonrpc": "2.0", "id": id, "result": result})
                       } else if method == "tools/call" {
                           let params = val.get("params").cloned().unwrap_or_else(|| json!({}));
                           let (tool_name_opt, _arguments_opt) = match params {
                               Value::Object(ref obj) => {
                                   let name = obj.get("name").and_then(|v| v.as_str());
                                   let arguments = obj.get("arguments").cloned().unwrap_or_else(|| json!({}));
                                   (name, Some(arguments))
                               }
                               _ => (None, None),
                           };

                           if tool_name_opt.is_none() {
                               let error = json!({"code": -32602, "message": "Invalid params for tools/call: expected object with string field 'name'."});
                               json!({"jsonrpc": "2.0", "id": id, "error": error})
                           } else {
                               let tool_name = tool_name_opt.unwrap();
                               let known_tool = matches!(tool_name, "route_plan" | "system_info" | "systems_nearby" | "gates_from");

                               if !known_tool {
                                   let error = json!({"code": -32601, "message": format!("Unknown tool: {}", tool_name)});
                                   json!({"jsonrpc": "2.0", "id": id, "error": error})
                               } else {
                                   // Known but not implemented yet - fail securely with an explicit error code.
                                   let error = json!({"code": -32001, "message": format!("Tool '{}' is not yet implemented on the server.", tool_name)});
                                   json!({"jsonrpc": "2.0", "id": id, "error": error})
                               }
                           }
                       } else if method == "resources/list" {
                           let result = json!({"resources": []});
                           json!({"jsonrpc": "2.0", "id": id, "result": result})
                       } else if method == "resources/read" {
                           let has_uri = match val.get("params") {
                               Some(Value::Object(ref obj)) => obj.get("uri").and_then(|v| v.as_str()).is_some(),
                               _ => false,
                           };
                           if !has_uri {
                               let error = json!({"code": -32602, "message": "Invalid params for resources/read: expected object with string field 'uri'."});
                               json!({"jsonrpc": "2.0", "id": id, "error": error})
                           } else {
                               let error = json!({"code": -32002, "message": "Resources are not implemented on this server."});
                               json!({"jsonrpc": "2.0", "id": id, "error": error})
                           }
                       } else {
                           let error = json!({"code": -32601, "message": format!("Unknown method: {}", method)});
                           json!({"jsonrpc": "2.0", "id": id, "error": error})
                       };

                       if let Err(e) = transport.write_message(&response).await {
                           if is_broken_pipe_error(&e) {
                               tracing::info!("Client disconnected (broken pipe)");
                               break;
                           }
                           return Err(e);
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

    fn initialize_server_from_target(target: Option<&PathBuf>) -> Result<McpServerState> {
        if let Some(p) = target {
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
    }

    let server = initialize_server_from_target(target.as_ref())?;

    // 3. Create transport and run server loop
    let transport = StdioTransport::new();
    run_server_loop(transport, server).await
}
