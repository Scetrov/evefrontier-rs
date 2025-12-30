# Quickstart: GHCR Container Repository Path Standardization

**Date**: 2025-12-30  
**Estimated Time**: 30 minutes

## Overview

This guide provides step-by-step instructions to update all GHCR container registry references from the incorrect `rslater-cs` namespace to the canonical `scetrov/evefrontier-rs` namespace.

## Prerequisites

- Git repository cloned and on feature branch `010-ghcr-container-repository-paths`
- Text editor or IDE with multi-file search/replace
- Helm CLI installed (for validation)

## Implementation Steps

### Step 1: Update CI/CD Workflow

**File**: `.github/workflows/docker-release.yml`

```yaml
# Change from:
  IMAGE_PREFIX: ghcr.io/rslater-cs

# Change to:
  IMAGE_PREFIX: ghcr.io/scetrov/evefrontier-rs
```

Also update/remove the comment on line 39.

### Step 2: Update Helm Chart

**File**: `charts/evefrontier/Chart.yaml`

Update all `rslater-cs` references to `Scetrov`:
- Line 20: `home` URL
- Line 23: `sources` URL  
- Line 27: `maintainers` URL

**File**: `charts/evefrontier/values.yaml`

Update image repositories:
```yaml
# route service (around line 17)
repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route

# scout-gates service (around line 52)
repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates

# scout-range service (around line 79)
repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range
```

**File**: `charts/evefrontier/README.md`

Update example repository path in the configuration table.

### Step 3: Update Cargo.toml Files

Update `repository` field in each service crate:

- `crates/evefrontier-service-route/Cargo.toml`
- `crates/evefrontier-service-scout-gates/Cargo.toml`
- `crates/evefrontier-service-scout-range/Cargo.toml`

```toml
repository = "https://github.com/Scetrov/evefrontier-rs"
```

### Step 4: Update Documentation

**File**: `docs/DEPLOYMENT.md`

Update clone URL:
```bash
git clone https://github.com/Scetrov/evefrontier-rs.git
```

**File**: `docs/RELEASE.md`

Update container image references in the Docker release section:
- Image table entries
- Pull command examples
- Verification script examples

## Validation

### 1. Verify No References Remain

```bash
# Should return no results (except TODO.md tracking item)
grep -r "rslater-cs" --include="*.yml" --include="*.yaml" --include="*.md" --include="*.toml" . | grep -v "TODO.md"
```

### 2. Validate Helm Chart

```bash
helm lint charts/evefrontier
helm template evefrontier charts/evefrontier
```

### 3. Run CI Checks

```bash
# Format and lint
cargo fmt --all -- --check
cargo clippy --workspace

# Verify Cargo.toml parsing
cargo metadata --format-version=1 > /dev/null
```

## Rollback

If issues are discovered after merge:

1. Revert the PR via GitHub UI
2. Previous images remain available at old paths until TTL
3. Update deployments to use reverted paths

## Post-Implementation

1. Mark TODO items as complete in `docs/TODO.md`
2. Update CHANGELOG.md with entry
3. Create PR with descriptive title and link to this spec
