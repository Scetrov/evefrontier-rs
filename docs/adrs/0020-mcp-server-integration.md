# ADR 0020: MCP Server Integration for AI Assistant Access

**Date**: 2025-12-31  
**Status**: Accepted  
**Author**: Copilot (AI Agent)  
**Related**: [ADR 0006: Software Components](0006-software-components.md), [ADR 0007: DevSecOps Practices](0007-devsecops-practices.md)

## Context

The EVE Frontier project provides powerful tools for route planning, spatial queries, and dataset
exploration via a CLI and Lambda functions. However, this functionality is not directly accessible
to modern AI assistants (Claude Desktop, VS Code Copilot, Cursor, etc.) that users increasingly
rely on for exploration and planning assistance.

The Model Context Protocol (MCP) is a standardized protocol (maintained by Anthropic) that enables
AI assistants to interact with external tools and data sources via stdio transport. Implementing an
MCP server for EVE Frontier would:

- **Enable AI-Driven Exploration**: Users can ask "Find a route from Nod to Brana avoiding high-temp
  systems" directly in their AI assistant
- **Broaden Accessibility**: The tool becomes available to non-CLI users (exploratory analysis,
  strategic planning, integration with other AI workflows)
- **Enterprise Integration**: Self-hosted or containerized deployment enables usage in corporate
  environments
- **Foster Community**: Prompt templates and interactive discovery encourage broader EVE Frontier
  engagement

## Decision

### 1. Create a new `evefrontier-mcp` binary crate

The MCP server will be implemented as a dedicated Rust crate (`crates/evefrontier-mcp/`) that:

- Depends on `evefrontier-lib` for core routing, system queries, and dataset loading
- Implements stdio transport using the official `rmcp` (Rust MCP SDK) v0.12+
- Runs as a long-lived process spawned by AI clients via platform-specific spawning mechanisms
- Loads the EVE Frontier dataset once at startup and shares it across requests via `Arc<Mutex<_>>`

**Rationale**:
- Separating the MCP server into its own crate maintains clean architecture (server concerns do not
  leak into the library)
- Allows independent versioning and deployment of the MCP interface
- Follows the existing pattern in the workspace (CLI, Lambda, Service crates all depend on lib)

### 2. Stdio-based JSON-RPC 2.0 transport

The server will use stdio transport with JSON-RPC 2.0 message format per the MCP specification:

- Input: JSON-RPC 2.0 requests from stdin (from the AI client process)
- Output: JSON-RPC 2.0 responses written to stdout (to the AI client process)
- Logging: ALL application logging redirected to stderr (critical to prevent stdout corruption)

**Rationale**:
- stdio is the primary transport for local MCP servers (Claude Desktop, VS Code, Cursor all expect
  this)
- JSON-RPC 2.0 is a well-established standard ensuring interoperability
- Strict stdout/stderr separation is non-negotiable for protocol integrity

### 3. Four core tools: route_plan, system_info, systems_nearby, gates_from

The server exposes four MCP tools matching the primary use cases:

| Tool | Purpose | Input | Output |
|------|---------|-------|--------|
| `route_plan` | Plan a route between two systems with optional constraints (algorithm, max-distance, temperature, gate-only) | origin, destination, algorithm, constraints | Ordered list of system names with distances and metadata |
| `system_info` | Retrieve metadata about a single system (coordinates, temperature, planets, moons, connected gates) | system_name | System object with full metadata |
| `systems_nearby` | Find systems within a spatial radius (light-years) with optional temperature filter | center_system, radius_ly, max_temperature | List of nearby systems with distances |
| `gates_from` | Get gate-connected systems from a given system | system_name | List of directly connected systems |

**Rationale**:
- `route_plan` is the highest-value tool (primary use case: "Plan a route")
- System queries enable exploratory workflows ("What's nearby?", "Tell me about X system")
- Spatial queries leverage the KD-tree spatial index (ADR 0009) for efficient discovery
- Gate connectivity supports strategic planning and analysis

### 4. Three resources: dataset metadata, algorithms, spatial index status

The server exposes three MCP resources for introspection and metadata access:

