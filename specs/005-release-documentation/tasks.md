# Tasks: Release & Signing Documentation

**Input**: Design documents from `/specs/005-release-documentation/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, quickstart.md ✅

**Tests**: NOT requested - this is a documentation-only feature with no code.

**Organization**: This feature has no user stories in the traditional sense (it's documentation).
Tasks are organized by document sections to enable incremental progress.

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create document skeleton and verify prerequisites

- [x] T001 Create `docs/RELEASE.md` with document header and table of contents structure
- [x] T002 [P] Verify GPG is configured locally with `gpg --list-secret-keys`
- [x] T003 [P] Verify cosign is available or document installation: `cosign version`
- [x] T004 [P] Verify cargo-sbom is installed or document installation: `cargo sbom --help`

---

## Phase 2: Prerequisites Section

**Purpose**: Document all tool setup requirements

- [x] T005 Write "Overview" section in `docs/RELEASE.md` explaining release philosophy
- [x] T006 Write "Prerequisites" introduction listing required tools in `docs/RELEASE.md`
- [x] T007 [P] Write "GPG Key Setup" subsection with key generation and keyserver publication in `docs/RELEASE.md`
- [x] T008 [P] Write "cosign Installation" subsection with install commands in `docs/RELEASE.md`
- [x] T009 [P] Write "cargo-sbom Installation" subsection in `docs/RELEASE.md`

**Checkpoint**: Prerequisites section complete - readers can set up their environment

---

## Phase 3: Core Release Workflow

**Purpose**: Document the main release process (FR-1, FR-2, FR-3)

- [x] T010 Write "Release Checklist" section with pre-release validation steps in `docs/RELEASE.md`
- [x] T011 Write "Version Bumping" section covering semantic versioning and Cargo.toml updates in `docs/RELEASE.md`
- [x] T012 Write "Tag Signing" section with `git tag -s` examples and verification in `docs/RELEASE.md`

**Checkpoint**: Core release workflow documented - users can create signed releases

---

## Phase 4: Build & Artifact Assembly

**Purpose**: Document building release artifacts (FR-4, FR-7)

- [x] T013 Write "Building Release Artifacts" introduction in `docs/RELEASE.md`
- [x] T014 [P] Write "Binary Compilation" subsection with release build commands in `docs/RELEASE.md`
- [x] T015 [P] Write "Cross-Compilation (aarch64)" subsection with cross-rs or target setup in `docs/RELEASE.md`
- [x] T016 [P] Write "Spatial Index Generation" subsection referencing `index-build` command in `docs/RELEASE.md`
- [x] T017 Write "Package Assembly" subsection with tarball creation and checksum generation in `docs/RELEASE.md`

**Checkpoint**: Build process documented - users can create release packages

---

## Phase 5: Signing Procedures

**Purpose**: Document artifact signing (FR-4, FR-5)

- [x] T018 Write "Artifact Signing" introduction in `docs/RELEASE.md`
- [x] T019 [P] Write "GPG Signatures" subsection with SHA256SUMS signing in `docs/RELEASE.md`
- [x] T020 [P] Write "cosign Signatures (Key-Based)" subsection for local releases in `docs/RELEASE.md`
- [x] T021 [P] Write "cosign Signatures (Keyless)" subsection for CI releases in `docs/RELEASE.md`

**Checkpoint**: Signing procedures documented - users can sign artifacts

---

## Phase 6: SBOM & GitHub Release

**Purpose**: Document SBOM generation and release publication (FR-6)

- [x] T022 Write "SBOM Generation" section with cargo-sbom examples in `docs/RELEASE.md`
- [x] T023 Write "GitHub Release Creation" section with artifact upload steps in `docs/RELEASE.md`

**Checkpoint**: Complete release publication documented

---

## Phase 7: Verification & Consumer Documentation

**Purpose**: Document how consumers verify releases (FR-9)

- [x] T024 Write "Verification" introduction in `docs/RELEASE.md`
- [x] T025 [P] Write "Tag Verification" subsection with `git tag -v` in `docs/RELEASE.md`
- [x] T026 [P] Write "Artifact Verification" subsection with GPG and cosign verification in `docs/RELEASE.md`
- [x] T027 [P] Write "SBOM Inspection" subsection with CycloneDX tools in `docs/RELEASE.md`

**Checkpoint**: Consumers can verify release integrity

---

## Phase 8: CI Integration & Reference

**Purpose**: Document CI patterns for future automation (FR-8, FR-10)

- [x] T028 Write "CI Integration Notes" section with GitHub Actions patterns in `docs/RELEASE.md`
- [x] T029 Write "Troubleshooting" section with common issues and solutions in `docs/RELEASE.md`
- [x] T029.5 Write "Rollback & Revocation" section documenting how to yank releases and revoke signatures in `docs/RELEASE.md` (FR-10)
- [x] T030 Write "References" section linking to Constitution, ADR 0007, and external docs in `docs/RELEASE.md`

**Checkpoint**: Complete document ready for review

---

## Phase 9: Polish & Finalization

**Purpose**: Review, validate, and update tracking

- [x] T031 Review `docs/RELEASE.md` for completeness against spec.md requirements
- [x] T032 [P] Run markdown linting on `docs/RELEASE.md`
- [x] T033 [P] Test documented commands execute correctly (minimum: `git tag -s`, `gpg --verify`, `sha256sum -c`)
- [x] T034 Update `docs/TODO.md` to mark release documentation task complete
- [x] T035 Add CHANGELOG.md entry for release documentation feature
- [x] T036 Create PR with all changes for review

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)
    ↓
Phase 2 (Prerequisites) ← T005-T009 can run in parallel
    ↓
Phase 3 (Core Workflow) ← Sequential: T010 → T011 → T012
    ↓
Phase 4 (Build) ← T014-T016 can run in parallel after T013
    ↓
Phase 5 (Signing) ← T019-T021 can run in parallel after T018
    ↓
Phase 6 (SBOM & GitHub) ← T022 → T023 sequential
    ↓
Phase 7 (Verification) ← T025-T027 can run in parallel after T024
    ↓
Phase 8 (CI & Reference) ← T028-T030 sequential or parallel
    ↓
Phase 9 (Polish) ← T032-T035 can run in parallel after T031
```

