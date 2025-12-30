# Tasks: GHCR Container Repository Path Standardization

**Input**: Design documents from `/specs/010-ghcr-container-repository-paths/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅

**Tests**: No tests required - validation via CI pipeline and grep verification

**Organization**: Tasks are grouped by file category for logical batch processing

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Verify current state and prepare for changes

- [x] T001 Run `grep -r "rslater-cs" . --include="*.yml" --include="*.yaml" --include="*.md" --include="*.toml" | grep -v "TODO.md" | wc -l` to confirm baseline count (expect ~18)
- [x] T002 [P] Create backup reference list of all `rslater-cs` occurrences for validation

---

## Phase 2: CI/CD Workflow Updates

**Purpose**: Update GitHub Actions workflow to use correct image prefix

**Goal**: CI/CD builds will push images to correct GHCR namespace

**Independent Test**: Run `grep "rslater-cs" .github/workflows/docker-release.yml` - should return 0 matches

- [x] T003 Update `IMAGE_PREFIX` in `.github/workflows/docker-release.yml` line 40 from `ghcr.io/rslater-cs` to `ghcr.io/scetrov/evefrontier-rs`
- [x] T004 Remove or update comment on line 39 referencing `rslater-cs` namespace in `.github/workflows/docker-release.yml`

**Checkpoint**: CI workflow now targets correct GHCR namespace ✅

---

## Phase 3: Helm Chart Updates

**Purpose**: Update Helm chart with correct repository defaults and metadata

**Goal**: `helm install` uses correct image repositories by default

**Independent Test**: Run `helm template evefrontier charts/evefrontier | grep "ghcr.io"` - all should show `scetrov/evefrontier-rs`

### Chart Metadata

- [x] T005 [P] Update `home` URL in `charts/evefrontier/Chart.yaml` line 20 from `rslater-cs` to `Scetrov`
- [x] T006 [P] Update `sources` URL in `charts/evefrontier/Chart.yaml` line 23 from `rslater-cs` to `Scetrov`
- [x] T007 [P] Update `maintainers` URL in `charts/evefrontier/Chart.yaml` line 27 from `rslater-cs` to `Scetrov`

### Image Repositories

- [x] T008 [P] Update route image repository in `charts/evefrontier/values.yaml` line 17 from `ghcr.io/rslater-cs/evefrontier-service-route` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route`
- [x] T009 [P] Update scout-gates image repository in `charts/evefrontier/values.yaml` line 52 from `ghcr.io/rslater-cs/evefrontier-service-scout-gates` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates`
- [x] T010 [P] Update scout-range image repository in `charts/evefrontier/values.yaml` line 79 from `ghcr.io/rslater-cs/evefrontier-service-scout-range` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range`

### Chart Documentation

