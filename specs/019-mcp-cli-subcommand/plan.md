# Implementation Plan: MCP CLI Subcommand

**Branch**: `019-mcp-cli-subcommand` | **Date**: 2026-01-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/019-mcp-cli-subcommand/spec.md`

## Summary

Implement the `evefrontier-cli mcp` subcommand to launch the Model Context Protocol (MCP) server
using stdio transport. This enables AI assistants (Claude Desktop, VS Code Copilot, Cursor) to
interact with the EVE Frontier dataset.

**Primary Requirement**: Add CLI integration for the existing `evefrontier-mcp` server
implementation with proper stdio transport, logging isolation (stderr only), and dataset
initialization.

**Technical Approach**: Extend `evefrontier-cli` with a new Clap subcommand that instantiates the
MCP server from `evefrontier-mcp` crate, configures tracing to stderr, establishes stdio transport,
and manages server lifecycle including graceful shutdown.

## Technical Context

**Language/Version**: Rust 1.91.1 (per `.rust-toolchain`)  
**Primary Dependencies**:

- `clap` 4.x (CLI argument parsing)
- `evefrontier-mcp` (existing MCP server implementation)
- `evefrontier-lib` (dataset loading, starmap operations)
- `tokio` (async runtime for MCP server)
- `tracing` / `tracing-subscriber` (logging to stderr)
- `serde_json` (JSON-RPC protocol serialization)

**Storage**: SQLite database (e6c3 dataset) via `evefrontier-lib::ensure_e6c3_dataset()`  
**Testing**: `cargo test` (unit tests), integration tests with mock stdio streams  
**Target Platform**: Linux, macOS, Windows (cross-platform CLI)  
**Project Type**: CLI application with library integration  
**Performance Goals**:

- Cold start (dataset already cached): <5 seconds
- Warm start (dataset load): <2 seconds
- Request handling latency: <500ms p95

**Constraints**:

- stdio transport MUST be protocol-clean (no stderr leakage to stdout)
- All application logs MUST go to stderr
- Memory usage <512MB under normal operation
- Graceful shutdown on SIGTERM/SIGINT with proper cleanup

**Scale/Scope**:

- Single CLI subcommand (`mcp`)
- Integration with existing `evefrontier-mcp` crate (~500 LOC)
- Configuration: CLI flags + environment variables
- Documentation: Claude Desktop config snippet, VS Code config, troubleshooting guide

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

### ✅ Test-Driven Development (Principle I)

- **Status**: PASS
- **Plan**: Follow TDD for stdio transport isolation, logging configuration, and server lifecycle
- **Tests Required**:
  - Unit: stdio reader/writer behavior, config resolution logic
  - Integration: end-to-end MCP handshake with mock client
  - Smoke: `evefrontier-cli mcp` launches and accepts `initialize` request

### ✅ Library-First Architecture (Principle II)

- **Status**: PASS
- **Rationale**: MCP server logic already lives in `evefrontier-mcp` library crate
- **CLI Role**: Thin wrapper instantiating server with stdio transport and config
- **No Violations**: All business logic (tool handlers, resource providers) in library

### ✅ Architecture Decision Records (Principle III)

- **Status**: PASS (No ADR Required)
- **Rationale**: This is feature implementation following existing architecture
- **Reference ADRs**:
  - ADR 0003: Dataset downloader (already implemented)
  - ADR 0006: Software components and workspace structure
  - ADR 0007: DevSecOps practices (logging, security)
- **Decision**: CLI integration does not introduce new architectural patterns

### ✅ Clean Code & Cognitive Load (Principle IV)

- **Status**: PASS
- **Complexity Targets**:
  - McCabe complexity <15 per function
  - Max nesting depth: 3 levels
  - Descriptive names (e.g., `configure_tracing_stderr()`, `run_stdio_transport()`)
- **Enforcement**: clippy with `complexity` lint group enabled

### ✅ Security-First Development (Principle V)

- **Status**: PASS
- **Security Considerations**:
  - Input validation: None required (MCP protocol handles malformed JSON)
  - Secrets: None (local stdio process, no authentication)
  - File paths: Use `evefrontier-lib` dataset resolver (already validates paths)
  - Error messages: Structured via RFC 9457, no sensitive info leakage
- **Audit**: No new external dependencies beyond `clap` (already audited)

### ✅ Testing Tiers (Principle VI)

- **Status**: PASS
- **Test Plan**:
  - **Tier 1 (Unit)**: Config resolution, logging setup, shutdown handlers
  - **Tier 2 (Smoke)**: `make test-cli-mcp` - launch server, send initialize, verify response
  - **Tier 3 (CI)**: Full pipeline including stdio isolation validation

### ⚠️ Refactoring & Technical Debt (Principle VII)

- **Status**: MONITORING
- **Potential Debt**: If stdio transport implementation is complex, extract to separate module
- **Mitigation**: Keep transport logic <100 LOC; extract if exceeds threshold
- **Tracking**: Document in `docs/TODO.md` if refactoring deferred

## GATE DECISION: ✅ PROCEED TO PHASE 0

All principles satisfied. No ADR required. Security considerations addressed. Proceed with research
phase.

## Project Structure

### Documentation (this feature)

```text
specs/019-mcp-cli-subcommand/
├── plan.md              # This file
├── research.md          # Phase 0: stdio transport patterns, Clap integration, tracing config
├── data-model.md        # Phase 1: RuntimeConfig, McpSubcommand structs
├── quickstart.md        # Phase 1: User guide for Claude Desktop/VS Code integration
└── contracts/           # Phase 1: CLI interface spec, config schema
    ├── cli-interface.md
    └── config-schema.json