### Parallel Opportunities

**Within Single Session**:
- T002, T003, T004 (verification tasks)
- T007, T008, T009 (tool setup subsections)
- T014, T015, T016 (build subsections)
- T019, T020, T021 (signing subsections)
- T025, T026, T027 (verification subsections)
- T032, T033, T034, T035 (finalization tasks)

---

## Implementation Strategy

### Single Document Approach

This feature produces a single output file (`docs/RELEASE.md`). The phased approach allows:

1. **Incremental progress**: Each phase adds a complete section
2. **Early validation**: Checkpoints after each phase to verify accuracy
3. **Parallel writing**: Subsections within a phase can be written independently

### Recommended Flow

1. **T001-T004**: Set up document and verify tools (15 min)
2. **T005-T009**: Prerequisites section (30 min)
3. **T010-T012**: Core workflow (30 min)
4. **T013-T017**: Build process (45 min)
5. **T018-T021**: Signing procedures (30 min)
6. **T022-T023**: SBOM and GitHub (20 min)
7. **T024-T027**: Verification (30 min)
8. **T028-T030**: CI and references (20 min)
9. **T031-T036**: Polish and finalize (30 min)

**Total estimated time**: 4-5 hours

---

## Notes

- All tasks modify `docs/RELEASE.md` unless otherwise specified
- Commands documented should be tested against the actual repository
- Cross-reference Constitution v1.1.0 and ADR 0007 where appropriate
- Use code blocks with shell syntax highlighting for commands
- Include both Linux and macOS variations where they differ
