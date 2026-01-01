# Feature Specification: MCP CLI Subcommand

**Feature Branch**: `019-mcp-cli-subcommand`  
**Created**: 2026-01-01  
**Status**: Planning  
**Parent Spec**: [016-mcp-server-integration](../016-mcp-server-integration/spec.md)

## Overview

Implement the `evefrontier-cli mcp` subcommand that launches the Model Context Protocol (MCP) server
with stdio transport. This enables AI assistants (Claude Desktop, VS Code Copilot, Cursor) to
interact with the EVE Frontier static dataset through the MCP protocol.

The MCP server implementation already exists in `crates/evefrontier-mcp`. This feature focuses on:

1. CLI integration: adding the `mcp` subcommand to `evefrontier-cli`
2. Proper stdio transport setup (input/output isolation)
3. Logging configuration (stderr only, preventing stdout corruption)
4. Dataset initialization and runtime lifecycle management
5. Configuration and documentation for MCP client integration

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Launch MCP Server from CLI (Priority: P1)

A developer wants to configure Claude Desktop to use the EVE Frontier MCP server by running
`evefrontier-cli mcp` as a stdio process.

**Why this priority**: This is the primary integration point for all MCP functionality; without the
CLI subcommand, the MCP server cannot be used.

**Independent Test**: Run `evefrontier-cli mcp` and verify the server initializes, performs MCP
handshake, and responds to `initialize` request.

**Acceptance Scenarios**:

1. **Given** evefrontier-cli is installed, **When** user runs `evefrontier-cli mcp`, **Then** the
   MCP server starts and writes only JSON-RPC messages to stdout.
2. **Given** the server is running, **When** an MCP client sends an `initialize` request, **Then**
   the server responds with capabilities (tools, resources, protocol version).
3. **Given** the dataset is not found, **When** server starts, **Then** it downloads the latest
   dataset and logs progress to stderr (not stdout).

---

### User Story 2 - Configure Claude Desktop Integration (Priority: P1)

A user wants to add EVE Frontier to their Claude Desktop configuration and use it in conversations.

**Why this priority**: Claude Desktop is the primary target for MCP integration; providing a working
configuration is essential for adoption.

**Independent Test**: Add the provided configuration to `claude_desktop_config.json`, restart
Claude, and verify "evefrontier" server appears in available tools.

**Acceptance Scenarios**:

1. **Given** a configuration snippet is provided in documentation, **When** user adds it to Claude
   Desktop config, **Then** the server starts successfully and tools are available.
2. **Given** the server is configured with a custom data directory via environment variable,
   **When** Claude Desktop launches the server, **Then** the correct dataset path is used.
3. **Given** an error occurs during initialization, **When** Claude Desktop connects, **Then** error
   details are logged to stderr and a friendly error is returned via the protocol.

---

### User Story 3 - Logging Isolation and Debugging (Priority: P2)

A developer troubleshooting MCP integration wants to see detailed logs without corrupting the stdio
protocol.

**Why this priority**: Debugging is critical for early adopters and developers; clean logging
separation prevents hard-to-diagnose protocol failures.

**Independent Test**: Run the server with `RUST_LOG=debug` and verify all logs go to stderr while
stdout contains only JSON-RPC messages.

**Acceptance Scenarios**:

1. **Given** `RUST_LOG=trace` is set, **When** the server runs, **Then** detailed tracing logs
   appear in stderr but stdout remains clean JSON-RPC.
2. **Given** a tool execution fails, **When** the error is logged, **Then** stderr contains the full
   error context and stdout returns a proper JSON-RPC error response.
3. **Given** the server is running, **When** dataset loading completes, **Then** initialization time
   metrics are logged to stderr.

---

### User Story 4 - Custom Data Directory Configuration (Priority: P2)

A user wants to specify a custom location for the EVE Frontier dataset (e.g., shared network drive,
custom cache).

**Why this priority**: Flexibility in dataset location is important for enterprise deployments and
users with specific storage requirements.

**Independent Test**: Set `EVEFRONTIER_DATA_DIR=/custom/path` and verify the server uses that path
for dataset storage.

**Acceptance Scenarios**:

1. **Given** `EVEFRONTIER_DATA_DIR` is set, **When** server starts, **Then** dataset is loaded from
   the specified directory.
