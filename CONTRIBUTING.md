# Contributing to EVE Frontier Rust

Thanks for your interest in contributing! This document explains the preferred workflow for filing
issues, submitting PRs, and running tests locally.

## Code of Conduct

This project follows the Contributor Covenant Code of Conduct. Please read `CODE_OF_CONDUCT.md` and
behave respectfully.

## How to file issues

- Search existing issues before opening a new one.
- For bug reports, use the Bug Report template and include: steps to reproduce, expected vs actual
  behavior, and environment details.
- For feature requests, use the Feature Request template and explain the motivation and high-level
  approach.

## Branching & PRs

- Branch from `main` using a short, descriptive name: `feature/<short>` or `fix/<short>`.
- Open a PR against `main` with the provided Pull Request template.
- Include unit tests for new behavior where practical.

## Maintaining CHANGELOG.md

This project maintains a CHANGELOG following [ADR 0010](docs/adrs/0010-maintain-changelog.md). All code changes must include a corresponding CHANGELOG.md entry.

### When CHANGELOG.md Update is Required

**✅ These changes REQUIRE a CHANGELOG.md entry:**
- Modifications to source code (`src/`, `crates/`, `examples/`, etc.)
- Changes to build system (`Cargo.toml`, `Makefile`)
- Test or benchmark code changes (`tests/`, `benches/`)

**⏭️ These changes are EXEMPT from CHANGELOG requirement:**
- Pure documentation updates (`docs/`, root `*.md` files)
- CI configuration (`.github/workflows/`)
- Repository configuration (`.gitignore`, `.nvmrc`, etc.)

### CHANGELOG Entry Format

Add entries to the `Unreleased` section at the top of `CHANGELOG.md` using this format:

```markdown
- YYYY-MM-DD - Author Name - [category] - Brief description of the change
```

**Valid categories:** `[feature]`, `[fix]`, `[docs]`, `[refactor]`, `[security]`, `[perf]`, `[ci]`, `[build]`, `[deps]`, `[lint]`

**Examples:**
```markdown
- 2025-12-07 - Jane Doe - [feature] - Added CI guard for CHANGELOG.md enforcement
- 2025-12-07 - auto-llm:copilot - [fix] - Fixed edge case in spatial routing with temperature filtering
- 2025-12-07 - John Smith - [perf] - Optimized KD-tree query performance by 15%
```

### CI Enforcement

**The CI workflow automatically validates CHANGELOG.md updates.** If your PR:
- ✅ Modifies code + updates CHANGELOG.md → CI passes
- ❌ Modifies code but skips CHANGELOG.md → CI fails with guidance
- ✅ Only changes docs/config → CI passes without CHANGELOG requirement
- ✅ Has `skip-changelog-check` label → CI passes (emergency only)

### Emergency Override

For time-sensitive fixes, add the `skip-changelog-check` label to your PR. This label should only be used with maintainer approval for genuine emergencies—changelog updates are expected to be addressed in a follow-up PR.

## Architecture Decision Records (ADRs)

This project uses Architecture Decision Records (ADRs) to document architecturally significant
decisions. ADRs are stored in `docs/adrs/` and follow the Nygard/Fowler format per
[ADR 0001](docs/adrs/0001-use-nygard-adr.md) and the
[project Constitution](. specify/memory/constitution.md).

### When to Create an ADR

Create an ADR when making decisions that affect:
- Project architecture or structure (workspace layout, module boundaries)
- Core algorithms or data structures (routing algorithms, spatial indexing)
- Technology choices (libraries, frameworks, build tools)
- Security or compliance requirements
- Performance characteristics or constraints
- API contracts or data schemas

**Do NOT create ADRs for:**
- Bug fixes (unless they require an architectural change)
- UI/UX tweaks or minor refactorings
- Documentation updates
- Dependency version bumps

### ADR Naming Convention

ADRs must follow this pattern: `docs/adrs/NNNN-slug-title.md`

**Rules:**
- **NNNN**: Four-digit sequence number (e.g., `0001`, `0042`, `0123`)
- **slug-title**: Lowercase words separated by hyphens (kebab-case)
- **File extension**: `.md`

**Valid examples:**
- ✅ `docs/adrs/0001-use-nygard-adr.md`
- ✅ `docs/adrs/0009-kd-tree-spatial-index.md`
- ✅ `docs/adrs/0013-containerization-strategy.md`

**Invalid examples:**
- ❌ `docs/adrs/adr-001-use-nygard.md` (number must come first)
- ❌ `docs/adrs/1-use-nygard-adr.md` (number must be 4 digits)
- ❌ `docs/adrs/0001-Use-Nygard-ADR.md` (slug must be lowercase)
- ❌ `docs/adrs/0001_use_nygard_adr.md` (use hyphens, not underscores)