```

### Source Code

```text
crates/evefrontier-cli/
├── src/
│   ├── main.rs          # [MODIFY] Add Mcp subcommand to enum
│   ├── commands/
│   │   └── mcp.rs       # [NEW] MCP subcommand implementation
│   └── config.rs        # [MODIFY] Add MCP-specific config resolution
├── tests/
│   └── mcp_stdio.rs     # [NEW] Integration test for stdio transport
└── Cargo.toml           # [MODIFY] Add evefrontier-mcp dependency

crates/evefrontier-mcp/
├── src/
│   ├── lib.rs           # [EXISTING] Server implementation already present
│   ├── server.rs        # [EXISTING] JSON-RPC handler
│   ├── tools.rs         # [EXISTING] Tool implementations
│   └── resources.rs     # [EXISTING] Resource providers
└── [NO CHANGES NEEDED]  # Server logic complete

docs/
└── USAGE.md             # [MODIFY] Add "MCP Server" section with config examples
```

**Structure Decision**: Single project structure (Option 1). The CLI is a thin wrapper around the
existing `evefrontier-mcp` library crate. All MCP server logic lives in the library; CLI provides
entry point, stdio transport, and configuration.

## Complexity Tracking

**No violations to justify.** All Constitution principles satisfied.

---

## Phase 0: Research & Discovery

**Goal**: Resolve all technical unknowns and establish implementation patterns.

### Research Tasks

#### R1: Stdio Transport Patterns in Rust

**Question**: What are the best practices for implementing stdio-based JSON-RPC transport in Rust?

**Research Approach**:

- Survey existing Rust MCP implementations (if any)
- Review `tokio::io::stdin()` and `tokio::io::stdout()` for async I/O
- Investigate line-buffered vs. length-prefixed message framing
- Document error handling for broken pipe, EOF, client disconnection

**Deliverable**: Recommended transport architecture with code examples

#### R2: Tracing Configuration for Stderr-Only Logging

**Question**: How to configure `tracing-subscriber` to write exclusively to stderr while keeping
stdout clean?

**Research Approach**:

- Review `tracing-subscriber::fmt()` builder API
- Test `with_writer(std::io::stderr)` configuration
- Validate no output leaks to stdout under error conditions
- Document `RUST_LOG` environment variable behavior

**Deliverable**: Tracing initialization code snippet with validation test

#### R3: Clap Subcommand Integration Patterns

**Question**: What's the best way to add a long-running subcommand (server) to an existing Clap CLI?

**Research Approach**:

- Review existing CLI structure in `crates/evefrontier-cli/src/main.rs`
- Examine how other subcommands handle async runtimes
- Determine if `#[tokio::main]` needed in subcommand vs. main
- Document config resolution order (CLI flags → env vars → defaults)

