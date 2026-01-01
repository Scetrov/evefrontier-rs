# Tasks: MCP CLI Subcommand

**Input**: Design documents from `/specs/019-mcp-cli-subcommand/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Tests**: Not explicitly requested in spec; focusing on implementation and smoke testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and documentation structure

- [ ] T001 Create `specs/019-mcp-cli-subcommand/data-model.md` documenting McpSubcommand, RuntimeConfig, StdioTransport, ServerState entities
- [ ] T002 Create `specs/019-mcp-cli-subcommand/contracts/` directory
- [ ] T003 [P] Create `specs/019-mcp-cli-subcommand/contracts/cli-interface.md` documenting command signature, env vars, exit codes
- [ ] T004 [P] Create `specs/019-mcp-cli-subcommand/contracts/config-schema.json` with Claude Desktop configuration example
- [ ] T005 Create `specs/019-mcp-cli-subcommand/quickstart.md` with user integration guide for Claude Desktop and VS Code

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Add `evefrontier-mcp` dependency to `crates/evefrontier-cli/Cargo.toml`
- [ ] T007 Add `tokio` with `io-util`, `macros`, `signal` features to `crates/evefrontier-cli/Cargo.toml`
- [ ] T008 Create `crates/evefrontier-cli/src/commands/` directory
- [ ] T009 Create `crates/evefrontier-cli/src/commands/mod.rs` exporting `mcp` module
- [ ] T010 Add `mod commands;` to `crates/evefrontier-cli/src/main.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Launch MCP Server from CLI (Priority: P1) 🎯 MVP

**Goal**: Implement `evefrontier-cli mcp` command that launches MCP server with stdio transport, proper logging isolation, and dataset initialization.

**Independent Test**: Run `evefrontier-cli mcp` manually, send JSON-RPC `initialize` request via stdin, verify response on stdout and logs on stderr.

### Implementation for User Story 1

