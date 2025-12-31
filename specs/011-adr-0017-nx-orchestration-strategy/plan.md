# Implementation Plan: ADR 0017 - NX Repository Orchestration Strategy

**Branch**: `011-adr-0017-nx-orchestration-strategy` | **Date**: 2025-12-31 | **Spec**: `specs/011-adr-0017-nx-orchestration-strategy/spec.md`
**Input**: Formalize implicit Nx orchestration patterns already implemented in the workspace

## Summary

Create ADR 0017 documenting the rationale, patterns, and trade-offs for using Nx to orchestrate Rust build, test, lint, and related tasks across the multi-crate workspace. The ADR will explain why Nx was chosen, how targets are configured (task defaults, named inputs, caching strategy), how this integrates with CI/CD workflows, and provide concrete examples for new contributors to follow when adding tasks or crates.

This is a documentation/formalization task for architecture already implemented; no code changes are required.

## Technical Context

**Language/Version**: Markdown documentation (following Nygard ADR format per ADR 0001)  
**Primary Dependencies**: N/A (documentation only)  
**Storage**: File-based (markdown in `docs/adrs/0017-nx-orchestration-strategy.md`)  
**Testing**: Manual review and validation against actual nx.json / project.json files  
**Target Platform**: Repository documentation (not deployed code)  
**Project Type**: Documentation artifact  
**Performance Goals**: N/A  
**Constraints**: Must be concise (<15 pages) and accessible to developers without deep Nx expertise  
**Scale/Scope**: Single ADR document covering workspace-wide Nx configuration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Violations**: None anticipated
- ✅ Library-First Architecture: N/A (documentation, not code)
- ✅ Test-Driven Development: Tests will validate ADR examples against actual configuration
- ✅ Schema Awareness: N/A
- ✅ Error Handling: N/A
- ✅ Observability: N/A (documentation may reference logging/tracing architecture)
- ✅ Security/Compliance: Follows ADR immutability rules from ADR 0001 and branch protection policies
- ✅ Branching & PR Workflow: Using feature branch per constitution (011-xxx)

## Project Structure

### Documentation (this feature)

```text
specs/011-adr-0017-nx-orchestration-strategy/
├── plan.md          # This file
├── spec.md          # Feature specification
├── research.md      # Phase 0 output (if needed for research)
└── tasks.md         # Phase 2: Implementation task checklist
```

### Output Deliverable

```text
docs/adrs/
└── 0017-nx-orchestration-strategy.md    # ADR document following Nygard format
```

### Related Configuration Files (referenced in ADR)

```text
Root workspace:
├── nx.json                              # Workspace-wide task configuration
├── .github/workflows/ci.yml             # CI integration examples
├── .github/workflows/release.yml        # Release workflow integration
└── CONTRIBUTING.md                      # Developer guidance (to be updated)

Per-crate examples:
├── crates/evefrontier-lib/project.json
├── crates/evefrontier-cli/project.json
├── crates/evefrontier-lambda-route/project.json
├── crates/evefrontier-service-route/project.json
└── scripts/project.json
```

**Structure Decision**: This is a documentation-only task. The ADR will be a single markdown file in `docs/adrs/` following the Nygard format established in ADR 0001. The ADR will reference (but not modify) existing nx.json and project.json files to ground its discussion in concrete examples. Updates to CONTRIBUTING.md and AGENTS.md will link to this ADR for task configuration guidance.

## Phase Breakdown

### Phase 0: Research (if needed)
- Validate that no ADR 0017 draft exists yet (confirmed)
- Gather Nx version, documentation links, and team context
- Review existing `.github/workflows/` to understand CI integration patterns
- Document any gaps between current implementation and Nx best practices

### Phase 1: Design & Documentation
- Create ADR 0017 draft with:
  - Problem statement (implicit Nx knowledge)
  - Decision (chosen configuration patterns)
  - Consequences (reproducibility, maintainability, performance trade-offs)
  - Alternatives considered (plain Cargo, Bazel, other tools)
  - Implementation details with concrete examples
  - Troubleshooting guide
- Include cross-references to related ADRs (0001, 0006, 0007)
- Validate examples against actual nx.json and project.json

### Phase 2: Integration & Review
- Update CONTRIBUTING.md to reference ADR 0017 for task configuration
- Update AGENTS.md to reference ADR 0017
- Create pull request with ADR and documentation updates
- Obtain approval per ADR 0001 governance

## Complexity Tracking

No violations anticipated; this is documentation formalization of existing architecture.