**Deliverable**: Code pattern for async subcommand with examples

#### R4: Graceful Shutdown Implementation

**Question**: How to handle SIGTERM/SIGINT for graceful MCP server shutdown?

**Research Approach**:

- Review `tokio::signal::ctrl_c()` and `tokio::signal::unix::signal()`
- Investigate MCP protocol shutdown sequence (any finalization needed?)
- Test if JSON-RPC responses can be sent during shutdown
- Document timeout strategy for in-flight requests

**Deliverable**: Shutdown handler pattern with signal handling

#### R5: Dataset Initialization Timing

**Question**: Should dataset loading happen during server init or lazily on first request?

**Research Approach**:

- Analyze cold-start performance of `evefrontier-lib::ensure_e6c3_dataset()`
- Compare upfront vs. lazy loading for MCP protocol constraints
- Review MCP spec for initialization timeout recommendations
- Measure impact on `initialize` request response time

**Deliverable**: Decision matrix with performance benchmarks

### Research Output: `research.md`

Document findings for all 5 tasks in structured format:

```markdown
# Research: MCP CLI Subcommand

## R1: Stdio Transport Patterns

**Decision**: [Chosen approach] **Rationale**: [Why selected] **Alternatives Considered**: [What
else evaluated] **Code Example**: [Snippet]

## R2: Tracing Configuration

... [same structure]

## R3-R5: [same structure for each]
```

---

## Phase 1: Design & Contracts

**Prerequisites**: `research.md` complete with all decisions documented

### D1: Data Model Design (`data-model.md`)

**Entities to Define**:

1. **McpSubcommand** (Clap struct)
   - Fields: `data_dir: Option<PathBuf>`, `log_level: Option<String>`
   - Validation: Path must be readable/writable if specified
   - Relationships: Consumes `RuntimeConfig`

2. **RuntimeConfig** (Configuration resolver)
   - Fields: `dataset_path: PathBuf`, `log_level: String`
   - Resolution order: CLI flag → `EVEFRONTIER_DATA_DIR` → `EVEFRONTIER_LOG_LEVEL` → defaults
   - Validation: Dataset path exists or is creatable

3. **StdioTransport** (I/O wrapper)
   - Fields: `stdin: BufReader<Stdin>`, `stdout: BufWriter<Stdout>`
   - Operations: `read_message()`, `write_message()`, `flush()`
   - Error handling: Broken pipe, EOF, parse errors

4. **ServerState** (Runtime state)
   - Fields: `starmap: Starmap`, `spatial_index: Option<SpatialIndex>`, `config: RuntimeConfig`
   - Lifecycle: init → serving → shutdown
   - Thread safety: Arc<Mutex<>> or Arc<RwLock<>> for shared access

**Output**: `data-model.md` with entity definitions, relationships, state transitions

### D2: API Contracts (`contracts/`)

#### CLI Interface Contract (`cli-interface.md`)

**Command Signature**:

```bash
evefrontier-cli mcp [OPTIONS]

OPTIONS:
    --data-dir <PATH>       Custom dataset directory (overrides EVEFRONTIER_DATA_DIR)
    --log-level <LEVEL>     Log level: trace, debug, info, warn, error (default: info)
    -h, --help              Print help information
```

**Environment Variables**:

- `EVEFRONTIER_DATA_DIR`: Dataset directory
- `EVEFRONTIER_LOG_LEVEL`: Log level
- `RUST_LOG`: Fine-grained tracing control (standard Rust convention)