- [ ] T011 [P] [US1] Add `Mcp` variant to `Command` enum in `crates/evefrontier-cli/src/main.rs`
- [ ] T012 [P] [US1] Create `McpCommandArgs` struct with `log_level: Option<String>` in `crates/evefrontier-cli/src/main.rs`
- [ ] T013 [US1] Create `crates/evefrontier-cli/src/commands/mcp.rs` with module skeleton
- [ ] T014 [US1] Implement `configure_tracing(log_level: Option<&str>)` function in `crates/evefrontier-cli/src/commands/mcp.rs` using `tracing_subscriber::fmt().with_writer(std::io::stderr)`
- [ ] T015 [US1] Implement `resolve_dataset_path(global: &GlobalOptions)` function in `crates/evefrontier-cli/src/commands/mcp.rs` following CLI flag → env var → XDG → fallback order
- [ ] T016 [US1] Define `StdioTransport` struct with `reader: BufReader<Stdin>` and `writer: Stdout` in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T017 [US1] Implement `StdioTransport::new()` constructor in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T018 [US1] Implement `async fn StdioTransport::read_message(&mut self) -> Result<Value>` with line-delimited JSON parsing in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T019 [US1] Implement `async fn StdioTransport::write_message(&mut self, msg: &Value) -> Result<()>` with newline appending in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T020 [US1] Implement `async fn run_server_loop(server: McpServerState, transport: StdioTransport) -> Result<()>` with `tokio::select!` on `ctrl_c()` and `read_message()` in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T021 [US1] Implement `pub async fn run_mcp_server(global: &GlobalOptions, args: &McpCommandArgs) -> Result<()>` orchestrating tracing config, dataset init, server creation, transport setup, and server loop in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T022 [US1] Wire `Command::Mcp(args)` match arm in `main()` to call `commands::mcp::run_mcp_server(&cli.global, args).await` in `crates/evefrontier-cli/src/main.rs`
- [ ] T023 [US1] Add `#[tokio::main]` attribute to `main()` function in `crates/evefrontier-cli/src/main.rs` (if not already present)
- [ ] T024 [US1] Add `use evefrontier_mcp::McpServerState;` and other necessary imports to `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T025 [US1] Test: Build CLI with `cargo build -p evefrontier-cli` and verify compilation succeeds
- [ ] T026 [US1] Test: Run `evefrontier-cli mcp --help` and verify help text displays
- [ ] T027 [US1] Test: Launch `evefrontier-cli mcp` with `RUST_LOG=debug`, send `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}` via stdin, verify JSON response on stdout and logs on stderr only

**Checkpoint**: User Story 1 complete - MCP server launches via CLI, handles stdio transport, and logs to stderr

---

## Phase 4: User Story 2 - Configure Claude Desktop Integration (Priority: P1)

**Goal**: Provide working Claude Desktop configuration and documentation for users to integrate EVE Frontier MCP server.

**Independent Test**: Follow quickstart.md instructions to add EVE Frontier to Claude Desktop config, restart Claude, ask "What systems are within 50 ly of Brana?" and verify response.

### Implementation for User Story 2

- [ ] T028 [P] [US2] Add "MCP Server" section to `docs/USAGE.md` with subsections: Overview, Configuration, Usage Examples, Troubleshooting
- [ ] T029 [P] [US2] Document Claude Desktop configuration in `docs/USAGE.md` with JSON snippet for `claude_desktop_config.json` including `command`, `args`, and `env` fields
- [ ] T030 [P] [US2] Document VS Code configuration in `docs/USAGE.md` with JSON snippet for `.vscode/mcp.json`
- [ ] T031 [P] [US2] Document Cursor configuration in `docs/USAGE.md` with appropriate config file path and format
- [ ] T032 [US2] Add example queries to `docs/USAGE.md` MCP Server section: route planning, system info, nearby systems, gate connections
- [ ] T033 [US2] Add troubleshooting subsection to `docs/USAGE.md` covering: dataset not found, stdout corruption, initialization timeout, protocol version mismatch
- [ ] T034 [US2] Update `README.md` "Features" section to mention MCP server integration
- [ ] T035 [US2] Add "AI Assistant Integration" subsection to `README.md` with link to `docs/USAGE.md#mcp-server`
- [ ] T036 [US2] Test: Validate all JSON configuration snippets are syntactically correct
- [ ] T037 [US2] Test: Follow quickstart.md instructions manually and verify Claude Desktop integration works

**Checkpoint**: User Story 2 complete - Documentation enables users to integrate with Claude Desktop, VS Code, and Cursor

---

## Phase 5: User Story 3 - Logging Isolation and Debugging (Priority: P2)

**Goal**: Enable developers to debug MCP integration with detailed logs without corrupting the stdio protocol.

**Independent Test**: Run `RUST_LOG=trace evefrontier-cli mcp` and verify stderr contains detailed traces while stdout remains clean JSON-RPC.

### Implementation for User Story 3

- [ ] T038 [US3] Add validation in `configure_tracing()` to ensure `with_writer(std::io::stderr)` is set in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T039 [US3] Add `tracing::info!("MCP server initialized, waiting for requests...")` at start of `run_server_loop()` in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T040 [US3] Add `tracing::info!("Received shutdown signal, exiting gracefully")` in ctrl_c branch in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T041 [US3] Add `tracing::info!("Client disconnected (EOF)")` when EOF detected in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T042 [US3] Add `tracing::error!("Transport error: {}", e)` for transport errors in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T043 [US3] Add dataset loading timing metrics: `tracing::info!("Dataset loaded in {:?}", elapsed)` after `McpServerState::with_path()` in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T044 [US3] Document `RUST_LOG` environment variable usage in `docs/USAGE.md` MCP Server section with level descriptions (trace, debug, info, warn, error)
- [ ] T045 [US3] Add debugging tips section to `docs/USAGE.md` with examples: capturing stderr to file, common error patterns, protocol corruption diagnosis
- [ ] T046 [US3] Create integration test `crates/evefrontier-cli/tests/mcp_stdio.rs` that verifies no stderr content leaks to stdout
- [ ] T047 [US3] Test: Run `RUST_LOG=trace evefrontier-cli mcp` with mock stdin and verify stdout contains only valid JSON-RPC messages

