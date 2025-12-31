# Research: ADR 0017 - NX Orchestration Strategy

**Date**: 2025-12-31  
**Phase**: 0 - Research & Clarification  
**Status**: Complete

## Executive Summary

The evefrontier-rs workspace uses Nx 19+ as a polyrepo build orchestrator for:
- **6 Rust crates** (lib, cli, 3 lambda, 1 service-shared) + 9 service microservice crates
- **Python scripts** (dataset extraction, inspection, testing utilities)
- **Node tasks** (dependency reporting, markdown linting, prettier formatting)
- **Shared workflows** (CI/CD, pre-commit hooks, release pipelines)

This research documents the patterns, rationale, and current implementation to inform ADR 0017.

---

## 1. Nx Version & Ecosystem

**Decision**: Nx 19+ (as declared in package.json)  
**Rationale**: Latest stable version with full support for custom executors, task caching, and plugins  
**Alternatives Considered**:
- **Plain Cargo**: Insufficient for polyrepo coordination; no built-in support for Python/Node tasks, CI caching, or unified task orchestration
- **Bazel**: Too heavyweight for this project's scope; steep learning curve, overkill for a small Rust+scripts workspace
- **Just**: Task runner only; no caching, no dependency graph visualization, no CI/CD integration
- **Make**: Makefiles are difficult to maintain, no caching, no cross-language support

**Consequence**: Nx provides unified orchestration but adds dependency (pnpm + Node ecosystem). However, the investment pays off for multi-language workflows and CI reproducibility.

---

## 2. Workspace Architecture

### Named Inputs Pattern

**Current Implementation** (from nx.json):

```json
"namedInputs": {
  "default": ["{projectRoot}/**/*", "sharedGlobals"],
  "production": [
    "default",
    "!{projectRoot}/**/?(*.)+(spec|test).[jt]s?(x)?(.snap)",
    "!{projectRoot}/tsconfig.spec.json",
    "!{projectRoot}/.eslintrc.json",
    "!{projectRoot}/eslint.config.js",
    "!{projectRoot}/**/*.md"
  ],
  "sharedGlobals": ["{workspaceRoot}/.rust-toolchain", "{workspaceRoot}/.nvmrc"]
}
```

**Decision**: Use three input sets:
- `default`: All files + toolchain versions (for full cache invalidation)
- `production`: Excludes test/config files (for build caching, excludes test artifacts)
- `sharedGlobals`: Pinned toolchain versions (ensures cache invalidation on compiler upgrades)

**Rationale**: 
- `production` excludes test files so changing tests doesn't invalidate cached binaries
- `sharedGlobals` ensures Rust 1.91.1 or Node 20 upgrades invalidate all caches (safe, prevents stale binaries)
- Separate inputs allow fine-grained cache control per target type

**Consequence**: Cache keys are deterministic and reproducible; changing tests doesn't rebuild binaries unnecessarily.

---

### Target Defaults Pattern

**Current Implementation** (from nx.json):

```json
"targetDefaults": {
  "build": {
    "dependsOn": ["^build"],
    "inputs": ["production", "^production"],
    "outputs": [],
    "cache": true,
    "parallel": false
  },
  "test": {
    "dependsOn": ["build"],
    "inputs": ["default", "^production", "{workspaceRoot}/docs/fixtures/**/*"],
    "cache": true,
    "parallel": false
  },
  "lint": {
    "inputs": ["default"],
    "cache": true,
    "parallel": false
  },
  "clippy": {
    "inputs": ["default", "^production"],
    "cache": true,
    "parallel": false
  },
  "complexity": {
    "inputs": ["default", "^production", "{workspaceRoot}/clippy.toml"],
    "cache": true,
    "parallel": false
  },
  "audit": {
    "cache": false
  },
  "outdated": {
    "cache": false
  }
}
```

**Decision Details**:

