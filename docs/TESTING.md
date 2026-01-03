# Testing Guide

This document describes the testing strategy for the EVE Frontier project, which provides a unified
testing framework used both locally and in CI.

## Overview

The project uses a **three-tier testing strategy**:

1. **Unit & Integration Tests** - Comprehensive Rust tests using `cargo test`
2. **Smoke Tests** - Quick end-to-end validation using the CLI
3. **CI Checks** - Full validation including formatting, linting, build, and tests

## Quick Reference

```bash
# Run all unit and integration tests
make test

# Run quick smoke tests with the CLI
make test-smoke

# Run everything (tests + smoke tests)
make test-all

# Run full CI checks locally (same as CI workflow)
make ci

# Format code
make fmt

# Run clippy lints
make lint
```

## Test Tiers Explained

### 1. Unit & Integration Tests (`make test`)

Located in `crates/*/tests/` directories, these tests provide comprehensive coverage:

- **Dataset tests** (`dataset_*.rs`) - Download, caching, fixture protection
- **Graph tests** (`graph.rs`) - Graph construction for routing
- **Routing tests** (`routing.rs`) - Pathfinding algorithms (BFS, Dijkstra, A\*)
- **CLI tests** (`route_commands.rs`) - Command-line interface behavior
- **Output tests** (`output.rs`) - Serialization formats (JSON, text, note)
- **Fuzzy matching tests** (`fuzzy_matching.rs`) - System name suggestions

**Test Fixture**: Uses real e6c3 data with 8 systems:

- Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G
- 12 jump gates, 26 planets, 43 moons
- Located at: `docs/fixtures/minimal/static_data.db`

**Run with:**

```bash
cargo test --workspace
# or
make test
```

### 2. Smoke Tests (`make test-smoke`)

Quick end-to-end tests using the **release** binary to validate:

1. Download command functionality
2. Basic route planning (Nod → Brana)
3. JSON output structure validation

**Run with:**

```bash
make test-smoke
```

> [!NOTE]
> Requires `jq` for JSON validation.

### 3. CI Checks (`make ci`)

Runs the **same checks as the CI workflow** locally:

1. ✅ **Format check** - `cargo fmt --all -- --check`
2. ✅ **Clippy** - `cargo clippy --workspace --all-targets -D warnings`
3. ✅ **Build** - `cargo build --workspace --all-targets`
4. ✅ **Tests** - `cargo test --workspace`

**Run with:**

```bash
make ci
```

> [!TIP]
> This is the **recommended command before pushing** to catch issues early.

## Pre-commit Hooks

The project uses [rusty-hook](https://github.com/swellaby/rusty-hook) to run CI checks automatically
before each commit:

- Auto-installs on first `cargo build`
- Runs the same checks as `make ci`
- Blocks commits if any check fails
- Provides fast feedback (typically 5-10 seconds with warm cache)

> [!CAUTION]
> To skip pre-commit hooks (not recommended):
>
> ```bash
> git commit --no-verify
> ```

## CI Workflow

The GitHub Actions CI workflow (`.github/workflows/ci.yml`) runs two jobs:

### Job 1: Build and Test

- Builds the workspace
- Runs all tests

### Job 2: Validate Documentation Examples

- Builds release binary
- Runs CLI examples from README/USAGE docs
- Validates JSON output structure
- Uses the same real e6c3 fixture systems (Nod, Brana, etc.)

## Testing Best Practices

### When Writing Tests

1. **Use the fixture** - Don't create new test data; use `docs/fixtures/minimal/static_data.db`
2. **Use real system names** - Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G
3. **Isolated environments** - Tests use temporary directories via `tempfile` crate
4. **Clear assertions** - Use descriptive predicates from `assert_cmd` and `predicates` crates

### Before Committing

```bash
# Run full CI checks locally
make ci

# If you added new features, run smoke tests
make test-smoke
```

The pre-commit hook will catch most issues, but running `make ci` gives you detailed output.

### Benchmarks

Pathfinding performance can be measured with Criterion benchmarks located under
`crates/evefrontier-lib/benches/`.

```bash
# Run all benchmarks (compiles release artifacts)
make bench

# Or directly via Cargo
cargo bench -p evefrontier-lib
```

Benchmarks run against the same pinned fixture (Nod ↔ Brana) and cover BFS, Dijkstra, and A\*
(hybrid and spatial) planners. Use them when tuning graph/pathfinding code to catch regressions
early.

### Before Pushing

```bash
# Run everything
make test-all
```

This ensures both automated tests and manual CLI validation pass.

## Debugging Test Failures

### Integration Test Failures

```bash
# Run specific test file
cargo test --test routing

# Run with output
cargo test --test routing -- --nocapture

# Run single test
cargo test --test routing dijkstra_route_plan_succeeds
```

### Smoke Test Failures

Smoke tests use environment variables to control behavior:

```bash
# Run manually with debug output
EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal/static_data.db \
RUST_LOG=debug \
./target/release/evefrontier-cli --data-dir /tmp/test route --from "Nod" --to "Brana"
```

### CI Failures

If CI fails but local tests pass:

1. Check Rust version matches CI (see `.github/workflows/ci.yml`)
2. Run `cargo test --locked` to use exact dependency versions
3. Ensure fixture is committed: `git status docs/fixtures/`

## Continuous Integration Details

### Caching Strategy

CI caches:

- `~/.cargo/registry` - Downloaded crates
- `~/.cargo/git` - Git dependencies
- `target/` - Build artifacts

Cache key: `cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}`

### Dependency Pinning

- Rust toolchain: **1.90.0** (via `.rust-toolchain` or workflow)
- Dependencies: Locked via `Cargo.lock` (committed to repo)

## Troubleshooting

### "Unknown system 'X'" in smoke tests

The smoke test fixture uses **real e6c3 system names**. Old examples may reference synthetic systems
like "Y:170N" or "BetaTest". Update to use real names:

```bash
# ❌ Old (won't work)
--from "Y:170N" --to "BetaTest"

# ✅ New (real e6c3 systems)
--from "Nod" --to "Brana"
```

### Pre-commit hook not running

```bash
# Reinstall hooks
cargo clean
cargo build
```

Hooks are installed via `build.rs` in the workspace root.

### jq not found (smoke tests)

```bash
# Ubuntu/Debian
sudo apt-get install jq

# macOS
brew install jq

# Arch
sudo pacman -S jq
```

## Future Enhancements

See `docs/TODO.md` for planned testing improvements:

- [ ] Benchmark harness for pathfinding performance
- [ ] Property-based testing for graph algorithms
- [ ] Integration with Nx for orchestrated testing
- [ ] Automated fixture updates on new e6c3 releases
