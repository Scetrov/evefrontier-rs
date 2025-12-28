# Implementation Plan: Release & Signing Documentation

**Branch**: `005-release-documentation` | **Date**: 2025-12-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-release-documentation/spec.md`

---

## Summary

Create comprehensive release documentation (`docs/RELEASE.md`) covering GPG tag signing, artifact
signing with cosign, SBOM generation, and the complete release workflow. This is foundational
documentation required before implementing CI release automation (per ADR 0007 and Constitution
v1.1.0).

## Technical Context

**Language/Version**: Documentation (Markdown) with shell command examples  
**Primary Dependencies**: GPG, cosign, cargo-sbom, sha256sum  
**Storage**: N/A  
**Testing**: Manual verification of documented commands  
**Target Platform**: Linux (primary), macOS (secondary)  
**Project Type**: Documentation-only feature  
**Performance Goals**: N/A  
**Constraints**: Must align with Constitution v1.1.0 release policies  
**Scale/Scope**: Single markdown document ~500-800 lines

---

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

| Principle              | Status       | Implementation                |
| ---------------------- | ------------ | ----------------------------- |
| I. TDD                 | ✅ N/A       | Documentation task, no code   |
| II. Library-First      | ✅ N/A       | Documentation task            |
| III. ADR Documentation | ✅ COMPLIANT | Implements ADR 0007 guidance  |
| IV. Clean Code         | ✅ N/A       | Documentation task            |
| V. Security-First      | ✅ COMPLIANT | Documents security procedures |
| VI. Testing Tiers      | ✅ N/A       | No executable code            |
| VII. Refactoring       | ✅ N/A       | New documentation             |

### Constitution Release Requirements (v1.1.0)

From "Versioning & Release Policy":

- ✅ Releases MUST be tagged with GPG signature: `git tag -s vX.Y.Z`
- ✅ Release artifacts MUST be signed with cosign/GPG and include SBOMs
- ✅ Release notes MUST be published on GitHub with migration guidance
- ✅ Releases MUST be tested against the current dataset and Lambda runtime

---

## Gate Evaluation

### Pre-Implementation Gates

| Gate             | Status    | Evidence                             |
| ---------------- | --------- | ------------------------------------ |
| ADR alignment    | ✅ PASSED | ADR 0007 requires this documentation |
| Security review  | ✅ PASSED | Documents security controls          |
| Breaking changes | ✅ NONE   | Documentation only                   |
| Dependencies     | ✅ MET    | GPG already required for commits     |

---

## Project Structure

### Documentation (this feature)

```text
specs/005-release-documentation/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: Tool research
└── quickstart.md        # Phase 1: Release quickstart guide
```

### Source Code (repository root)

```text
docs/
├── RELEASE.md           # PRIMARY OUTPUT: Complete release guide
├── DEPLOYMENT.md        # Existing (cross-reference)
├── USAGE.md             # Existing (cross-reference)
└── TODO.md              # Update to mark task complete
```

**Structure Decision**: Documentation-only feature. Single new file `docs/RELEASE.md` with
cross-references to existing documentation.

---

## Complexity Tracking

No complexity violations. This is straightforward documentation work.

---

## Phase 0: Research (Complete)

See [research.md](./research.md) for detailed findings.

### Key Decisions

| Decision                  | Rationale                                |
| ------------------------- | ---------------------------------------- |
| GPG tag signing           | Matches existing commit signing workflow |
| cosign for binaries       | Adds transparency log, modern approach   |
| cargo-sbom for SBOM       | Native Rust, CycloneDX format            |
| Detached signatures       | Standard practice, smaller artifacts     |
| Both key + keyless cosign | Flexibility for local and CI releases    |

---

## Phase 1: Design & Artifacts (Complete)

### Generated Artifacts

| Artifact                         | Status      | Description                   |
| -------------------------------- | ----------- | ----------------------------- |
| [spec.md](./spec.md)             | ✅ Complete | Feature specification         |
| [research.md](./research.md)     | ✅ Complete | Tool research and decisions   |
| [quickstart.md](./quickstart.md) | ✅ Complete | Abbreviated release checklist |

### docs/RELEASE.md Structure

```markdown
# Release Procedures

## Table of Contents

## Overview

## Prerequisites

### GPG Key Setup

### cosign Installation

### cargo-sbom Installation

## Release Checklist

## Version Bumping

### Semantic Versioning

### Cargo.toml Updates

## Tag Signing

### Creating Signed Tags

### Tag Verification

## Building Release Artifacts

### Binary Compilation

### Cross-Compilation (aarch64)

### Spatial Index Generation

### Package Assembly

## Artifact Signing

### GPG Signatures

### cosign Signatures (Key-Based)

### cosign Signatures (Keyless)

## SBOM Generation

## GitHub Release Creation

## Verification

### For Release Authors

### For Consumers

## CI Integration Notes

## Troubleshooting

## References
```

### Post-Design Constitution Re-Check

| Principle              | Status       | Notes                        |
| ---------------------- | ------------ | ---------------------------- |
| I. TDD                 | ✅ N/A       | Documentation only           |
| II. Library-First      | ✅ N/A       | No code changes              |
| III. ADR Documentation | ✅ COMPLIANT | References ADR 0007          |
| IV. Clean Code         | ✅ N/A       | N/A                          |
| V. Security-First      | ✅ COMPLIANT | Documents signing procedures |
| VI. Testing Tiers      | ✅ N/A       | Manual verification only     |
| VII. Refactoring       | ✅ N/A       | New content                  |

---

## Phase 2: Implementation Tasks

_To be generated by `/speckit.tasks` command_

### Task Overview

| Task ID | Description                               | Est. Effort |
| ------- | ----------------------------------------- | ----------- |
| T1      | Create docs/RELEASE.md with full content  | 2-3 hours   |
| T2      | Update docs/TODO.md to mark task complete | 5 min       |
| T3      | Update CHANGELOG.md                       | 5 min       |
| T4      | PR review and merge                       | 30 min      |

### Implementation Order

1. **T1**: Write `docs/RELEASE.md` following structure above
2. **T2**: Update `docs/TODO.md` checkbox
3. **T3**: Add CHANGELOG entry
4. **T4**: Create PR, review, merge

---

## Stop Point

**Planning phase complete.** This plan is ready for `/speckit.tasks` to generate detailed
implementation tasks.

### Generated Artifacts Summary

| File          | Purpose                         | Location                           |
| ------------- | ------------------------------- | ---------------------------------- |
| spec.md       | Feature requirements            | `specs/005-release-documentation/` |
| plan.md       | Implementation plan (this file) | `specs/005-release-documentation/` |
| research.md   | Tool research findings          | `specs/005-release-documentation/` |
| quickstart.md | Abbreviated release guide       | `specs/005-release-documentation/` |

### Next Steps

1. Run `/speckit.tasks` to generate detailed task breakdown
2. Implement `docs/RELEASE.md`
3. Update TODO.md and CHANGELOG.md
4. Create PR for review
