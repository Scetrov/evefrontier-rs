use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn spawn_server() -> std::io::Result<Child> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .expect("CARGO_MANIFEST_DIR should have a parent")
        .parent()
        .expect("workspace should be two levels up from manifest dir");

    let exe = workspace_root.join("target/debug/evefrontier-cli");
    Command::new(exe)
        .arg("mcp")
        .arg("--data-dir")
        .arg("./docs/fixtures/minimal_static_data.db")
        .env("RUST_LOG", "info")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(workspace_root)
        .spawn()
}

fn send_request(child: &mut Child, request: Value) -> std::io::Result<Value> {
    // Write request and read a single JSON-RPC response (blocking read is fine in test).

    let stdin = child
        .stdin
        .as_mut()
        .expect("child process should have stdin piped");
    let mut stdout = BufReader::new(
        child
            .stdout
            .as_mut()
            .expect("child process should have stdout piped"),
    );

    writeln!(stdin, "{}", request)?;
    stdin.flush()?;

    // Read response synchronously (blocking is fine in test)
    let mut line = String::new();
    stdout.read_line(&mut line)?;

    serde_json::from_str(&line).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse JSON response: {}", e),
        )
    })
}

#[test]
fn test_stdio_isolation_initialize() {
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

    // Close stdin to signal EOF and allow server to exit gracefully
    drop(server.stdin.take());

    // Wait for process to exit (with timeout) using a portable loop.
    let start = Instant::now();
    let exit_status = loop {
        if server
            .try_wait()
            .expect("Failed to check server status")
            .is_some()
        {
            break server
                .try_wait()
                .expect("Failed to check server status")
                .unwrap();
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("Server did not exit within timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    assert!(
        exit_status.success(),
        "Server exited with error: {:?}",
        exit_status
    );

    // Read stderr output and ensure it contains logging
    let mut stderr = String::new();
    server
        .stderr
        .as_mut()
        .unwrap()
        .read_to_string(&mut stderr)
        .ok();
    assert!(stderr.contains("MCP server initialized"));

    // Ensure no log-level keywords leaked to stdout by checking response doesn't contain "INFO"/"ERROR"
    let stdout_str = serde_json::to_string(&response).unwrap();
    assert!(!stdout_str.contains("INFO") && !stdout_str.contains("ERROR"));
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

    let response = send_request(&mut server, request).expect("Failed to get tools/list response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].get("tools").is_some());

    // Close stdin and let process exit cleanly.
    drop(server.stdin.take());
    let start = Instant::now();
    loop {
        if server
            .try_wait()
            .expect("Failed to check server status")
            .is_some()
        {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("Server did not exit within timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