2. **Given** the custom directory does not exist, **When** server starts, **Then** the directory is
   created and dataset is downloaded.
3. **Given** `--data-dir` CLI flag is used, **When** server starts, **Then** the flag value takes
   precedence over environment variable.

---

### Edge Cases

- What happens when stdout is closed by the client unexpectedly?
- How does the system handle SIGTERM/SIGINT for graceful shutdown?
- What happens when concurrent requests exceed available memory?
- How are protocol version mismatches detected and reported?
- What happens when the dataset file is corrupted or incomplete?

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST implement `evefrontier-cli mcp` subcommand that launches the MCP server
- **FR-002**: System MUST use stdio transport exclusively (read from stdin, write to stdout)
- **FR-003**: System MUST redirect all application logs to stderr (tracing, errors, initialization)
- **FR-004**: System MUST initialize dataset before accepting requests (download if missing)
- **FR-005**: System MUST implement graceful shutdown on SIGTERM/SIGINT
- **FR-006**: System MUST support custom data directory via `--data-dir` flag and
  `EVEFRONTIER_DATA_DIR` env var
- **FR-007**: System MUST return initialization errors via MCP protocol (not process exit codes
  during handshake)
- **FR-008**: System MUST log cold-start metrics (dataset load time, initialization time) to stderr

### Non-Functional Requirements

- **NFR-001**: Server MUST initialize within 5 seconds on warm cache (dataset already downloaded)
- **NFR-002**: Server MUST handle at least 10 concurrent requests without blocking
- **NFR-003**: Memory usage MUST not exceed 512MB under normal operation
- **NFR-004**: Logging MUST support standard `RUST_LOG` environment variable for level control

### Key Entities _(include if feature involves data)_

- **McpSubcommand**: Clap subcommand struct for `mcp` command with configuration options
- **StdioTransport**: Wrapper managing stdin/stdout as MCP transport layer
- **RuntimeConfig**: Configuration struct combining CLI args, env vars, and defaults
- **ServerLifecycle**: State machine managing initialization, request processing, and shutdown

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: `evefrontier-cli mcp --help` displays usage information
- **SC-002**: Running `evefrontier-cli mcp` completes MCP handshake with Claude Desktop successfully
- **SC-003**: All 4 core MCP tools are callable from Claude Desktop after configuration
- **SC-004**: Documentation includes working `claude_desktop_config.json` snippet
- **SC-005**: Integration test validates stdio transport isolation (no stderr leakage to stdout)
- **SC-006**: Graceful shutdown on Ctrl+C returns proper JSON-RPC response and exits cleanly

## Out of Scope

- GUI or web-based MCP client (stdio only)
- HTTP transport for MCP (stdio only in this phase)
- Authentication/authorization (local process trust model)
- Multi-tenancy or user isolation (single-user local process)
- Persistent connection state across restarts

## Technical Notes

### Stdio Transport Requirements

The MCP protocol over stdio requires strict separation:

- **stdin**: Read JSON-RPC 2.0 messages (one per line or length-prefixed)
- **stdout**: Write JSON-RPC 2.0 responses only (no debug output, no logs)
- **stderr**: All application logs, errors, and diagnostic information

Any non-JSON data written to stdout will corrupt the protocol and cause client failures.

### Integration with Existing MCP Server

The `evefrontier-mcp` crate already implements:

- JSON-RPC 2.0 message handling
- Tool registration (`route_plan`, `system_info`, `systems_nearby`, `gates_from`)
- Resource handlers (`evefrontier://dataset/info`, etc.)
- Error formatting (RFC 9457 Problem Details)

This feature adds:

- CLI entry point
- Stdio transport wiring
- Logging configuration
- Dataset initialization orchestration

### Reference Configuration

Example Claude Desktop configuration:

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "evefrontier-cli",
      "args": ["mcp"],
      "env": {
        "EVEFRONTIER_DATA_DIR": "${HOME}/.cache/evefrontier",
        "RUST_LOG": "info"
      }
    }
  }
}
```

## References

- [MCP Specification](https://modelcontextprotocol.io/docs/specification)
- [Parent Spec: MCP Server Integration](../016-mcp-server-integration/spec.md)
- [ADR 0003: Downloader and Caching](../../docs/adrs/0003-downloader-caching.md)
- [evefrontier-mcp crate](../../crates/evefrontier-mcp/)