### ADR Immutability Policy

**ADRs are immutable after ratification.** This ensures historical context is preserved and prevents
revisionist changes.

#### Creating a New ADR

1. Determine the next sequence number by checking existing ADRs
2. Create a file in `docs/adrs/` with the proper naming pattern
3. Use the ADR template from `docs/adrs/TEMPLATE.md` or copy an existing ADR structure
4. Fill in all sections: Status, Context, Decision, Rationale, Consequences, References
5. Submit as part of your PR
6. The CI workflow will validate the filename pattern automatically

#### Editing an Existing ADR

**Substantive changes are not allowed.** Instead:

1. **Create a new ADR** that supersedes or amends the original
2. Set the Status in the new ADR to: `Supersedes ADR XXXX` or `Amends ADR XXXX`
3. Explain in the Context section why the original decision is being revised
4. Document the new approach in the Decision section
5. Cross-reference the original ADR in the References section

**Example:**
```markdown
# ADR 0013: Use PostgreSQL Instead of SQLite

## Status

Supersedes ADR 0004

## Context

ADR 0004 adopted SQLite for dataset storage. Since then, we've discovered that...
```

#### Fixing Typos or Errors

For **minor corrections only** (typos, formatting, broken links):

1. Make your changes to the existing ADR file
2. Request the `allow-adr-edits` label on your PR (ask a maintainer)
3. Explain in the PR description what is being corrected and why it's not substantive

The ADR governance CI workflow will:
- ✅ **Allow** edits to existing ADRs if the `allow-adr-edits` label is present
- ❌ **Block** edits to existing ADRs without the label
- ✅ **Allow** all new ADRs (validated for filename pattern only)

**Use the label sparingly** — it's meant for obvious errors, not policy changes.

#### Enforcement

The `.github/workflows/adr-governance.yml` workflow automatically:
- Validates ADR filenames match the required pattern
- Detects edits to existing ADRs and checks for the override label
- Provides detailed error messages with guidance

If the workflow fails on your PR:
- **Pattern violation**: Fix the filename to match `NNNN-slug-title.md`
- **Immutability violation**: Either create a new ADR or request the `allow-adr-edits` label with justification

### ADR Best Practices

- **Be specific**: Document the exact decision, not just general principles
- **Include context**: Explain the problem being solved and constraints
- **Show alternatives**: Document options considered and why they were rejected
- **Document consequences**: Both positive and negative outcomes
- **Add references**: Link to related ADRs, issues, documentation, or external resources
- **Use present tense**: Write as if the decision is being made now ("We will use..." not "We used...")
- **Keep it concise**: 1-3 pages maximum; link to detailed specs elsewhere if needed

### ADR Template

See `docs/adrs/TEMPLATE.md` for the standard structure. Key sections:

```markdown
# ADR NNNN: Title

## Status
<!-- Proposed | Accepted | Deprecated | Superseded by ADR XXXX -->

## Context
<!-- What problem are we solving? What constraints exist? -->

## Decision
<!-- What are we doing? Be specific and actionable. -->

## Rationale
<!-- Why this approach? What alternatives were considered? -->

## Consequences
<!-- What does this decision enable? What trade-offs do we accept? -->

## References
<!-- Links to related ADRs, issues, RFCs, external docs -->
```

