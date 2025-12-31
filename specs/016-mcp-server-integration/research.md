# Research: MCP Server Integration

**Date**: 2025-12-31  
**Feature**: 016-mcp-server-integration  
**Purpose**: Resolve technical unknowns and validate design decisions before implementation

## Research Questions

### Q1: Which Rust MCP SDK to use?

**Decision**: Use `rmcp` (official Rust SDK from modelcontextprotocol org)

**Rationale**:
- Official SDK maintained by the Model Context Protocol team
- Active development (v0.12.0 as of Dec 2025, 2.8k stars)
- Supports both client and server modes
- Built on tokio async runtime (matches our existing async patterns)
- Includes procedural macros for ergonomic tool definition
- Well-documented with examples

**Alternatives Considered**:
- `mcp-server` crate: Lower-level, less ergonomic, fewer features
- Custom implementation: Too much effort, protocol is complex
- `mcp-rs`: Community fork, less active than official SDK

**Key Dependencies**:
```toml
rmcp = { version = "0.12", features = ["server"] }
schemars = "0.8"  # Required for JSON Schema generation
```

---

### Q2: How to implement stdio transport correctly?

**Decision**: Use `rmcp`'s built-in `StdioTransport` with tokio runtime

**Rationale**:
- stdio is the primary transport for local MCP servers (Claude Desktop, VS Code)
- `rmcp` provides `transport::stdio::StdioTransport` for this purpose
- Must redirect ALL logs to stderr to avoid corrupting JSON-RPC on stdout

**Implementation Pattern**:
```rust
use rmcp::transport::stdio::serve_stdio;
use tracing_subscriber::fmt;

fn main() -> Result<()> {
    // CRITICAL: Configure tracing to write to stderr only
    fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let server = MyMcpServer::new()?;
        serve_stdio(server).await
    })
}
```

**Key Insight**: The `println!` macro and any stdout writes will corrupt the JSON-RPC protocol.
All logging must use `tracing::*` macros configured for stderr.

---

### Q3: How to define tools with input validation?

**Decision**: Use `rmcp-macros` `#[tool]` attribute macro with `schemars` for JSON Schema

**Rationale**:
- Procedural macro reduces boilerplate
- `schemars` generates JSON Schema compliant with MCP specification (2020-12 draft)
- Input validation happens automatically via serde deserialization

**Implementation Pattern**:
```rust
use rmcp::tool;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RoutePlanInput {
    /// Starting system name
    pub origin: String,
    /// Goal system name
    pub destination: String,
    /// Routing algorithm: 'bfs', 'dijkstra', or 'a-star'
    #[serde(default)]
    pub algorithm: Option<String>,
    /// Maximum jump distance in light-years
    pub max_jump: Option<f64>,
}

#[tool(
    name = "route_plan",
    description = "Plan a route between two star systems in EVE Frontier"
)]
async fn route_plan(input: RoutePlanInput) -> Result<String, ToolError> {
    // Implementation calls evefrontier-lib::plan_route
}
```

---

### Q4: How to handle errors in MCP tools?

**Decision**: Return RFC 9457-style problem details wrapped in MCP error response

**Rationale**:
- Consistent with existing Lambda error handling pattern
- Provides structured errors that AI can interpret
- Includes helpful suggestions (e.g., fuzzy match results for unknown systems)

**Implementation Pattern**:
```rust
use rmcp::handler::server::ToolError;

fn handle_unknown_system(name: &str, suggestions: Vec<String>) -> ToolError {
    ToolError::ExecutionError {
        message: format!("Unknown system: '{}'. Did you mean: {}?", 
            name, 
            suggestions.join(", ")
        ),
    }
}
```

---

### Q5: How to load and share the starmap across tool invocations?

**Decision**: Load starmap and spatial index once at server initialization, share via `Arc`

**Rationale**:
- Avoids repeated disk I/O and parsing on each tool call
- Consistent with Lambda runtime pattern (`LambdaRuntime` singleton)
- Spatial index is ~50-100MB, should only be loaded once