- [x] T011 [P] Update example repository path in `charts/evefrontier/README.md` line 66 from `ghcr.io/rslater-cs/evefrontier-service-<name>` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-<name>`

**Checkpoint**: Helm chart lint passes with `helm lint charts/evefrontier` ✅

---

## Phase 4: Cargo.toml Updates

**Purpose**: Update service crate repository metadata for crates.io publishing

**Goal**: `cargo metadata` shows correct repository URLs

**Independent Test**: Run `grep "rslater-cs" crates/*/Cargo.toml` - should return 0 matches

- [x] T012 [P] Update `repository` field in `crates/evefrontier-service-route/Cargo.toml` line 8 from `rslater-cs` to `Scetrov`
- [x] T013 [P] Update `repository` field in `crates/evefrontier-service-scout-gates/Cargo.toml` line 8 from `rslater-cs` to `Scetrov`
- [x] T014 [P] Update `repository` field in `crates/evefrontier-service-scout-range/Cargo.toml` line 8 from `rslater-cs` to `Scetrov`

**Checkpoint**: `cargo metadata --format-version=1` succeeds ✅

---

## Phase 5: Documentation Updates

**Purpose**: Update documentation with correct paths for users

**Goal**: Documentation examples use correct, copy-pasteable commands

**Independent Test**: Run `grep "rslater-cs" docs/*.md` - should return 0 matches

### Deployment Documentation

- [x] T015 Update clone URL in `docs/DEPLOYMENT.md` line 42 from `github.com/rslater-cs/evefrontier-rs.git` to `github.com/Scetrov/evefrontier-rs.git`

### Release Documentation

- [x] T016 [P] Update container image table in `docs/RELEASE.md` lines 771-773 (route, scout-gates, scout-range) from `ghcr.io/rslater-cs/...` to `ghcr.io/scetrov/evefrontier-rs/...`
- [x] T017 [P] Update pull example in `docs/RELEASE.md` line 805 from `ghcr.io/rslater-cs/evefrontier-service-route:v0.1.0` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route:v0.1.0`
- [x] T018 [P] Update verification script in `docs/RELEASE.md` line 813 from `ghcr.io/rslater-cs/evefrontier-service-${svc}:v0.1.0` to `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-${svc}:v0.1.0`

**Checkpoint**: Documentation renders correctly with valid URLs ✅

---

## Phase 6: Polish & Validation

**Purpose**: Verify all changes complete and update tracking

- [x] T019 Run `grep -r "rslater-cs" . --include="*.yml" --include="*.yaml" --include="*.md" --include="*.toml" | grep -v "TODO.md"` - confirm 0 matches
- [x] T020 Run `helm lint charts/evefrontier` - confirm lint passes
- [x] T021 Run `cargo fmt --all -- --check` - confirm formatting OK
- [x] T022 Run `cargo clippy --workspace` - confirm no new warnings
- [x] T023 Mark TODO items complete in `docs/TODO.md` (lines 8-10)
- [x] T024 Add entry to `CHANGELOG.md` under Unreleased section

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) → Validates baseline
       ↓
Phases 2-5 can run in PARALLEL (independent files)
       ↓
Phase 6 (Validation) → Confirms all changes complete
```

### Parallel Opportunities by Phase

**Phase 2 (CI/CD)**: T003-T004 must be sequential (same file)

**Phase 3 (Helm)**: 
- T005, T006, T007 can run in parallel (same file, different lines - manual merge)
- T008, T009, T010 can run in parallel (same file, different lines - manual merge)  
- T011 can run in parallel with T005-T010 (different file)

**Phase 4 (Cargo)**: T012, T013, T014 can all run in parallel (different files)

**Phase 5 (Docs)**: 
- T015 can run in parallel with T016-T018 (different files)
- T016, T017, T018 must be sequential (same file)

**Phase 6 (Validation)**: T019-T022 can run in parallel, T023-T024 must be last

---

## Parallel Example: Maximum Parallelism

```bash
# After Phase 1 completes, these can all start simultaneously:

# Team Member A: CI/CD
T003: Update IMAGE_PREFIX in .github/workflows/docker-release.yml
T004: Update comment in .github/workflows/docker-release.yml

# Team Member B: Helm Chart  
T005-T011: All Helm chart updates

# Team Member C: Cargo.toml
T012-T014: All three Cargo.toml updates (fully parallel - different files)

# Team Member D: Documentation
T015-T018: All documentation updates
```

---

## Implementation Strategy

### Recommended: Single Pass

Since this is a simple find-replace task, recommended approach:

1. Complete T001-T002 (Setup/baseline)
2. Execute all file updates (T003-T018) in a single editing session
3. Run validation (T019-T022)
4. Update tracking documents (T023-T024)
5. Commit all changes atomically

### Total Task Count: 24 tasks

| Phase | Tasks | Parallelizable |
|-------|-------|----------------|
| Setup | 2 | 1 |
| CI/CD | 2 | 0 |
| Helm | 7 | 6 |
| Cargo | 3 | 3 |
| Docs | 4 | 3 |
| Validation | 6 | 4 |

---

## Notes

- All `[P]` tasks target different files or non-overlapping lines
- This is a configuration-only change - no production code modified
- Validation uses existing CI tooling (helm lint, cargo fmt, cargo clippy)
- Single atomic PR recommended for auditability