| Resource | URI | Content | Purpose |
|----------|-----|---------|---------|
| Dataset Info | `evefrontier://dataset/info` | System count, jump count, schema version, build timestamp | Enables clients to understand data availability and freshness |
| Algorithms | `evefrontier://algorithms` | List of available routing algorithms and their constraints | Helps clients select appropriate algorithms for queries |
| Spatial Index Status | `evefrontier://spatial-index/status` | Index version, build timestamp, availability (loaded/auto-build) | Informs client about performance characteristics and freshness |

**Rationale**:
- Resources provide introspection without requiring tool execution
- Dataset metadata helps clients understand capabilities (system count, available algorithms)
- Spatial index status enables clients to understand performance trade-offs
- All information is static (computed once at startup), avoiding request overhead

### 5. MCP Prompts for enhanced AI assistance (Phase 2+, optional)

The server MAY expose MCP prompt templates for common EVE Frontier queries. Examples:

- **"Route Planning Advisor"**: Guides users through multi-leg route planning with constraints
- **"System Explorer"**: Helps analyze system characteristics and nearby alternatives
- **"Strategic Planner"**: Combines routing and spatial queries for operational planning

Prompts are optional and can be deferred to v1.1+ if higher-priority features need bandwidth.

**Rationale**:
- Prompts improve AI response quality by providing structured input templates
- They educate users on best practices and available capabilities
- They are not critical for MVP; the tools work without them

### 6. Error handling via RFC 9457 Problem Details

All tool failures return structured error responses using RFC 9457 Problem Details format:

```json
{
  "code": 404,
  "message": "System 'Unknown' not found",
  "type": "https://evefrontier.local/errors/system-not-found",
  "context": {
    "system_name": "Unknown",
    "suggestions": ["Nod", "Brana"],
    "message": "Did you mean one of these systems?"
  }
}
```

**Rationale**:
- RFC 9457 is a standardized format for structured error responses
- Including suggestions for fuzzy matching misses improves user experience
- Problem type URIs enable client-side error handling and user education

### 7. Fuzzy matching for system names

Tool handlers implement fuzzy matching (using `strsim` crate) to suggest corrections when a system
name is not found exactly:

- User input: "nodde"
- Response: Error with suggestions: ["Nod"] (sorted by similarity)

**Rationale**:
- EVE Frontier system names are not always intuitive (e.g., "P:STK3", "H:2L2S")
- Fuzzy matching significantly improves user experience by reducing failed queries
- Already used in the CLI; extending to MCP ensures consistency

### 8. Spatial index support with auto-build fallback

The server optionally loads a pre-built KD-tree spatial index at startup (built via
`evefrontier-cli index-build`):

- If index exists: Use it for fast spatial queries (negligible overhead)
- If index missing: Use library's fallback auto-build on first `systems_nearby` call (logs warning)
- If index stale: Log warning; client should rebuild via CLI

**Rationale**:
- Pre-built index provides near-instant spatial queries (< 50ms typical)
- Auto-build fallback ensures tool always works, even without index
- Clear logging helps users understand performance characteristics

### 9. Docker containerization with distroless base

The binary can be containerized using a multi-stage Dockerfile:

- Build stage: Use `rust:latest` with `cargo-zigbuild` for musl static linking
- Runtime stage: `gcr.io/distroless/cc-debian12:nonroot` (~20MB, secure by default)
- Security: No shell, non-root user, CAP_DROP=all, read-only root filesystem

**Rationale**:
- Distroless images minimize attack surface and container size
- Multi-stage build ensures only the binary is in the final image
- Static linking enables seamless deployment across container runtimes

### 10. Integration with existing Nx orchestration (ADR 0017)

The MCP crate follows Nx orchestration patterns established in ADR 0017:

- `project.json` defines targets: `build`, `test`, `clippy`, `fmt`
- Workspace `nx.json` provides target defaults and caching
- CI workflow uses Nx tasks: `nx run evefrontier-mcp:test`

**Rationale**:
- Consistency with existing workspace patterns reduces cognitive load
- Nx caching accelerates CI and local development
- Standardized task names enable script automation

## Consequences

### Benefits

