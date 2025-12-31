# Feature Specification: MCP Server Integration

**Feature Branch**: `016-mcp-server-integration`  
**Created**: 2025-12-31  
**Status**: Draft  
**Input**: User request from TODO.md "MCP Server Integration" section

## Overview

Implement a Model Context Protocol (MCP) server for EVE Frontier that enables AI assistants (Claude
Desktop, VS Code Copilot, Cursor, etc.) to interact with the EVE Frontier static dataset. The MCP
server will expose core library functionality as MCP tools, resources, and prompts, allowing AI
applications to query star systems, plan routes, and explore the EVE Frontier universe.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Route Planning via AI Assistant (Priority: P1)

A user wants to ask their AI assistant "Find a route from Nod to Brana avoiding high-temperature
systems" and receive an actionable route with hop-by-hop navigation.

**Why this priority**: Route planning is the primary use case for the EVE Frontier CLI; exposing it
via MCP makes the tool accessible to a broader audience without requiring CLI knowledge.

**Independent Test**: Can be fully tested by configuring Claude Desktop with the MCP server and
asking for a route between two known systems. Success is receiving a valid route response.

**Acceptance Scenarios**:

1. **Given** the MCP server is running and connected to Claude Desktop, **When** user asks "Plan a
   route from Nod to Brana", **Then** the AI returns a formatted route with system names, distances,
   and hop count.
2. **Given** the MCP server is running, **When** user requests a route with constraints (e.g., "max
   temperature 500K, avoid gate-only paths"), **Then** the server applies constraints and returns a
   filtered route or explains why no route exists.
3. **Given** an invalid system name is provided, **When** the tool is called, **Then** the server
   returns fuzzy match suggestions and an appropriate error message.

---

### User Story 2 - System Information Queries (Priority: P1)

A user wants to ask "What systems are within 50 light-years of Brana?" or "Tell me about the Nod
system" and receive detailed information.

**Why this priority**: System queries complement route planning and enable exploratory use cases
like scouting and strategic planning.

**Independent Test**: Query a known system name and verify the response includes coordinates,
temperature, planets, moons, and connected gates.

**Acceptance Scenarios**:

1. **Given** the MCP server is connected, **When** user asks "What is the temperature of system
   H:2L2S?", **Then** the server returns the system's minimum external temperature.
2. **Given** a spatial index is available, **When** user asks "Find systems within 30 ly of Brana",
   **Then** the server returns a list of nearby systems with distances.
3. **Given** a system has jump gates, **When** user queries the system, **Then** gate-connected
   neighbors are included in the response.

---

### User Story 3 - Dataset Resource Access (Priority: P2)

A developer wants to access raw dataset information (schema, system count, available algorithms) for
building integrations or understanding the data model.

**Why this priority**: Resource access enables advanced users to understand the underlying data
structure without needing to read source code.

**Independent Test**: Request the "dataset://schema" resource and verify it returns table
definitions and column names.

**Acceptance Scenarios**:

1. **Given** the MCP server is running, **When** client requests `resources/list`, **Then** the
   server returns available resources including dataset metadata.
2. **Given** a resource URI like `evefrontier://systems/count`, **When** client requests
   `resources/read`, **Then** the server returns the system count from the loaded dataset.
3. **Given** the spatial index is loaded, **When** client queries resource metadata, **Then** index
   version and build timestamp are included.

---

### User Story 4 - Interactive Prompt Templates (Priority: P3)

A user wants pre-built prompt templates for common EVE Frontier queries to improve AI response
quality.

**Why this priority**: Prompts enhance user experience but are not core functionality; they can be
added after tools and resources are stable.

**Independent Test**: Request `prompts/list` and verify templates are returned. Use a template and
verify it generates well-structured queries.

**Acceptance Scenarios**:

1. **Given** the MCP server supports prompts, **When** client requests `prompts/list`, **Then** a
   list of available prompt templates is returned (e.g., "route_planning", "system_exploration").
2. **Given** the "route_planning" prompt template, **When** user fills in parameters (origin,
   destination), **Then** a well-structured routing query is generated for the AI.

---

### User Story 5 - Docker/Container Deployment (Priority: P2)

An operations team wants to deploy the MCP server in a containerized environment with proper
security hardening for production use.

**Why this priority**: Container deployment enables enterprise and self-hosted usage patterns
critical for adoption.

**Independent Test**: Build and run the Docker image, connect via stdio transport, and verify tool
execution works.

**Acceptance Scenarios**:

1. **Given** the Dockerfile is built, **When** the container starts, **Then** it runs with
   `CAP_DROP=all` and non-root user.
2. **Given** the container is running, **When** an MCP client connects via stdio, **Then**
   initialization handshake completes successfully.
3. **Given** logging is configured, **When** the server processes requests, **Then** logs are
   written to stderr (not stdout) to avoid protocol corruption.

---

### Edge Cases

- What happens when the dataset is not found or corrupted at startup?
- How does the system handle concurrent requests from multiple AI sessions?
- What happens when spatial index is requested but not available?
- How are very long routes (100+ hops) handled without timeout?
- What happens when the MCP protocol version is incompatible?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement MCP server using stdio transport for local process communication
- **FR-002**: System MUST expose `route_plan` tool accepting origin, destination, algorithm, and
  constraint parameters
- **FR-003**: System MUST expose `system_info` tool returning system metadata (coordinates,
  temperature, gates, planets, moons)
- **FR-004**: System MUST expose `systems_nearby` tool for spatial range queries with optional
  temperature filter
- **FR-005**: System MUST expose `gates_from` tool returning gate-connected systems from a given
  system
- **FR-006**: System MUST implement `resources/list` and `resources/read` for dataset metadata
- **FR-007**: System MUST redirect all logging to stderr to prevent stdout protocol corruption
- **FR-008**: System MUST perform fuzzy matching on system names with suggestions on mismatch
- **FR-009**: System MUST return RFC 9457-style error responses for tool failures
- **FR-010**: System MUST support JSON-RPC 2.0 message format per MCP specification

### Non-Functional Requirements

- **NFR-001**: Cold start time MUST be under 5 seconds including dataset load
- **NFR-002**: Tool execution latency MUST be under 500ms for typical queries (p95)
- **NFR-003**: Memory usage MUST not exceed 512MB under normal operation
- **NFR-004**: System MUST support MCP protocol version 2024-11-05 or later

### Key Entities *(include if feature involves data)*

- **MCPServer**: Main server struct managing lifecycle, tool registration, and request dispatch
- **Tool**: Executable function with name, description, JSON Schema for input, and handler function
- **Resource**: Data source with URI scheme, content type, and read handler
- **Prompt**: Reusable template with name, description, and argument schema

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: MCP server successfully initializes and completes handshake with Claude Desktop
- **SC-002**: All 4 core tools (route_plan, system_info, systems_nearby, gates_from) pass
  integration tests
- **SC-003**: Docker container passes security scan with no HIGH or CRITICAL vulnerabilities
- **SC-004**: Response times meet NFR-002 for 95% of requests in load testing
- **SC-005**: Documentation includes working configuration examples for Claude Desktop, VS Code, and
  Cursor