**Checkpoint**: User Story 3 complete - Logging infrastructure supports debugging without protocol corruption

---

## Phase 6: User Story 4 - Custom Data Directory Configuration (Priority: P2)

**Goal**: Allow users to specify custom dataset locations via CLI flag or environment variable.

**Independent Test**: Set `EVEFRONTIER_DATA_DIR=/custom/path` and verify server uses that path; then test `--data-dir /other/path` takes precedence.

### Implementation for User Story 4

- [ ] T048 [US4] Verify `GlobalOptions::data_dir` field exists in `crates/evefrontier-cli/src/main.rs` (should already be present)
- [ ] T049 [US4] Update `resolve_dataset_path()` in `crates/evefrontier-cli/src/commands/mcp.rs` to prioritize: `global.data_dir` → `EVEFRONTIER_DATA_DIR` → XDG → fallback
- [ ] T050 [US4] Add directory creation logic in `resolve_dataset_path()` if custom path doesn't exist in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T051 [US4] Add error handling for unwritable/invalid custom paths in `crates/evefrontier-cli/src/commands/mcp.rs`
- [ ] T052 [US4] Document `--data-dir` flag usage in `docs/USAGE.md` MCP Server section
- [ ] T053 [US4] Document `EVEFRONTIER_DATA_DIR` environment variable in `docs/USAGE.md` MCP Server section
- [ ] T054 [US4] Update Claude Desktop config example in `docs/USAGE.md` to show custom data directory via `env`
- [ ] T055 [US4] Test: Launch with `--data-dir /tmp/evefrontier-test` and verify dataset is downloaded there
- [ ] T056 [US4] Test: Launch with `EVEFRONTIER_DATA_DIR=/tmp/evefrontier-env` and verify it's used
- [ ] T057 [US4] Test: Launch with both flag and env var set, verify flag takes precedence

**Checkpoint**: User Story 4 complete - Users can customize dataset location flexibly

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T058 [P] Add "MCP Server" entry to `CHANGELOG.md` under Unreleased section
- [ ] T059 [P] Update `.github/copilot-instructions.md` or `.github/agents/copilot.md` with MCP server usage patterns
- [ ] T060 [P] Run `cargo fmt --all` to format all modified code
- [ ] T061 [P] Run `cargo clippy -p evefrontier-cli -- -D warnings` and fix any lints
- [ ] T062 [P] Run `cargo test -p evefrontier-cli` and ensure all tests pass
- [ ] T063 Add smoke test to `Makefile` or CI: `test-mcp-smoke` target that launches server, sends initialize, verifies response
- [ ] T064 Update `crates/evefrontier-cli/README.md` (if exists) or project README with MCP subcommand
- [ ] T065 Verify all file paths in tasks match actual implementation
- [ ] T066 Run manual end-to-end test: Configure Claude Desktop, ask routing question, verify correct answer
- [ ] T067 Update `specs/019-mcp-cli-subcommand/plan.md` status to "Implementation Complete"

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately (documentation artifacts)
- **Foundational (Phase 2)**: Can start in parallel with Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2) completion - BLOCKS other stories (provides core implementation)
- **User Story 2 (Phase 4)**: Depends on User Story 1 (need working implementation to document)
- **User Story 3 (Phase 5)**: Depends on User Story 1 (enhances logging in existing implementation)
- **User Story 4 (Phase 6)**: Depends on User Story 1 (extends config resolution in existing implementation)
- **Polish (Phase 7)**: Depends on all user stories being complete

### Critical Path

