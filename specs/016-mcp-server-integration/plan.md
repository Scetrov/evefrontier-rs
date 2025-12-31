# Implementation Plan: MCP Server Integration

**Branch**: `016-mcp-server-integration` | **Date**: 2025-12-31 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/016-mcp-server-integration/spec.md`

## Summary

Implement a Model Context Protocol (MCP) server for EVE Frontier that enables AI assistants to
interact with the starmap dataset via stdio transport. The server will expose route planning, system
queries, and spatial search as MCP tools, with dataset metadata as resources. The implementation
uses the official Rust MCP SDK (`rmcp`) and follows the existing library-first architecture pattern.

## Technical Context

**Language/Version**: Rust 1.91.1 (per `.rust-toolchain`)  
**Primary Dependencies**: `rmcp` (official MCP SDK), `tokio`, `serde`, `schemars` (JSON Schema)  
**Storage**: SQLite database (bundled EVE Frontier dataset from `evefrontier_datasets` repo)  
**Testing**: `cargo test` with integration tests using fixture dataset  
**Target Platform**: Linux, macOS, Windows (stdio transport for local MCP clients)  
**Project Type**: Library extension + new CLI subcommand (follows existing workspace pattern)  
**Performance Goals**: <5s cold start, <500ms p95 tool latency, handle 10+ concurrent queries  
**Constraints**: <512MB memory (Lambda-compatible), all logs to stderr, JSON-RPC 2.0 compliant  
**Scale/Scope**: Single-user local server (one MCP client connection at a time via stdio)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence/Notes |
|-----------|--------|----------------|
| I. Test-Driven Development | ✅ PASS | Will write tool handler tests first, then implement |
| II. Library-First Architecture | ✅ PASS | MCP handlers call `evefrontier-lib` APIs; no business logic in handlers |
| III. Architecture Decision Records | ⚠️ PENDING | ADR required for MCP integration decisions |
| IV. Clean Code & Cognitive Load | ✅ PASS | Tool handlers follow single-responsibility (one tool = one handler) |
| V. Security-First Development | ✅ PASS | Input validation on all tool parameters; no external URL fetching |
| VI. Testing Tiers | ✅ PASS | Unit tests for handlers, integration tests for stdio transport |
| VII. Refactoring & Technical Debt | ✅ PASS | Building on existing library; no legacy code modification |

**Pre-Phase 0 Gate**: ✅ PASS - All principles satisfied or have clear path to compliance.

### Post-Design Constitution Re-Check (Phase 1 Complete)

| Principle | Status | Evidence/Notes |
|-----------|--------|----------------|
| I. Test-Driven Development | ✅ PASS | Tool schemas defined → tests can be written first → implementation follows |
| II. Library-First Architecture | ✅ PASS | MCP-specific glue in `evefrontier-mcp/src/`; business logic delegates to existing `evefrontier-lib` APIs (routing, dataset loading, spatial queries) |
| III. Architecture Decision Records | ✅ PASS | ADR 0020 to be created during implementation (tracked in tasks.md) |
| IV. Clean Code & Cognitive Load | ✅ PASS | Each tool has single responsibility; clear input/output contracts |
| V. Security-First Development | ✅ PASS | JSON Schema validation; no external network calls; input bounds checked |
| VI. Testing Tiers | ✅ PASS | Unit (tool handlers) → Integration (protocol) → Smoke (mcp-inspector) |
| VII. Refactoring & Technical Debt | ✅ PASS | Extends existing APIs; no breaking changes to lib public API |

**Post-Phase 1 Gate**: ✅ PASS - Design aligns with all Constitution principles.

## Project Structure

### Documentation (this feature)

```text
specs/016-mcp-server-integration/
├── plan.md              # This file
├── research.md          # Phase 0: MCP SDK evaluation, transport patterns
├── data-model.md        # Phase 1: Tool schemas, resource URIs
├── quickstart.md        # Phase 1: Configuration examples for AI clients
├── contracts/           # Phase 1: JSON Schema definitions
│   ├── tools.json       # Tool input/output schemas
│   └── resources.json   # Resource URI schemas
└── tasks.md             # Phase 2: Implementation tasks (TDD)
```

### Source Code (repository root)

```text
crates/
├── evefrontier-lib/
│   └── src/
│       └── mcp/                    # NEW: MCP-specific types and helpers
│           ├── mod.rs              # Module exports
│           ├── tools.rs            # Tool handler implementations
│           └── resources.rs        # Resource handler implementations
│
├── evefrontier-mcp/                # NEW: MCP server binary crate
│   ├── Cargo.toml
│   ├── project.json                # Nx configuration
│   ├── src/
│   │   ├── main.rs                 # Entry point, stdio transport setup
│   │   └── server.rs               # MCP server configuration and lifecycle
│   └── tests/
│       ├── integration_test.rs     # End-to-end MCP protocol tests
│       └── tool_tests.rs           # Individual tool handler tests
│
├── evefrontier-cli/
│   └── src/
│       └── commands/
│           └── mcp.rs              # NEW: `evefrontier-cli mcp` subcommand
│
└── evefrontier-service-mcp/        # FUTURE: HTTP-based MCP transport (Phase 2+)
    └── ...

