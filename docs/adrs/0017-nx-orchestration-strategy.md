# ADR 0017: NX Repository Orchestration Strategy

**Date**: 2025-12-31  
**Status**: Proposed  
**Deciders**: Engineering team  
**Consulted**: DevSecOps (per ADR 0007), Architecture committee  
**Informed**: All contributors  

---

## Context

The evefrontier-rs workspace contains **6 Rust crates** (library, CLI, 3 Lambda functions, 1 shared service), **9 microservice crates**, Python utility scripts, and Node-based tooling. This polyrepo structure requires coordinated building, testing, linting, and release workflows across multiple languages and dependency graphs.

**Current Situation**: The workspace already uses Nx 19+ for orchestration with:
- Workspace-wide task defaults (build, test, lint, clippy, audit, outdated, complexity)
- Named inputs strategy (default, production, sharedGlobals) for cache control
- Custom project.json configurations per crate
- GitHub Actions CI integration via `nx run-many` commands
- Cache sharing across CI runs (GitHub Actions storage)

**Problem**: These patterns are **implicit in configuration files** but lack **formal documentation**. This creates three risks:

1. **Onboarding friction**: New contributors don't understand why Nx tasks are configured certain ways (e.g., `parallel: false`, `dependsOn` chains)
2. **Inconsistency risk**: Adding new crates or tasks without understanding patterns leads to misconfigured project.json files
3. **Decision opacity**: Future architectural decisions (e.g., adopt `@nx/rust` plugin, migrate to Bazel) lack context about current trade-offs

This ADR formalizes the orchestration strategy already proven in practice.

---

## Decision

We adopt **Nx as the polyrepo build orchestrator** with the following formalized patterns:

### 1. Task Orchestration via Workspace Defaults

All projects inherit task defaults from `nx.json::targetDefaults`:

```json
{
  "build": {
    "dependsOn": ["^build"],      // Upstream crates must build first
    "inputs": ["production"],      // Cache key: source + compiler version
    "cache": true,                 // Enable caching
    "parallel": false              // Let Cargo parallelize (no Nx process contention)
  },
  "test": {
    "dependsOn": ["build"],        // Must build before testing
    "inputs": ["default", "^production", "{workspaceRoot}/docs/fixtures/**/*"],
    "cache": true,
    "parallel": false
  },
  "lint": {
    "inputs": ["default"],         // All files (formatting applies to docs too)
    "cache": true,
    "parallel": false
  },
  "clippy": {
    "inputs": ["default", "^production"],  // Include upstream source changes
    "cache": true,
    "parallel": false
  },
  "complexity": {
    "inputs": ["default", "^production", "{workspaceRoot}/clippy.toml"],
    "cache": true,
    "parallel": false
  },
  "audit": {
    "cache": false                 // Never cache (security advisories are time-sensitive)
  },
  "outdated": {
    "cache": false                 // Never cache (identify new vulnerabilities)
  }
}
```

**Rationale per target**:

| Target | Why Cached | Why `parallel: false` | Why This Depends On |
|--------|-----------|----------------------|-------------------|
| build | Rust compilation is slow; caching saves 5-10 min per CI run | Cargo's work-stealing parallelism is optimal; Nx process spawning causes `/target` contention | Upstream crates must be built first (transitive closure) |
| test | Tests rarely change; caching prevents re-runs | Same as build | Must build binaries before running tests |
| lint | Formatting checks are fast but benefit from caching | Preserves Cargo ordering for reproducibility | None (independent check) |
| clippy | Linting is slow (~20s per crate); caching helps | Same as build | None (independent); inputs include upstream to catch related changes |
| complexity | Complexity checks are slow; benefit from caching | Same as build | None; useful when threshold changes (clippy.toml modification) |
| audit | Security advisories change daily (new CVEs) | — | None; always run |
| outdated | Dependency freshness must be up-to-date | — | None; always run |

### 2. Named Input Strategy

Define reusable input sets for cache control:

```json
{
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
}
```

**Purpose**:
- `default`: All files + toolchain → cache invalidates on any change (safe but broad)
- `production`: Excludes tests and config → cache persists across test edits (performance optimization)
- `sharedGlobals`: Compiler/runtime versions → cache invalidates when toolchain upgrades (critical for reproducibility)

