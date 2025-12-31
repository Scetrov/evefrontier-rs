use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use tracing::{error, info};

use evefrontier_mcp::resources::{
    AlgorithmsResource, DatasetInfoResource, SpatialIndexStatusResource,
};
use evefrontier_mcp::server::McpServerState;
use evefrontier_mcp::types::{GatesFromInput, RoutePlanInput, SystemInfoInput, SystemsNearbyInput};

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcError {
    fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: msg.into(),
            data: None,
        }
    }

    fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }

    fn method_not_found(msg: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: msg.into(),
            data: None,
        }
    }

    fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - MUST redirect to stderr to avoid stdout protocol corruption
    // Disable ANSI colors to avoid escape codes in MCP client logs
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("evefrontier_mcp=info".parse()?),
        )
        .init();

    info!("Initializing EVE Frontier MCP server...");

    // Load dataset and spatial index
    let state = match McpServerState::new() {
        Ok(s) => {
            info!(
                "Server state loaded: {} systems",
                s.dataset_info().system_count
            );
            s
        }
        Err(e) => {
            error!("Failed to initialize server state: {}", e);
            return Err(e.into());
        }
    };

    info!("Starting MCP server on stdio transport...");

    // Process JSON-RPC messages from stdin
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to read line from stdin: {}", e);
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let response = handle_request(&line, &state).await;
        let response_json = serde_json::to_string(&response)?;

        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    info!("MCP server shutting down");
    Ok(())
}

async fn handle_request(line: &str, state: &McpServerState) -> JsonRpcResponse {
    // Parse JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(JsonRpcError::parse_error(format!(
                    "Invalid JSON-RPC request: {}",
                    e
                ))),
            };
        }
    };

    if request.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError::invalid_request("Only JSON-RPC 2.0 supported")),
        };
    }

    // Handle MCP protocol methods
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(state).await,
        "tools/list" => handle_tools_list(state).await,
        "tools/call" => handle_tools_call(&request.params, state).await,
        "resources/list" => handle_resources_list(state).await,
        "resources/read" => handle_resources_read(&request.params, state).await,
        "prompts/list" => handle_prompts_list().await,
        "prompts/get" => handle_prompts_get(&request.params).await,
        _ => Err(JsonRpcError::method_not_found(format!(
            "Unknown method: {}",
            request.method
        ))),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    }
}

async fn handle_initialize(_state: &McpServerState) -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "evefrontier",
            "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        }
    }))
}

async fn handle_tools_list(_state: &McpServerState) -> Result<Value, JsonRpcError> {
    // Return static tool descriptors
    let tools = vec![
        serde_json::json!({
            "name": "route_plan",
            "description": "Plan a route between two star systems with optional constraints",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "origin": { "type": "string", "description": "Starting system name" },
                    "destination": { "type": "string", "description": "Destination system name" },
                    "algorithm": { "type": "string", "enum": ["bfs", "dijkstra", "a-star"], "description": "Routing algorithm" },
                    "max_jump": { "type": "number", "description": "Maximum jump distance in light years" },
                    "max_temperature": { "type": "number", "description": "Maximum system temperature in Kelvin" },
                    "avoid_systems": { "type": "array", "items": { "type": "string" }, "description": "Systems to avoid" },
                    "avoid_gates": { "type": "boolean", "description": "Avoid jump gates" }
                },
                "required": ["origin", "destination"]
            }
        }),
        serde_json::json!({
            "name": "system_info",
            "description": "Get detailed information about a star system",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "system_name": { "type": "string", "description": "System name to query" }
                },
                "required": ["system_name"]
            }
        }),
        serde_json::json!({
            "name": "systems_nearby",
            "description": "Find star systems within a radius",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "origin": { "type": "string", "description": "Center system name" },
                    "radius": { "type": "number", "description": "Search radius in light years" },
                    "max_temperature": { "type": "number", "description": "Maximum system temperature in Kelvin" }
                },
                "required": ["origin", "radius"]
            }
        }),
        serde_json::json!({
            "name": "gates_from",
            "description": "Get jump gate connections from a system",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "system_name": { "type": "string", "description": "System name to query" }
                },
                "required": ["system_name"]
            }
        }),
    ];
    Ok(serde_json::json!({ "tools": tools }))
}