docs/
├── MCP_SERVER.md                   # NEW: MCP server documentation
└── adrs/
    └── 0020-mcp-server-integration.md  # NEW: ADR for MCP decisions
```

**Structure Decision**: Following the existing workspace pattern, create a new `evefrontier-mcp`
crate for the MCP server binary. MCP-specific logic lives in `evefrontier-lib/src/mcp/` module to
maintain library-first architecture. CLI gets a thin `mcp` subcommand that delegates to the library.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| New crate (`evefrontier-mcp`) | Standalone binary for MCP server | Embedding in CLI would bloat CLI binary size; MCP users don't need routing CLI |
| MCP module in lib | Centralize tool schemas/handlers | Duplicating across CLI and standalone would violate DRY |

## Dependencies Analysis

### Required Crates (to add)

```toml
# evefrontier-lib/Cargo.toml additions
schemars = "0.8"  # JSON Schema generation for MCP tool inputs

# evefrontier-mcp/Cargo.toml (new crate)
[dependencies]
evefrontier-lib = { path = "../evefrontier-lib" }
rmcp = { version = "0.12", features = ["server"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Dependency Risk Assessment

| Dependency | Maturity | Risk | Mitigation |
|------------|----------|------|------------|
| `rmcp` | Official SDK, active | Low | Official support from Anthropic/MCP team |
| `schemars` | Stable, widely used | Low | Already used in ecosystem |
| `tokio` | Mature | Low | Already a dependency |

## MCP Tools Design

### Tool: `route_plan`

**Purpose**: Plan a route between two star systems.

**Input Schema**:
```json
{
  "origin": "string (required) - Starting system name",
  "destination": "string (required) - Goal system name",
  "algorithm": "string (optional) - 'bfs' | 'dijkstra' | 'a-star' (default: 'a-star')",
  "max_jump": "number (optional) - Maximum jump distance in light-years",
  "max_temperature": "number (optional) - Maximum system temperature in Kelvin",
  "avoid_systems": "array<string> (optional) - System names to exclude",
  "avoid_gates": "boolean (optional) - Use spatial-only routing"
}
```

**Output**: Route with system names, hop count, total distance, edge types.

### Tool: `system_info`

**Purpose**: Get detailed information about a star system.

**Input Schema**:
```json
{
  "system_name": "string (required) - System name (supports fuzzy matching)"
}
```

**Output**: System coordinates, temperature, planets, moons, connected gates.

### Tool: `systems_nearby`

**Purpose**: Find systems within a radius of a given system.

**Input Schema**:
```json
{
  "system_name": "string (required) - Center system name",
  "radius_ly": "number (required) - Search radius in light-years",
  "max_temperature": "number (optional) - Temperature filter",
  "limit": "number (optional) - Maximum results (default: 20)"
}
```

**Output**: List of nearby systems with distances.

### Tool: `gates_from`

**Purpose**: List jump gate connections from a system.

**Input Schema**:
```json
{
  "system_name": "string (required) - System name"
}
```

**Output**: List of gate-connected systems.

## MCP Resources Design

| URI | Description | Content Type |
|-----|-------------|--------------|
| `evefrontier://dataset/info` | Dataset metadata (system count, schema version) | `application/json` |
| `evefrontier://algorithms` | Available routing algorithms and descriptions | `application/json` |
| `evefrontier://spatial-index/status` | Spatial index availability and version | `application/json` |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `rmcp` API changes | Medium | Medium | Pin version, add integration tests |
| Stdio buffer issues | Low | High | Test with large routes; add streaming for long responses |
| Memory pressure from spatial index | Medium | Medium | Lazy-load index; document memory requirements |
| Protocol version mismatch | Low | Medium | Implement version negotiation per MCP spec |