1. **Broader User Base**: AI assistant integration makes EVE Frontier accessible to non-CLI users
2. **Improved UX**: Prompts and fuzzy matching enhance user experience
3. **Enterprise Readiness**: Docker containerization enables corporate deployments
4. **Extensibility**: MCP protocol is open; future tools can be added easily
5. **Community Engagement**: Interactive AI-driven exploration encourages adoption

### Costs & Risks

1. **Maintenance Burden**: New code surface (MCP crate + tests + documentation)
   - **Mitigation**: Keep implementation simple; defer optional features (prompts)
2. **Logging Complexity**: stderr-only logging requires careful tracing configuration
   - **Mitigation**: Use `tracing-subscriber` with explicit stderr writer; test extensively
3. **Concurrent Request Handling**: Multiple AI clients may connect to the same server instance
   - **Mitigation**: Use Arc<RwLock> for dataset and spatial index; stress-test in CI
4. **Cold Start Latency**: Dataset + index loading on startup
   - **Mitigation**: Lazy-load spatial index; measure and document cold-start times
5. **Protocol Version Pinning**: MCP protocol may evolve; rmcp SDK updates required
   - **Mitigation**: Pin `rmcp` version; document minimum supported protocol version

## Alternatives Considered

### 1. Embed MCP server in CLI as `mcp` subcommand

**Pros**: Reuse CLI infra, single binary  
**Cons**: Couples CLI and MCP lifetimes; harder to containerize standalone  
**Decision**: Create separate crate; CLI can spawn it if desired

### 2. Use async HTTP server instead of stdio

**Pros**: Easier tooling integration (curl, HTTP clients)  
**Cons**: Doesn't match MCP protocol; Claude Desktop expects stdio  
**Decision**: Stick with stdio per MCP spec

### 3. Implement custom MCP server from scratch

**Pros**: Full control, lightweight  
**Cons**: Protocol complexity, security risks, maintenance burden  
**Decision**: Use official `rmcp` SDK

### 4. Deploy MCP server as Lambda function

**Pros**: Serverless scaling, AWS integration  
**Cons**: No streaming stdin/stdout support; incompatible with MCP transport  
**Decision**: Stdio-based process for local AI clients; Lambdas remain for HTTP API

## Validation & Metrics

### Success Criteria (from feature spec)

- ✅ SC-001: MCP server successfully initializes and completes handshake with Claude Desktop
- ✅ SC-002: All 4 core tools pass integration tests
- ✅ SC-003: Docker container passes security scan (no HIGH/CRITICAL vulnerabilities)
- ✅ SC-004: p95 response latency < 500ms for typical queries
- ✅ SC-005: Configuration examples for Claude Desktop, VS Code, Cursor

### Testing Strategy

1. **Unit Tests**: Error types, fuzzy matching, constraint validation (in `evefrontier-mcp/tests/`)
2. **Integration Tests**: Tool handlers with fixture database (via `docs/fixtures/minimal_static_data.db`)
3. **Protocol Tests**: JSON-RPC 2.0 message exchange, MCP handshake (via test client mock)
4. **Container Tests**: Dockerfile builds, image runs, security scan passes
5. **E2E Tests**: Real AI client (Claude Desktop) connects and executes tools

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Cold Start | < 5s | Including dataset load |
| Tool Latency (p95) | < 500ms | Typical route or query |
| Memory Usage | < 512MB | Running with fixture DB + spatial index |
| Docker Image Size | < 100MB | Multi-stage build + distroless base |

## Related ADRs

- [ADR 0006: Software Components](0006-software-components.md) — Component architecture
- [ADR 0007: DevSecOps Practices](0007-devsecops-practices.md) — Testing and security
- [ADR 0009: Spatial Index (KD-tree)](0009-spatial-index.md) — KD-tree implementation
- [ADR 0017: NX Repository Orchestration](0017-nx-orchestration-strategy.md) — Nx task patterns

## References

- [Model Context Protocol Specification](https://spec.modelcontextprotocol.io/)
- [rmcp GitHub Repository](https://github.com/modelcontextprotocol/rust-sdk)
- [RFC 9457: Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc9457)
- [Claude Desktop Configuration](https://claude.ai/docs/claude-desktop)
- [VS Code MCP Integration](https://code.visualstudio.com/docs/copilot)