**Exit Codes**:

- `0`: Graceful shutdown (SIGTERM, Ctrl+C, client disconnect)
- `1`: Configuration error (invalid paths, missing dataset)
- `2`: Runtime error (protocol failure, internal panic)

#### Config Schema Contract (`config-schema.json`)

JSON schema defining:

- Claude Desktop `mcpServers` configuration
- Environment variable mappings
- Default values and examples

**Example**:

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

**Output**: `contracts/cli-interface.md` and `contracts/config-schema.json`

### D3: Quickstart Guide (`quickstart.md`)

**Target Audience**: Users integrating EVE Frontier with Claude Desktop or VS Code

**Sections**:

1. **Prerequisites**: evefrontier-cli installation, Claude Desktop/VS Code setup
2. **Configuration**: Step-by-step guide with screenshots
3. **Verification**: Test queries to confirm integration working
4. **Troubleshooting**: Common issues and solutions

**Example Test Query**:

> "What systems are within 50 light-years of Brana?"

**Expected Response**:

> Lists nearby systems with distances

**Output**: `quickstart.md` with complete user workflow

### D4: Agent Context Update

**Action**: Run `.specify/scripts/bash/update-agent-context.sh copilot`

**Purpose**: Update `.github/copilot-instructions.md` or `.github/agents/copilot.md` with:

- New MCP server capability
- CLI subcommand usage
- Configuration patterns
- Troubleshooting guide references

**Verification**: Context file updated with new MCP section between markers

---

## Phase 2: Implementation Planning (Deferred to `/speckit.tasks`)

**Note**: Phase 2 generates `tasks.md` with concrete implementation tasks, test cases, and
acceptance criteria. This is NOT created by `/speckit.plan` - it requires the `/speckit.tasks`
command after design phase completion.

**Planned Task Categories** (preview):

1. **T1-T5**: Core implementation (Clap integration, stdio transport, logging)
2. **T6-T10**: Testing (unit, integration, smoke tests)
3. **T11-T15**: Documentation (USAGE.md, README.md, quickstart)
4. **T16-T20**: CI/CD integration (test automation, release packaging)

**Blocking Dependencies**:

- Phase 0 research complete → Phase 1 design
- Phase 1 design complete → Phase 2 tasks generation

---

## Post-Design Constitution Re-Check

**Timing**: After Phase 1 design artifacts complete, before Phase 2 task generation

**Verification Checklist**:

- ✅ Library-first: MCP logic still in `evefrontier-mcp`, CLI is thin wrapper
- ✅ Clean code: Config resolution <100 LOC, transport wrapper <150 LOC
- ✅ Security: No new secrets, error messages sanitized
- ✅ Testing: Integration test plan covers stdio isolation
- ✅ ADR requirement: Still N/A (no new architectural patterns)

**Gate Decision**: If all checks pass → Proceed to `/speckit.tasks`

---

## Summary & Next Steps

**Current Status**: Planning complete, awaiting Phase 0 research execution

**Immediate Actions**:

1. ✅ Execute Phase 0 research (R1-R5)
2. ⏳ Document findings in `research.md`
3. ⏳ Execute Phase 1 design (D1-D4)
4. ⏳ Re-check Constitution compliance
5. ⏳ Run `/speckit.tasks` to generate implementation tasks

**Estimated Effort**:

- Phase 0 (Research): 4-6 hours
- Phase 1 (Design): 3-4 hours
- Phase 2 (Implementation): 8-12 hours (generated by `/speckit.tasks`)

**Dependencies**:

- External: None (all crates available)
- Internal: `evefrontier-mcp` crate must be functional (already implemented per TODO.md)

**Risk Assessment**:

- **Low Risk**: Stdio transport is standard pattern
- **Medium Risk**: Tracing configuration correctness (mitigated by integration test)
- **Low Risk**: Dataset initialization timing (fallback to sync loading if needed)