**Implementation Pattern**:
```rust
use std::sync::Arc;

pub struct McpServerState {
    pub starmap: Arc<Starmap>,
    pub spatial_index: Option<Arc<SpatialIndex>>,
}

impl McpServerState {
    pub fn new(data_dir: Option<&Path>) -> Result<Self> {
        let paths = ensure_e6c3_dataset(data_dir)?;
        let starmap = Arc::new(load_starmap(&paths.database)?);
        let spatial_index = try_load_spatial_index(&paths.spatial_index)
            .ok()
            .map(Arc::new);
        
        Ok(Self { starmap, spatial_index })
    }
}
```

---

### Q6: What MCP resources should be exposed?

**Decision**: Expose three resources for dataset metadata and capability discovery

**Rationale**:
- Resources are read-only data that help AI understand the server's capabilities
- Dataset info helps AI calibrate expectations (system count, schema version)
- Algorithm list helps AI choose appropriate routing strategy

**Resources**:

| URI | Content |
|-----|---------|
| `evefrontier://dataset/info` | `{"system_count": 8000, "schema": "e6c3", "loaded_at": "..."}` |
| `evefrontier://algorithms` | `[{"name": "bfs", "description": "..."}, ...]` |
| `evefrontier://spatial-index/status` | `{"available": true, "version": 2, "build_time": "..."}` |

---

### Q7: How to integrate with CLI?

**Decision**: Add `evefrontier-cli mcp` subcommand that spawns the MCP server process

**Rationale**:
- Consistent with existing CLI structure (subcommands for each operation)
- Users already have CLI installed; no separate binary needed
- Can share `--data-dir` flag with other commands

**Implementation**:
```rust
// In evefrontier-cli/src/commands/mcp.rs
#[derive(Debug, Args)]
pub struct McpArgs {
    /// Override dataset directory
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

pub fn run_mcp_server(args: McpArgs) -> Result<()> {
    evefrontier_lib::mcp::run_stdio_server(args.data_dir.as_deref())
}
```

---

### Q8: How to test MCP server functionality?

**Decision**: Three testing tiers matching Constitution requirements

**Rationale**:
- Unit tests for individual tool handlers (fast, isolated)
- Integration tests using mock stdio for protocol compliance
- Smoke tests with real MCP client (Claude Desktop or mcp-inspector)

**Testing Patterns**:

1. **Unit Tests** (tool handlers):
```rust
#[tokio::test]
async fn test_route_plan_happy_path() {
    let state = McpServerState::from_fixture();
    let input = RoutePlanInput {
        origin: "Nod".into(),
        destination: "Brana".into(),
        algorithm: Some("a-star".into()),
        ..Default::default()
    };
    let result = route_plan_handler(&state, input).await;
    assert!(result.is_ok());
}
```

2. **Integration Tests** (protocol):
```rust
#[tokio::test]
async fn test_stdio_initialization() {
    let (tx, rx) = create_mock_stdio();
    let server = spawn_server(rx, tx);
    
    // Send initialize request
    send_json(&tx, json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { "protocolVersion": "2024-11-05" }
    }));
    
    let response = receive_json(&rx).await;
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
}
```

3. **Smoke Tests** (manual with mcp-inspector):
```bash
# Install MCP inspector
npx @modelcontextprotocol/inspector evefrontier-cli mcp

# Verify tools are listed and can be called
```

---

## Decisions Summary

| Question | Decision |
|----------|----------|
| SDK | `rmcp` v0.12 (official) |
| Transport | stdio via `rmcp::transport::stdio` |
| Tool definition | `#[tool]` macro + `schemars` |
| Error handling | RFC 9457-style errors |
| State management | `Arc<Starmap>` + `Arc<SpatialIndex>` |
| Resources | 3 metadata endpoints |
| CLI integration | `evefrontier-cli mcp` subcommand |
| Testing | Unit → Integration → Smoke (3-tier) |

## Open Questions (for implementation)

1. Should we support MCP prompts in v1? (Deferred to P3 per spec)
2. How to handle very large route responses? (Consider chunking or pagination)
3. Should spatial index be lazily loaded on first spatial query? (Yes, to reduce cold-start time)