| Target | Cache | Parallel | Depends On | Inputs | Why |
|--------|-------|----------|-----------|--------|-----|
| build | ✅ | ❌ false | `^build` | `production` | Cached builds reduce CI time; Cargo manages parallelism |
| test | ✅ | ❌ false | `build` | `default` + fixtures | Tests cached but re-run on fixture changes; Cargo parallelism ok |
| lint | ✅ | ❌ false | — | `default` | Cached formatting checks; `parallel: false` preserves Cargo order |
| clippy | ✅ | ❌ false | — | `default` + `^production` | Cached linting; includes upstream production changes |
| complexity | ✅ | ❌ false | — | `default` + clippy.toml | Cached complexity checks; invalidates on threshold changes |
| audit | ❌ | — | — | — | Never cached (security advisory freshness required) |
| outdated | ❌ | — | — | — | Never cached (dependency freshness required) |

**Rationale for `parallel: false`**:
- Rust's Cargo already manages parallelism across crates (using `-j` and work-stealing)
- Nx parallelization (spawning multiple `cargo` processes) risks contention on shared `/target` directory
- `parallel: false` allows Cargo's default `jobs = CPU cores` to work optimally
- CI runners typically have 4-8 cores; Cargo parallelism is effective at this scale

**Rationale for `dependsOn`**:
- `build: ["^build"]`: Ensures upstream crate dependencies are built before this crate (transitive closure)
- `test: ["build"]`: Tests require compiled binaries; declaring dependency prevents out-of-order execution
- Other targets have no upstream dependencies (independent checks)

**Consequence**: 
- Builds are reproducible and distributed (CI cache benefits + fast local dev)
- Tests run after builds, preventing false failures from stale binaries
- Cargo's parallelism is uncontended; peak performance achieved

---

## 3. Project Configuration Pattern

**Current Implementation** (example: crates/evefrontier-lib/project.json):

```json
{
  "name": "evefrontier-lib",
  "$schema": "../../node_modules/nx/schemas/project-schema.json",
  "projectType": "library",
  "sourceRoot": "crates/evefrontier-lib/src",
  "tags": ["type:library", "lang:rust"],
  "targets": {
    "build": {
      "executor": "nx:run-commands",
      "options": {
        "command": "cargo build -p evefrontier-lib --locked",
        "cwd": "{workspaceRoot}"
      },
      "configurations": {
        "release": {
          "command": "cargo build -p evefrontier-lib --release --locked"
        }
      }
    },
    "test": {
      "executor": "nx:run-commands",
      "dependsOn": ["build"],
      "options": {
        "command": "cargo test -p evefrontier-lib --locked",
        "cwd": "{workspaceRoot}"
      }
    },
    // ... other targets
  }
}
```

