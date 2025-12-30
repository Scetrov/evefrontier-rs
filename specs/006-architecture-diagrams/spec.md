# Feature Specification: Architecture Diagrams

**Spec ID**: 006  
**Created**: 2025-12-29  
**Status**: Draft  
**Priority**: Medium

## Problem Statement

The EVE Frontier Rust workspace lacks visual documentation of its architecture. New contributors and
future maintainers need to understand:

1. How data flows from download → loading → graph construction → routing
2. The relationship between library crate, CLI, and Lambda components
3. The sequence of operations in key workflows (route planning, spatial queries)
4. The module dependencies within `evefrontier-lib`

Without diagrams, understanding the system requires reading multiple source files and ADRs.

## Goals

1. Create architecture diagrams that visually document the system's data flow and component
   relationships
2. Produce diagrams that are version-controllable (text-based format like Mermaid)
3. Include diagrams in existing documentation without requiring external tools to render
4. Cover the main consumer workflows: CLI route planning, Lambda invocation, spatial index queries

## Non-Goals

1. Exhaustive documentation of every internal function or private type
2. Deployment architecture (Terraform, AWS infrastructure) - covered in `docs/DEPLOYMENT.md`
3. Database schema diagrams (dataset structure is external and not controlled by this repo)
4. UML class diagrams for every struct/enum (Rustdoc serves this purpose)

## Requirements

### Functional Requirements

1. **Component Diagram**: High-level overview showing relationships between:
   - `evefrontier-lib` (core library)
   - `evefrontier-cli` (CLI wrapper)
   - `evefrontier-lambda-*` (Lambda functions)
   - `evefrontier-lambda-shared` (shared Lambda infrastructure)
   - External systems (GitHub releases, SQLite database, spatial index file)

2. **Data Flow Diagram**: Illustrate how data transforms through the system:
   - Download from GitHub → extract → cache
   - Load SQLite → build Starmap → construct Graph
   - Request → routing → response

3. **Sequence Diagrams**: Document key operations:
   - CLI route command execution flow
   - Lambda cold-start initialization
   - Spatial index build and query

4. **Module Dependency Diagram**: Show how `evefrontier-lib` modules depend on each other:
   - `github.rs` → `dataset.rs` → `db.rs` → `graph.rs` → `path.rs` → `routing.rs`
   - `spatial.rs` integration points

### Non-Functional Requirements

1. Diagrams MUST use Mermaid syntax (renders in GitHub, VS Code, and most Markdown viewers)
2. Diagrams MUST be embedded in `docs/ARCHITECTURE.md` (new file)
3. Diagram source MUST be in the same Markdown file (not external files)
4. Diagrams MUST be referenced from `README.md` with a link to `docs/ARCHITECTURE.md`

## Acceptance Criteria

- [x] `docs/ARCHITECTURE.md` exists with at least 4 diagrams (component, data flow, 2 sequence)
- [x] Diagrams render correctly in GitHub Markdown preview
- [x] `README.md` links to `docs/ARCHITECTURE.md` in the documentation section
- [x] Diagrams accurately reflect the current codebase structure
- [x] No external rendering tools required (Mermaid only)

## Technical Notes

### Mermaid Diagram Types to Use

- `graph TD` or `graph LR` for component and data flow diagrams
- `sequenceDiagram` for sequence diagrams
- `flowchart` for detailed process flows

### Files to Reference

When creating diagrams, reference these source files for accuracy:
- `crates/evefrontier-lib/src/lib.rs` - public API exports
- `crates/evefrontier-lib/src/github.rs` - downloader
- `crates/evefrontier-lib/src/dataset.rs` - dataset path resolution
- `crates/evefrontier-lib/src/db.rs` - starmap loading
- `crates/evefrontier-lib/src/graph.rs` - graph construction
- `crates/evefrontier-lib/src/routing.rs` - route planning
- `crates/evefrontier-lib/src/spatial.rs` - KD-tree index
- `crates/evefrontier-cli/src/main.rs` - CLI entry point
- `crates/evefrontier-lambda-shared/src/runtime.rs` - Lambda initialization

## Related Items

- [TODO.md](../../docs/TODO.md): "Add architecture diagrams or sequence diagrams"
- [ADR 0002](../../docs/adrs/0002-workspace-structure.md): Workspace structure decisions
- [ADR 0006](../../docs/adrs/0006-software-components.md): Software component architecture
- [docs/USAGE.md](../../docs/USAGE.md): Usage documentation to complement