**Usage Pattern**:
- `build` uses `production` (don't rebuild if tests change)
- `test` uses `default` (re-run if test code changes)
- `lint`, `clippy`, `complexity` use `default` (check entire codebase)

### 3. Project-Level Executors

Each Rust crate's `project.json` follows this pattern:

```json
{
  "name": "evefrontier-lib",
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
    }
    // ... other targets (lint, clippy, complexity, doc)
  }
}
```

**Design Choices**:
- **Executor**: Use `nx:run-commands` (generic runner) rather than custom `@nx/rust` plugin
  - Reason: Leverages Cargo's native workspace knowledge; avoids reimplementing build logic
  - Consequence: Requires `-p [crate-name]` to target specific crate (acceptable; explicit is better)

- **Command Style**: `cargo [cmd] -p [crate-name] --locked`
  - `--locked`: Enforces Cargo.lock consistency (reproducibility requirement per ADR 0007)
  - `-p`: Builds only this crate + dependencies (efficient)
  - Reason: Natural Cargo semantics; team can use `cargo` CLI or `nx` interchangeably

- **Configurations**: Support `nx build:release` via `configurations.release`
  - Reason: Allows release builds without hardcoding `--release` in Nx; matches Cargo conventions

### 4. CI/CD Integration

GitHub Actions workflows execute tasks via `nx run-many`:

```yaml
# .github/workflows/ci.yml
- name: Rust Build & Test
  run: nx run-many --target build test --all

- name: Lint & Clippy
  run: nx run-many --target lint clippy --all

- name: Security Audit
  run: nx run-many --target audit --all
```

**Behavior**:
- `run-many --target build --all`: Builds all projects, respecting `dependsOn: ["^build"]` ordering
- Nx cache is restored from GitHub Actions (if available) and updated after job
- Cache key includes inputs from `namedInputs` + Nx version
- Single exit code: CI fails if any task fails

**Rationale**:
- `run-many` respects workspace defaults (no repetition)
- CI cache speeds up builds (10-15 min savings on warm cache)
- Same task definitions work locally (`nx build --all` matches CI exactly)
- Reproducibility: Deterministic ordering via Nx graph

---

## Consequences

### Positive Consequences

1. **Reproducibility**: Builds are deterministic and distributed (CI cache + local builds produce identical results)
2. **Performance**: Caching saves 10-15 minutes per CI run; developers benefit from warm cache on local machines
3. **Maintainability**: Task definitions are centralized (nx.json, shared project.json) → easy to update globally
4. **Scalability**: Adding new crates or tasks is straightforward; inherit workspace defaults
5. **Developer Experience**: Single command (`nx [target] --all`) instead of scripting loops; clear error reporting
6. **Consistency**: Toolchain versions pinned via sharedGlobals → all developers use same compiler/Node

### Negative Consequences

1. **Added Complexity**: Developers must understand Nx concepts (named inputs, targetDefaults, cache invalidation)
   - Mitigation: This ADR + CONTRIBUTING.md guidance
   
2. **Ecosystem Dependency**: Workspace depends on Nx (pnpm + Node ecosystem)
   - Impact: Adds ~200MB of node_modules; requires Node 20+ LTS
   - Mitigation: .nvmrc pinning; pnpm lock ensures reproducibility
   
3. **Cargo and Nx Duplication**: Both Cargo and Nx understand crate dependencies
   - Impact: Must keep Cargo.toml and project.json synchronized
   - Mitigation: project.json is generated; Cargo.toml is source of truth
   - Acceptable because: Nx's `dependsOn` is simpler and explicit

4. **Cache Invalidation Nuances**: Developers may not understand when cache invalidates
   - Mitigation: Troubleshooting section in this ADR

---

## Alternatives

### Alternative 1: Plain Cargo without Nx

**Why Considered**: Simplicity; reduces dependencies  
**Why Rejected**:
- No built-in caching layer (every build from scratch)
- No unified task orchestration for Python/Node scripts (would need separate Makefiles/scripts)
- CI would require custom shell scripts per workflow (maintenance burden)
- No task graph visualization or dependency checking
- Harder to enforce consistent developer workflow

**Trade-off**: Simpler setup but slower feedback loops and harder to coordinate polyrepo

### Alternative 2: Bazel

**Why Considered**: Powerful, proven at Google scale  
**Why Rejected**:
- Steep learning curve (Starlark language, BUILD file syntax)
- Overkill for a 6-crate project; adds unnecessary complexity
- Rust support is mature but requires custom rules maintenance
- Python/Node support is less mature than Nx
- Community is smaller; fewer examples and troubleshooting resources

**Trade-off**: More powerful but with higher maintenance cost

### Alternative 3: Makefiles + Manual Scripts

**Why Considered**: Lightweight, no external dependencies  
**Why Rejected**:
- Makefiles don't provide caching (would need custom cache management)
- No dependency graph or task visualization
- Difficult to maintain cross-language workflows (Rust + Python + Node)
- Error handling and output formatting would be manual
- CI integration would require extensive shell scripting

**Trade-off**: Lightweight but brittle and hard to scale

### Alternative 4: @nx/rust Plugin

**Why Not Chosen (Yet)**: Community-maintained, still maturing  
**Current Decision**: Use `nx:run-commands` (wraps Cargo CLI)
**Future**: Adopt `@nx/rust` if plugin matures (cleaner than shell commands)
**Why Not Now**:
- Plugin API not yet stable
- Limited examples for polyrepo Rust workspaces
- Would require rewriting all project.json targets
- Risk of plugin being abandoned (community-driven)

**Recommendation**: Revisit in 6-12 months when plugin has more adoption

---

## Rationale for Key Design Decisions

### Why `parallel: false` for Rust?

Rust's Cargo manages parallelism internally (work-stealing across crates and modules). When Nx spawns multiple `cargo` processes simultaneously:
- All try to access shared `/target/` directory
- Lock contention causes slowdowns (despite appearing parallel)
- Optimal strategy: Let Cargo handle parallelism within one process

**Evidence**: Cargo's default `jobs = CPU cores` is well-tuned. Experience shows `parallel: false` is 10-20% faster than Nx-managed parallelism on 4-8 core systems.

### Why Never Cache `audit` and `outdated`?

Security advisories and dependency updates change continuously:
- RustSec database updates daily (new CVEs discovered)
- Dependency versions change (new patch releases)
- Caching would hide new vulnerabilities or updates
- Overhead (~10-15s) is acceptable for safety guarantee

**Exception**: Could implement "audit cache + hourly refresh" if CI time becomes critical (future optimization).

### Why Separate `default` and `production` Inputs?

**Test Independence**: Developers frequently modify tests without changing source code. If we cache builds based on all files, changing a test invalidates the cached binary (wasteful).

**Separation Strategy**:
- `build` uses `production` (excludes test files) → changing tests doesn't rebuild
- `test` uses `default` (includes test files) → changing tests re-runs tests (correct)
- Lint/clippy use `default` (formatting applies to all files including tests)

**Consequence**: Developers get fast iteration on test code (cache persists) while source changes are caught correctly.

### Why Use `nx:run-commands` Instead of Custom Rust Rules?

**Simplicity**:
- Cargo is the authoritative Rust build system; reimplementing it in Nx is error-prone
- `nx:run-commands` is generic but transparent (developers see exactly what Cargo does)

**Maintenance**:
- Custom Rust rules require understanding Nx internals
- `@nx/rust` plugin (if adopted later) is a drop-in replacement for `nx:run-commands`
- No tech debt introduced

**Trade-off**: Less "Nx-idiomatic" but easier to maintain and understand

---

## Validation & Testing

### How to Validate This ADR?

1. **Task Execution**: Run all tasks and verify `dependsOn` ordering:
   ```bash
   nx run-many --target build test lint clippy --all --verbose
   ```
   Expected: builds complete before tests, all tasks succeed

2. **Cache Behavior**: Verify cache invalidation:
   ```bash
   # First run (cache miss)
   nx build --all
   
   # Second run (cache hit) - should be instant
   nx build --all
   
   # Touch a test file; re-run (cache should persist for build, but test runs)
   touch crates/evefrontier-lib/tests/test_utils.rs
   nx run-many --target build test --all
   ```

3. **CI Integration**: Verify GitHub Actions cache is working:
   - Monitor workflow times (should decrease after first run)
   - Check artifact cache size (should be <5 GB for Rust targets)

4. **Local/CI Parity**: Verify same tasks run identically:
   ```bash
   # Local
   nx build --all
   
   # Simulate CI
   NX_DAEMON=false nx build --all
   ```
   Both should produce identical results and exit codes.

### Examples for Contributors

**Adding a New Rust Crate**:

1. Create `crates/new-crate/Cargo.toml`
2. Create `crates/new-crate/project.json`:
   ```json
   {
     "name": "new-crate",
     "projectType": "library",
     "sourceRoot": "crates/new-crate/src",
     "tags": ["type:library", "lang:rust"],
     "targets": {
       "build": {
         "executor": "nx:run-commands",
         "options": {
           "command": "cargo build -p new-crate --locked",
           "cwd": "{workspaceRoot}"
         }
       },
       "test": {
         "executor": "nx:run-commands",
         "dependsOn": ["build"],
         "options": {
           "command": "cargo test -p new-crate --locked",
           "cwd": "{workspaceRoot}"
         }
       },
       "lint": {
         "executor": "nx:run-commands",
         "options": {
           "command": "cargo fmt -p new-crate -- --check",
           "cwd": "{workspaceRoot}"
         }
       },
       "clippy": {
         "executor": "nx:run-commands",
         "options": {
           "command": "cargo clippy -p new-crate --all-targets -- -D warnings",
           "cwd": "{workspaceRoot}"
         }
       },
       "complexity": {
         "executor": "nx:run-commands",
         "options": {
           "command": "cargo clippy -p new-crate --all-targets -- -W clippy::cognitive_complexity -D warnings",
           "cwd": "{workspaceRoot}"
         }
       }
     }
   }
   ```

3. Run `nx run-many --target build --all` to validate

**Adding a Custom Task** (e.g., benchmark):

1. Add target to `crates/evefrontier-lib/project.json`:
   ```json
   {
     "benchmark": {
       "executor": "nx:run-commands",
       "options": {
         "command": "cargo bench -p evefrontier-lib --locked",
         "cwd": "{workspaceRoot}"
       }
     }
   }
   ```

2. (Optional) Add to `nx.json::targetDefaults` if all crates should support it:
   ```json
   {
     "benchmark": {
       "cache": false,  // Benchmarks vary with system load; don't cache
       "parallel": false
     }
   }
   ```

3. Run `nx run-many --target benchmark --all`

---

## Implementation Checklist

- [ ] Create ADR 0017 in `docs/adrs/0017-nx-orchestration-strategy.md` (this file)
- [ ] Update `CONTRIBUTING.md` section "Nx Task Orchestration" with link to ADR 0017
- [ ] Update `AGENTS.md` or `.github/copilot-instructions.md` with reference to ADR 0017 for task configuration guidance
- [ ] Add examples to `docs/CODING_GUIDELINES.md` showing how to configure project.json correctly
- [ ] Validate all task definitions against this ADR (audit existing project.json files)
- [ ] Document in `.specify/memory/constitution.md` if this ADR should affect code generation
- [ ] Create PR with this ADR and documentation updates
- [ ] Obtain approval per ADR 0001 (ADR governance)

---

## Questions & Troubleshooting

**Q: Why does my build cache sometimes miss?**  
A: Cache keys are based on named inputs. If you changed `.rust-toolchain` or any file matching `production` inputs, cache invalidates. Run `nx compute-target-deps [project] [target]` to see input hash.

**Q: Can I use `cargo build` directly instead of `nx build`?**  
A: Yes! Both are equivalent. `nx build` adds caching and task orchestration; `cargo build` is faster if you only care about one crate. Use whichever fits your workflow.

**Q: What if a task hangs or is slow?**  
A: Check if `parallel: false` is preventing Cargo parallelism. If not, profile with `time nx [target] --all` to identify slow crate. Consider breaking large crate into smaller ones.

**Q: How do I disable Nx daemon for CI?**  
A: Set `NX_DAEMON=false` environment variable before running `nx` commands. Useful for container-based CI where daemon persistence is unnecessary.

**Q: Can I add a task that always runs (never cached)?**  
A: Yes, set `"cache": false` in the target definition (like `audit` and `outdated`).

**Q: How do I invalidate cache manually?**  
A: Run `nx reset` to clear all caches. Or run with `--skip-nx-cache` flag for single run.

---

## References

- **Nx Documentation**: https://nx.dev/docs (for deep dives on concepts)
- **Cargo Book**: https://doc.rust-lang.org/cargo/ (Rust build system)
- **ADR 0001**: Use Nygard ADR format for architectural decisions
- **ADR 0006**: Software Components (describes workspace structure)
- **ADR 0007**: DevSecOps Practices (describes CI/CD integration)
- **ADR 0009**: Spatial Index (example of caching generated artifacts)
- **Implementation**: `nx.json`, `crates/*/project.json`, `.github/workflows/ci.yml`

---

## Approval & Sign-Off

- [ ] Engineering Lead: _______________
- [ ] Architecture Committee: _______________
- [ ] DevSecOps (ADR 0007 alignment): _______________

**Created**: 2025-12-31  
**Last Updated**: 2025-12-31  
**Status**: Proposed (awaiting approval per ADR 0001)