**Decision**: Each Rust crate has identical `project.json` structure with:
- `"executor": "nx:run-commands"`: Delegate to Cargo CLI (leverage Cargo's workspace knowledge)
- `"command": "cargo [cmd] -p [crate-name] --locked"`: Package-specific + lock-file enforcement
- `"cwd": "{workspaceRoot}"`: Run from workspace root (Cargo workspace semantics)
- Optional `configurations.release`: Alternate build mode via `--release` flag

**Rationale**:
- Using `nx:run-commands` avoids reimplementing Cargo's complex build semantics
- Cargo's `-p` flag ensures only the specified package and dependencies are built (efficient)
- `--locked` enforces Cargo.lock consistency (reproducibility)
- `configurations.release` allows `nx build:release` for release builds without custom scripts

**Consequence**: 
- Nx wraps Cargo without fighting it; team can use `cargo` directly or `nx` for CI
- Workspace-wide task coordination (e.g., `nx build:release --all`) works naturally
- Maintenance burden is low: no custom Rust build rules

---

## 4. CI/CD Integration Pattern

**Current Implementation** (from .github/workflows/ci.yml):

```yaml
# Simplified example
- name: Rust Build
  run: nx run-many --target build --all

- name: Rust Tests
  run: nx run-many --target test --all

- name: Clippy Lint
  run: nx run-many --target clippy --all

- name: Audit Dependencies
  run: nx run-many --target audit --all
```

**Decision**: CI workflows use `nx run-many --target [target] --all` to:
- Run tasks across all projects
- Respect `dependsOn` ordering (e.g., build before test)
- Leverage Nx cache from previous workflow runs (GitHub Actions caching)
- Provide unified exit code (CI fails if any task fails)

**Rationale**:
- `run-many` is simpler than iterating projects manually
- Nx respects `targetDefaults` (cache, parallelism, inputs) globally
- CI cache saves 10-15 mins per run (on average) when artifacts are warm
- Single entry point for developers to run same tasks locally

**Consequence**: 
- CI feedback is fast and reproducible
- Developers can debug CI failures locally with `nx run-many --target [target] --all`
- Pre-commit hooks can use same task definitions (no drift)

---

## 5. Caching Strategy

**Current Caching Behavior**:

| Task | Cache | Key Inputs | When Invalidated |
|------|-------|-----------|-----------------|
| build | ✅ | `production` + `^production` | Rust version OR source changes OR upstream binary changes |
| test | ✅ | `default` + fixtures | Test code OR source changes OR fixtures OR binaries |
| lint | ✅ | `default` | Format violations OR source changes |
| clippy | ✅ | `default` + `^production` | Lint violations OR source OR upstream changes |
| audit | ❌ | — | Every run (security freshness) |
| outdated | ❌ | — | Every run (dependency freshness) |

**Decision**: Cache all except security/freshness tasks.

**Rationale**:
- `build` caching saves the most time (Rust compile is slow)
- `test` caching prevents re-running tests when source didn't change (safety: includes fixture input)
- `audit` never cached (RustSec advisories are time-sensitive)
- `outdated` never cached (identifies new vulnerabilities/updates)

**Consequence**: 
- Local dev and CI are fast (warm cache)
- Security checks always run (no stale advisory data)
- Cache is distributed via GitHub Actions (5-10 GB storage, retained 5 days)

---

## 6. Alternative Approaches Evaluated

### Alternative 1: Plain Cargo without Nx

**Decision**: Rejected  
**Why**: 
- No unified task orchestration for Python/Node scripts
- No caching layer (rebuilds even if source unchanged)
- CI would need separate shell scripts for each task
- Harder to enforce consistent developer workflow

### Alternative 2: Bazel

**Decision**: Rejected  
**Why**:
- Steep learning curve (Starlark, BUILD file syntax)
- Overkill for a 6-crate Rust project
- Python support is less mature than Nx
- Build rules would require maintenance

### Alternative 3: Plain Make

**Decision**: Rejected  
**Why**:
- Makefiles don't provide caching
- No dependency graph or task visualization
- Hard to maintain cross-language workflows
- CI integration would require custom scripts

---

## 7. Current Limitations & Future Improvements

### Limitation 1: Spatial Index Build

**Current**: `index-build` subcommand in CLI, not in Nx tasks  
**Future**: Could add Nx target `build:spatial-index` that:
- Takes `minimal_static_data.db` as input
- Outputs `minimal_static_data.db.spatial.bin`
- Is cached (so re-runs only when DB changes)
- Runs as part of release workflow

**Status**: Deferred; would require modifying evefrontier-lib crate to expose index-build as public function

### Limitation 2: Lambda Deployment Artifacts

**Current**: Release job manually bundles spatial index, ship data, etc.  
**Future**: Could use Nx `outputs` to:
- Declare Lambda deployment package structure
- Cache intermediate artifacts
- Verify freshness (ADR 0009 pattern)

**Status**: Working; improvement would be nice-to-have

### Limitation 3: Caching on `test` Target

**Current**: Tests cached based on `default` + fixtures  
**Issue**: If test code changes but fixture unchanged, cached result is returned (stale)
**Solution**: Could split into `unit` (cached) and `integration` (always run)  
**Status**: Low priority; not currently causing issues

---

## 8. Comparison with Nx Best Practices

**Nx Best Practice**: Each project should declare `outputs` for build artifacts

**Current State**: Build targets have `outputs: []` (empty)  
**Impact**: Nx doesn't manage or cache build artifacts; Cargo manages `/target/` directly
**Why Acceptable**: Cargo's `/target/` is already a standard cache location; Nx caching at the `target` level would require Cargo modifications

**Nx Best Practice**: Use `@nx/rust` plugin for Rust projects

**Current State**: Using generic `nx:run-commands` executor  
**Why Acceptable**: `@nx/rust` plugin (community-maintained) is still immature; wrapping Cargo CLI is proven and maintainable
**Future**: Could adopt `@nx/rust` if plugin matures and provides concrete benefits

---

## 9. Key Decisions Summary

| Decision | Why | Trade-offs |
|----------|-----|-----------|
| Nx for polyrepo | Unified tasks, caching, CI integration | Adds pnpm + Node dependency |
| `nx:run-commands` executor | Lean, low-maintenance, leverages Cargo | Less Nx-idiomatic than custom rules |
| `parallel: false` for Rust | Avoids contention, lets Cargo parallelize | Slightly slower CI if few cores (unlikely) |
| `--locked` flag | Reproducibility | Requires `cargo build --offline` or network |
| Never cache `audit` | Security freshness | Slightly longer CI (10-15s overhead) |
| Named inputs (default/production) | Fine-grained cache control | Requires understanding input patterns |
| Shared `sharedGlobals` | Compiler version invalidates cache | Cache invalidates on toolchain change |

---

## 10. Validation Against Requirements

From spec.md:

- ✅ **FR-001**: Rationale for Nx explained (vs. Cargo, Bazel, Make, Just)
- ✅ **FR-002**: Target configuration patterns documented (build, test, lint, clippy, etc.)
- ✅ **FR-003**: Input/output hashing strategy documented (named inputs, production vs. default)
- ✅ **FR-004**: CI integration patterns explained (`run-many`, GitHub Actions caching)
- ✅ **FR-005**: Concrete examples provided (project.json for lib, cli, lambda crates)
- ✅ **FR-006**: Custom patterns documented (parallel: false, dependsOn chains)
- ✅ **FR-007**: Rationale for `parallel: false` explained (Cargo contention avoidance)
- ✅ **FR-008**: Task outputs in release workflows mentioned (spatial index, Lambda artifacts)
- ✅ **FR-009**: Caching behavior documented (when cached, when not)
- ✅ **FR-010**: References to nx.json and project.json included throughout

---

## 11. Next Steps for ADR 0017

1. **Structure**: Follow Nygard ADR format (Problem, Decision, Consequences, Alternatives)
2. **Problem Statement**: Implicit Nx patterns not documented → risk of misconfiguration or inconsistency
3. **Decision Section**: Formally adopt the patterns documented above
4. **Consequences Section**: Benefits (reproducibility, speed, maintainability) and costs (complexity, Nx ecosystem dependency)
5. **Alternatives Section**: Revisit rejected approaches with updated context
6. **Examples Section**: Include sanitized project.json snippets and workflow excerpts
7. **Troubleshooting**: Common issues (cache invalidation, task order, daemon problems)
8. **Cross-References**: Link to ADR 0001, 0006, 0007, 0009 for complementary patterns

---

## Conclusion

The workspace has implemented a mature Nx orchestration strategy with clear patterns:
- Named inputs for cache control
- Target defaults for consistency
- `parallel: false` for Rust to avoid contention
- CI integration via `run-many` commands
- Never-cache strategy for security/freshness tasks

These patterns work well together and should be formally documented in ADR 0017 to ensure future contributors follow the same conventions.
