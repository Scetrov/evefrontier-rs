# Tasks: Architecture Diagrams

**Input**: Design documents from `/specs/006-architecture-diagrams/`
**Prerequisites**: plan.md (required), spec.md (required), research.md (component inventory and flow analysis)

**Tests**: Not applicable - this is a documentation-only feature with visual verification.

**Organization**: Tasks are organized by diagram type rather than user stories since this is a documentation feature with a single deliverable file.

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different sections, no dependencies)
- Include exact file paths in descriptions

## Path Conventions

- Documentation at repository root: `docs/`
- Primary deliverable: `docs/ARCHITECTURE.md`
- Updates to existing files: `README.md`, `docs/TODO.md`

---

## Phase 1: Setup

**Purpose**: Create the documentation file structure

- [x] T001 Create `docs/ARCHITECTURE.md` with document header, table of contents, and introduction section

---

## Phase 2: Foundational Diagrams

**Purpose**: Create the high-level diagrams that establish the system architecture context

- [x] T002 Create Component Overview Diagram in `docs/ARCHITECTURE.md` showing crate relationships (evefrontier-lib, evefrontier-cli, Lambda crates, external systems)
- [x] T003 Create Module Dependency Diagram in `docs/ARCHITECTURE.md` showing evefrontier-lib internal module dependencies (github.rs → dataset.rs → db.rs → graph.rs → path.rs → routing.rs, spatial.rs integration)

**Checkpoint**: Core architecture diagrams complete - process flow diagrams can now be added

---

## Phase 3: Data Flow Diagrams

**Purpose**: Document how data transforms through the system

- [x] T004 Create Dataset Download Flow Diagram in `docs/ARCHITECTURE.md` showing ensure_dataset → github.rs → cache → DatasetPaths flow
- [x] T005 Create Starmap Load Flow Diagram in `docs/ARCHITECTURE.md` showing SQLite → schema detection → Starmap → Graph construction
- [x] T006 Create Route Planning Flow Diagram in `docs/ARCHITECTURE.md` showing RouteRequest → plan_route → algorithm selection → pathfinding → RoutePlan

**Checkpoint**: Data transformation flows documented

---

## Phase 4: Sequence Diagrams

**Purpose**: Document time-ordered interactions for key operations

- [x] T007 Create CLI Route Command Sequence Diagram in `docs/ARCHITECTURE.md` showing User → CLI → ensure_dataset → load_starmap → plan_route → output formatting
- [x] T008 Create Lambda Cold-Start Sequence Diagram in `docs/ARCHITECTURE.md` showing AWS → init_runtime → include_bytes → rusqlite → load_starmap_from_connection → SpatialIndex::load_from_bytes → OnceLock storage

**Checkpoint**: All required diagrams complete

---

## Phase 5: Integration & Polish

**Purpose**: Cross-references and completion

- [x] T009 [P] Add "See Also" section to `docs/ARCHITECTURE.md` linking to related docs (USAGE.md, ADRs, DEPLOYMENT.md)
- [x] T010 [P] Update `README.md` to add link to `docs/ARCHITECTURE.md` in documentation section
- [x] T011 Mark "Add architecture diagrams or sequence diagrams" task complete in `docs/TODO.md`
- [x] T012 Verify all diagrams render correctly in VS Code Markdown preview
- [x] T013 Commit changes and verify diagrams render correctly in GitHub Markdown preview

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - creates empty file structure
- **Foundational (Phase 2)**: Depends on Setup - creates context diagrams first
- **Data Flow (Phase 3)**: Depends on Foundational - builds on component understanding
- **Sequence (Phase 4)**: Depends on Data Flow - references established flows
- **Polish (Phase 5)**: Depends on all diagrams complete

### Parallel Opportunities

Phase 2 can be parallelized:
```
Task T002: Component Overview Diagram
Task T003: Module Dependency Diagram
```

Phase 3 can be parallelized:
```
Task T004: Dataset Download Flow
Task T005: Starmap Load Flow  
Task T006: Route Planning Flow
```

Phase 4 can be parallelized:
```
Task T007: CLI Route Sequence
Task T008: Lambda Cold-Start Sequence
```

Phase 5 tasks T009, T010 can be parallelized (different files).

---

## Implementation Strategy

### Recommended Execution Order

1. **T001**: Create file structure with ToC
2. **T002 + T003 in parallel**: High-level architecture diagrams
3. **T004 + T005 + T006 in parallel**: Data flow diagrams
4. **T007 + T008 in parallel**: Sequence diagrams
5. **T009 + T010 in parallel**: Cross-references
6. **T011**: Update TODO.md
7. **T012 + T013**: Visual verification

### Total Task Count: 13

| Phase | Tasks | Parallel Opportunities |
|-------|-------|------------------------|
| Setup | 1 | None |
| Foundational | 2 | 2 tasks |
| Data Flow | 3 | 3 tasks |
| Sequence | 2 | 2 tasks |
| Polish | 5 | 2 tasks |

### Estimated Effort

- Documentation creation only - no code changes
- Each diagram: 10-15 minutes to design and verify
- Total: ~2-3 hours for complete implementation

---

## Validation Checklist

Before marking tasks complete:

- [x] All diagrams use Mermaid syntax (no external tools)
- [x] Diagrams render in VS Code Markdown preview
- [x] Diagrams render in GitHub Markdown preview
- [x] Node names match actual crate/module names from research.md
- [x] Arrows accurately represent dependencies from code analysis
- [x] Table of contents matches section headings
- [x] Links in README.md and cross-references work

---

## Notes

- Reference `specs/006-architecture-diagrams/research.md` for component inventory and flow analysis
- Reference `specs/006-architecture-diagrams/quickstart.md` for Mermaid style guidelines
- Diagrams should be comprehensive but not overwhelming - focus on main flows
- Use subgraphs to group related components for readability
- Keep diagram complexity manageable for GitHub rendering limits