For more details, see:
- [ADR 0001: Use Nygard-style ADRs](docs/adrs/0001-use-nygard-adr.md)
- [Constitution Principle III](.specify/memory/constitution.md#iii-architecture-decision-records-mandatory)


## Tooling requirements

- Node.js: use the version pinned in `.nvmrc` (currently 24 LTS as of November 2025). Install via `nvm use`.
- Package manager: pnpm 10+. The repository declares `"packageManager": "pnpm@10.0.0"` and
  enforces `engines.pnpm >= 10.0.0`.

Quick setup:

```sh
# Ensure Node 24 per .nvmrc
nvm use

# Install pnpm v10 globally (recommended)
npm install -g pnpm@10

# Install workspace tools and generate lockfile
pnpm install
```

Common developer commands (recommended via package.json scripts for consistency):

```sh
pnpm run build
pnpm run test
pnpm run clippy
pnpm run lint:md
```

Or, run Nx directly with the required exclusion flag to avoid recursive invocation of the root project:

```sh
pnpm nx run-many -t build --exclude evefrontier-rs
pnpm nx run-many -t test --exclude evefrontier-rs
pnpm nx run-many -t clippy --exclude evefrontier-rs
```

## Nx Task Orchestration

The repository uses Nx to orchestrate Rust build, test, lint, and clippy tasks across all 6 crates in the workspace:

- evefrontier-lib
- evefrontier-cli
- evefrontier-lambda-shared
- evefrontier-lambda-route
- evefrontier-lambda-scout-gates
- evefrontier-lambda-scout-range

> [!NOTE]
> When referring to "all 6 crates", this means all workspace members listed above. "All Lambda
> crates" refers to the 4 crates under `crates/evefrontier-lambda-*` (including the shared crate),
> while "Lambda functions" refers specifically to the 3 function crates (`route`, `scout-gates`,
> `scout-range`). Be explicit in PRs and documentation to avoid confusion.

Nx provides:
- **Task dependencies**: Tests automatically run after builds complete (`test` depends on `build`)
- **Intelligent caching**: Nx caches task outputs locally to skip redundant work
- **Affected detection**: Run tasks only for projects impacted by your changes

### Task Execution Model

Nx tasks are configured with `parallel: false` for all Rust targets. This means:
- Nx orchestrates task order and dependencies (e.g., ensuring builds complete before tests)
- Cargo manages its own internal parallelism for compilation (`-j` flag)
- No conflicts between Nx and Cargo parallelism

### Common Nx Commands

Run a single project's tests (with automatic build):
```sh
pnpm nx run evefrontier-lib:test
```

Run tests for all projects:
```sh
pnpm nx run-many --target=test --all
```

Run tests only for projects affected by your changes:
```sh
pnpm nx affected --target=test
```

Build specific projects with explicit dependency order:
```sh
pnpm nx run-many --target=build --projects=evefrontier-lib,evefrontier-cli
```

Run clippy on all crates:
```sh
pnpm nx run-many --target=clippy --all
```

### Cache Behavior

- Nx caches build and test outputs in `.nx/cache/` (local only, not committed)
- Cache keys include input files (Cargo.toml, Cargo.lock, src/**), toolchain version, and command
- To clear cache: `pnpm nx reset`
- To bypass cache for a single run: `pnpm nx run <project>:<target> --skip-nx-cache`

### Troubleshooting

If Nx daemon causes issues (rare):
```sh
NX_DAEMON=false pnpm nx run-many --target=test --all
```

To see detailed task execution:
```sh
pnpm nx run <project>:<target> --verbose
```

## Scripts Project

The `scripts/` directory is registered as an Nx project (`scripts`) and contains utility scripts
for fixture management, database inspection, and development tooling.

### Available Script Tasks

| Task | Description |
|------|-------------|
| `scripts:venv-setup` | Set up Python virtual environment |
| `scripts:fixture-verify` | Verify fixture integrity against recorded metadata |
| `scripts:fixture-status` | Display current fixture status and statistics |
| `scripts:fixture-record` | Record current fixture metadata after updates |
| `scripts:inspect-db <path>` | Inspect SQLite database schema and contents |
| `scripts:verify-all` | Run all verification tasks |

### Running Script Tasks

```sh
# Verify test fixtures are intact
pnpm nx run scripts:fixture-verify

# Inspect a database file
pnpm nx run scripts:inspect-db docs/fixtures/minimal_static_data.db

# Run all verification tasks
pnpm nx run scripts:verify-all
```

### Python Virtual Environment

Python scripts use a local virtual environment in `scripts/.venv/`:

```sh
# Set up the virtual environment (first-time or after requirements change)
pnpm nx run scripts:venv-setup
```

Dependencies are defined in `scripts/requirements.txt`. Currently, all scripts use Python stdlib
only, so no external packages are required.

For detailed documentation of all scripts, see `scripts/README.md`.

## Local development

1. Install Rust (see `.rust-toolchain`) and Node (see `.nvmrc` if using Node tools).
2. Build the workspace:

```pwsh
cargo build --workspace
```

3. Run tests:

```pwsh
cargo test --workspace
```

4. Use the included `minimal_static_data.db` fixture for deterministic tests. See `docs/USAGE.md`
   for an example of wiring `EVEFRONTIER_DATA_DIR`.

## Formatting & linting

- Run `cargo fmt` and `cargo clippy` before opening a PR.
- Run `pnpm run lint:md` to lint markdown files.

## Security

If you discover a security vulnerability, please report it using GitHub's private security advisory system.

## Contact

# Pre-start checklist for contributors and agents

Before starting work on a change, read the repository guidance and check for relevant documentation:

- Read `.github/copilot-instructions.md` for workspace-wide conventions and developer guidance.
- Review `docs/` and `docs/adrs/` for ADRs and design decisions that could affect your change.
- If your change touches release packaging, CI, or security, consult
  `docs/adrs/0007-devsecops-practices.md`.

Following this checklist helps avoid duplicated effort and ensures changes align with established
decisions.

# Contributing

Thank you for contributing! This document explains how to set up a local development environment,
run the standard checks, and contribute changes to the repository.

Environment

1. Install Rust (recommended: rustup)

- Install `rustup` from the official site: <https://rustup.rs>

- The repository uses Rust 1.91.1 (pinned in `.rust-toolchain`). When you enter the repository
  directory, rustup will automatically use this version. To verify:

  rustc --version # should show 1.91.1

2. Install Node.js and pnpm (for developer tooling)

- Use a tool like `volta` or `nvm` to manage Node versions.
- The repository pins Node 24 (LTS) in `.nvmrc`. If using nvm:

  nvm use # automatically uses version from .nvmrc

> [!NOTE]
> If your environment policy mandates current LTS prior to October 2025, use Node 22 LTS and
> update `.nvmrc` accordingly. The workspace scripts and Nx tooling are compatible with Node 22+,
> but the repository standard is Node 24 LTS to align with modern features and CI configuration.

- Install `pnpm` (recommended using corepack for version consistency):

  corepack enable
  corepack prepare pnpm@10.0.0 --activate

  Alternatively, install globally with version pin:

  npm install -g pnpm@10

3. Optional developer tools

- Install `cargo-audit` for dependency scanning:

  cargo install cargo-audit

4. VS Code MCP Configuration (Optional)

The repository includes `.vscode/mcp.json` which configures the GitHub Copilot Model Context
Protocol (MCP) server. This enables enhanced GitHub integration features within VS Code when using
GitHub Copilot.

**This configuration is optional.** The MCP setup is not required to build, test, or contribute to
the project. It provides additional AI-assisted features for developers using GitHub Copilot in VS
Code, such as:

- Enhanced repository context awareness
- GitHub API integration for PR/issue management
- Improved code suggestions based on repository patterns

If you're not using GitHub Copilot or prefer not to use MCP features, you can safely ignore this
configuration. The repository will work normally without it.

Local checks

- Build the Rust workspace:

  cargo build --workspace

- Run tests:

  cargo test --workspace

- Run Rust lints:

  cargo clippy --all-targets --all-features -- -D warnings

- Run code complexity analysis (checks against configured thresholds in `clippy.toml`):

  pnpm nx run-many -t complexity --exclude evefrontier-rs

  Or using cargo directly:

  cargo clippy --workspace --all-targets -- \
    -W clippy::cognitive_complexity \
    -W clippy::too_many_lines \
    -W clippy::excessive_nesting \
    -D warnings

- Format and lint markdown:

  pnpm install pnpm run lint:md

- Format code and docs:

  pnpm run format

Pre-commit and CI

- The project uses `husky` and `lint-staged` for local pre-commit hooks. These run quick formatters
  on staged files. Keep hooks fast — heavy checks run in CI.
- CI should run the full matrix: `cargo test`, `clippy`, `cargo audit` (or nightly),
  `pnpm run lint:md`, and any additional integration tests.

Dependency Management

- **Nightly Dependency Checks**: A GitHub Actions workflow runs nightly to check for outdated Rust
  and Node dependencies. The workflow publishes artifacts containing reports you can review:
  - `rust-outdated-report`: JSON and text reports from `cargo outdated`
  - `node-outdated-report`: JSON and text reports from `pnpm outdated`
  - Find these under the "Dependency Check" workflow runs in the Actions tab
  - Reports are kept for 30 days
- **Manual Dependency Updates**: To update dependencies locally:
  - Rust: `cargo update` (patch versions) or edit `Cargo.toml` for minor/major updates
  - Node: `pnpm update` or edit `package.json`
  - Always run tests after updating dependencies
  - Update `CHANGELOG.md` when making dependency changes

Signing and releases

- For release artifacts, prefer signing binaries or release archives using a dedicated signing
  process (GPG or `cosign`), and publish attestations for the important pipeline steps
  (build/test/scan).

Notes

- If you need deterministic runs for tests that use the dataset downloader, call
  `ensure_dataset(Some(path), DatasetRelease::latest())` (or choose a specific
  `DatasetRelease::tag(...)`) to provide a fixed dataset path.
- If you don't want to install Node locally, you can still build and test the Rust crates; however
  markdown linting and other Node-based tooling won't run until Node/pnpm is available.
- ensure that your commit message is less than 72 chars in the first line, add more details in the
  summary from line 3 onwards.
- use conventional commit standards to create clear commits

Thanks for helping improve the project. If you add new developer-facing tools, please update
`CONTRIBUTING.md` and `docs/adrs/0006-software-components.md`.
