//! Integration tests for MCP server JSON-RPC protocol
//!
//! These tests verify the MCP server's JSON-RPC 2.0 protocol implementation
//! by spawning the server binary and sending actual requests over stdin.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

/// Helper to spawn MCP server process
fn spawn_server() -> std::io::Result<Child> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("evefrontier-mcp")
        .arg("--")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // Suppress logging output
        .current_dir(workspace_root)
        .spawn()
}

/// Send a JSON-RPC request and parse the response
fn send_request(child: &mut Child, request: Value) -> std::io::Result<Value> {
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send request
    writeln!(stdin, "{}", request)?;
    stdin.flush()?;

    // Read response
    let mut line = String::new();
    reader.read_line(&mut line)?;

    serde_json::from_str(&line).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse JSON response: {}", e),
        )
    })
}

#[test]
fn test_initialize_protocol() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "evefrontier");
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert!(response["result"]["capabilities"]["resources"].is_object());

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_tools_list() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4);

    let tool_names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"route_plan"));
    assert!(tool_names.contains(&"system_info"));
    assert!(tool_names.contains(&"systems_nearby"));
    assert!(tool_names.contains(&"gates_from"));

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_resources_list() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list",
        "params": {}
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["result"]["resources"].is_array());

    let resources = response["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 3);

    let uris: Vec<_> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"evefrontier://dataset/info"));
    assert!(uris.contains(&"evefrontier://algorithms"));
    assert!(uris.contains(&"evefrontier://spatial-index/status"));

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_tools_call_route_plan_stub() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "route_plan",
            "arguments": {
                "origin": "Nod",
                "destination": "Brana"
            }
        }
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 4);
    assert!(response["result"].is_object());

    // Stub returns NOT_IMPLEMENTED error
    let content = &response["result"]["content"];
    assert!(content.is_array());
    let text = content[0]["text"].as_str().unwrap();
    let output: Value = serde_json::from_str(text).unwrap();
    assert_eq!(output["success"], false);
    assert_eq!(output["error"]["code"], "NOT_IMPLEMENTED");

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_tools_call_invalid_tool() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 5);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // Method not found
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown tool"));

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_invalid_json_rpc_version() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "1.0",
        "id": 6,
        "method": "initialize",
        "params": {}
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 6);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32600); // Invalid request

    server.kill().ok();
    server.wait().ok();
    server.wait().ok();
}

#[test]
fn test_parse_error() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send malformed JSON
    writeln!(stdin, "{{{{not valid json}}}}").unwrap();
    stdin.flush().unwrap();

    // Read response
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    let response: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // Parse error

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_resources_read_dataset_info() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "resources/read",
        "params": {
            "uri": "evefrontier://dataset/info"
        }
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 7);
    assert!(response["result"]["contents"].is_array());

    let contents = response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["mimeType"], "application/json");

    let text = contents[0]["text"].as_str().unwrap();
    let dataset_info: Value = serde_json::from_str(text).unwrap();
    assert!(dataset_info["system_count"].is_number());
    assert!(dataset_info["gate_count"].is_number());

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_resources_read_algorithms() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "resources/read",
        "params": {
            "uri": "evefrontier://algorithms"
        }
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 8);
    assert!(response["result"]["contents"].is_array());

    let contents = response["result"]["contents"].as_array().unwrap();
    let text = contents[0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).unwrap();

    assert!(data["algorithms"].is_array());
    assert_eq!(data["default"], "a-star");

    let algorithms = data["algorithms"].as_array().unwrap();
    assert_eq!(algorithms.len(), 3);

    let names: Vec<_> = algorithms
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"bfs"));
    assert!(names.contains(&"dijkstra"));
    assert!(names.contains(&"a-star"));

    server.kill().ok();
    server.wait().ok();
}

#[test]
fn test_prompts_list_empty() {
    let mut server = spawn_server().expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "prompts/list",
        "params": {}
    });

    let response = send_request(&mut server, request).expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 9);
    assert!(response["result"]["prompts"].is_array());
    assert_eq!(response["result"]["prompts"].as_array().unwrap().len(), 0);

    server.kill().ok();
    server.wait().ok();
}