async fn handle_tools_call(
    params: &Option<Value>,
    _state: &McpServerState,
) -> Result<Value, JsonRpcError> {
    let params = params
        .as_ref()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?;

    let tool_name = params["name"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing tool name"))?;

    let arguments = &params["arguments"];

    let result = match tool_name {
        "route_plan" => {
            let input: RoutePlanInput = serde_json::from_value(arguments.clone())
                .map_err(|e| JsonRpcError::invalid_params(format!("Invalid input: {}", e)))?;

            // Validation happens inside tool execute()
            let output = evefrontier_mcp::tools::RoutePlanTool::execute(input)
                .await
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

            serde_json::to_value(output).map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        }
        "system_info" => {
            let input: SystemInfoInput = serde_json::from_value(arguments.clone())
                .map_err(|e| JsonRpcError::invalid_params(format!("Invalid input: {}", e)))?;

            // Validation happens inside tool execute()
            let output = evefrontier_mcp::tools::SystemInfoTool::execute(input)
                .await
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

            serde_json::to_value(output).map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        }
        "systems_nearby" => {
            let input: SystemsNearbyInput = serde_json::from_value(arguments.clone())
                .map_err(|e| JsonRpcError::invalid_params(format!("Invalid input: {}", e)))?;

            // Validation happens inside tool execute()
            let output = evefrontier_mcp::tools::SystemsNearbyTool::execute(input)
                .await
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

            serde_json::to_value(output).map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        }
        "gates_from" => {
            let input: GatesFromInput = serde_json::from_value(arguments.clone())
                .map_err(|e| JsonRpcError::invalid_params(format!("Invalid input: {}", e)))?;

            // Validation happens inside tool execute()
            let output = evefrontier_mcp::tools::GatesFromTool::execute(input)
                .await
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

            serde_json::to_value(output).map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        }
        _ => {
            return Err(JsonRpcError::method_not_found(format!(
                "Unknown tool: {}",
                tool_name
            )))
        }
    };

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result)
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        }]
    }))
}

async fn handle_resources_list(state: &McpServerState) -> Result<Value, JsonRpcError> {
    let resources = state.resources();
    Ok(serde_json::json!({ "resources": resources }))
}

async fn handle_resources_read(
    params: &Option<Value>,
    state: &McpServerState,
) -> Result<Value, JsonRpcError> {
    let params = params
        .as_ref()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?;

    let uri = params["uri"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing resource URI"))?;

    let content = match uri {
        "evefrontier://dataset/info" => DatasetInfoResource::read(state)
            .await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?,
        "evefrontier://algorithms" => AlgorithmsResource::read()
            .await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?,
        "evefrontier://spatial-index/status" => SpatialIndexStatusResource::read(state)
            .await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?,
        _ => {
            return Err(JsonRpcError::invalid_params(format!(
                "Unknown resource URI: {}",
                uri
            )))
        }
    };

    Ok(serde_json::json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": content
        }]
    }))
}

async fn handle_prompts_list() -> Result<Value, JsonRpcError> {
    let prompts = evefrontier_mcp::prompts::list_prompts()
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(serde_json::json!({ "prompts": prompts }))
}

async fn handle_prompts_get(params: &Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params
        .as_ref()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?;

    let name = params["name"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing prompt name"))?;

    let arguments = params["arguments"].clone();

    let prompt_text = evefrontier_mcp::prompts::get_prompt(name, &arguments)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(serde_json::json!({
        "description": format!("Prompt template: {}", name),
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": prompt_text
                }
            }
        ]
    }))
}