```
Phase 1 (Setup) ─────────────┐
                              ├──→ Phase 3 (US1) ──→ Phase 4 (US2) ──┐
Phase 2 (Foundational) ──────┘                                        ├──→ Phase 7 (Polish)
                              ├──→ Phase 5 (US3) ──────────────────┤
                              └──→ Phase 6 (US4) ──────────────────┘
```

### Minimum Viable Product (MVP)

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (User Story 1)

This provides:
- Working `evefrontier-cli mcp` command
- Stdio transport with proper isolation
- Dataset initialization
- Basic logging

Phases 4-6 enhance the MVP with documentation and flexibility but aren't required for basic functionality.

### Parallel Opportunities

**Phase 1 (Setup)**: All tasks T001-T005 marked [P] can run in parallel (different documentation files)

**Phase 2 (Foundational)**: Tasks T006-T010 must run sequentially (Cargo.toml → directory → mod.rs → main.rs imports)

**Phase 3 (User Story 1)**: 
- T011, T012, T013 can run in parallel [P] (different concerns: enum variant, args struct, module file)
- T014-T024 must run in sequence (building up the implementation)
- T025-T027 must run after implementation (testing)

**Phase 4 (User Story 2)**:
- T028-T031 can run in parallel [P] (different documentation files)
- T032-T037 build on documentation incrementally

**Phase 5 (User Story 3)**:
- T038-T047 must run sequentially (building up logging and testing)

**Phase 6 (User Story 4)**:
- T048-T057 must run sequentially (config resolution and testing)

**Phase 7 (Polish)**:
- T058-T062 can run in parallel [P] (independent quality checks)
- T063-T067 must run sequentially (integration validation)

### Parallel Example: If 3 developers available

**Week 1**:
- Dev 1: Phase 2 (Foundational) → Phase 3 (US1 implementation)
- Dev 2: Phase 1 (Setup/Documentation)
- Dev 3: Phase 1 (Setup/Documentation)

**Week 2**:
- Dev 1: Phase 3 (US1 testing) → Phase 4 (US2)
- Dev 2: Phase 5 (US3)
- Dev 3: Phase 6 (US4)

**Week 3**:
- All: Phase 7 (Polish, integration testing, final validation)

### Sequential Implementation (Single developer)

1. Complete Phase 1 (1-2 hours)
2. Complete Phase 2 (30 minutes)
3. Complete Phase 3 - US1 (4-6 hours) ← **MVP CHECKPOINT**
4. Complete Phase 4 - US2 (2-3 hours)
5. Complete Phase 5 - US3 (1-2 hours)
6. Complete Phase 6 - US4 (1-2 hours)
7. Complete Phase 7 - Polish (2-3 hours)

**Total Estimated Effort**: 12-18 hours (single developer, sequential)

---

## Implementation Strategy

### Test-Driven Development

While explicit tests aren't requested in the spec, the implementation follows TDD principles:

1. **Manual smoke tests** after each phase (T027, T037, T047, T057)
2. **Integration test** for stdio isolation (T046)
3. **Build verification** throughout (T025, T061, T062)

### Incremental Delivery

Each user story phase delivers independently testable functionality:

- **After Phase 3**: Can manually test MCP server launch and handshake
- **After Phase 4**: Can integrate with Claude Desktop and ask questions
- **After Phase 5**: Can debug with detailed tracing
- **After Phase 6**: Can customize dataset locations

### Success Criteria Mapping

From spec.md success criteria:

- **SC-001** (`--help` displays): → T026
- **SC-002** (Claude Desktop handshake): → T027, T037
- **SC-003** (All tools callable): → T066 (end-to-end test)
- **SC-004** (Documentation with config): → T029-T031
- **SC-005** (Stdio isolation test): → T046
- **SC-006** (Graceful shutdown): → T020, T040

All success criteria are covered by tasks in this plan.

---

**Total Tasks**: 67  
**Parallelizable Tasks**: 13 (marked with [P])  
**User Story Tasks**: 50 (T011-T057, excluding setup and polish)  
**Estimated Duration**: 12-18 hours (single developer) or 8-10 hours (3 developers in parallel)
