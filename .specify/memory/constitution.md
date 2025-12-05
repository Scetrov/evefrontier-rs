<!-- 
SYNC IMPACT REPORT: Constitution v1.0.0
===========================================
- Version: 1.0.0 (initial ratification)
- New principles: 7 (TDD-First, Library-First, ADR Documentation, Clean Code, Security-First, Testing Tiers, Refactoring & Technical Debt)
- Governance structure: Amendments via RFC-style ADR, version semver, ratification required
- Templates requiring updates: 
  ✅ plan-template.md (ensure TDD & ADR checks)
  ✅ spec-template.md (require test scenarios)
  ✅ tasks-template.md (add TDD, ADR, security task types)
  ✅ commands/speckit.constitution.prompt.md (updated context)
- No intentionally deferred placeholders
-->

# EveFrontier Rust Workspace Constitution

## Core Principles

### I. Test-Driven Development (NON-NEGOTIABLE)
All production code MUST follow the Red-Green-Refactor cycle strictly:
1. Write tests first that express the desired behavior (Red)
2. Implement minimal code to pass tests (Green)
3. Refactor to improve clarity without changing behavior (Refactor)

Exceptions are explicitly forbidden. Unit tests MUST cover happy paths, edge cases, and error conditions. Integration tests MUST validate cross-crate contracts. Minimum threshold: 70% code coverage for library crates; 80% for critical paths (pathfinding, graph construction).

**Rationale**: TDD ensures correctness, prevents regression, documents intended behavior, and creates a safety net for refactoring. It is non-negotiable for maintainability and production reliability.

### II. Library-First Architecture
Every feature MUST start as a reusable library crate in `crates/evefrontier-lib/` before being exposed via CLI or Lambda. Libraries MUST be:
- Self-contained and independently testable without external dependencies
- Documented with clear public API (Rustdoc comments for all public items)
- Schema-aware (support multiple dataset formats with runtime detection per ADR 0004)
- Free of console I/O; libraries work with data types only

CLI and Lambda crates are thin wrappers that call library functions; business logic MUST NOT live in CLI/Lambda main.rs or handler functions.

**Rationale**: Library-first ensures code reusability, testability, and clean separation of concerns. It enables Lambda, CLI, and future consumers to share identical business logic.

### III. Architecture Decision Records (Mandatory)
Every architecturally significant decision MUST be documented as a Nygard-style ADR in `docs/adrs/`. ADRs:
- MUST be immutable after ratification (record changes as new ADRs, never edit historical records)
- MUST follow the naming pattern `NNNN-slug-title.md` (e.g., `0001-use-nygard-adr.md`)
- MUST include Status, Context, Decision, Rationale, Consequences, and References sections
- MUST be approved in a PR before implementation begins (decision precedes code)

Large refactorings, schema changes, major algorithm shifts, or new constraints require an ADR. Small bug fixes and UI tweaks do not. When in doubt, document as an ADR.

**Rationale**: ADRs preserve tribal knowledge, enable future contributors to understand design rationale, and prevent recurring debates. Recording decisions as code enforces accountability and transparency.

### IV. Clean Code & Cognitive Load
Code MUST prioritize readability and maintainability:
- Variable and function names MUST be descriptive (avoid abbreviations except domain conventions like "BFS", "A*")
- Functions MUST have a single responsibility (McCabe complexity < 15)
- Nesting depth MUST not exceed 3 levels; extract helper functions if deeper
- Comments explain "why", not "what" (code should be self-documenting for "what")
- Avoid magic numbers; use named constants with clear intent

Rust idioms are preferred: use `Result<T, E>` for error handling, pattern matching over if-else chains, and iterator combinators over manual loops where idiomatic.

**Rationale**: Clean code reduces defects, accelerates onboarding, and lowers maintenance burden. Cognitive load is a finite resource; every unnecessary complexity degrades team velocity.

### V. Security-First Development
Security considerations MUST be addressed at design time, not added later:
- All external inputs (URLs, file paths, user data) MUST be validated against allow-lists
- Database queries MUST use parameterized statements (never string concatenation)
- Secrets MUST never be hardcoded; use environment variables or secret managers
- Error messages MUST not leak sensitive information (paths, internal IDs, stack traces in production)
- Dependencies MUST be audited regularly via `cargo audit` and updated promptly

Mandatory security review for any code handling external data, authentication, or cryptography. See `.github/instructions/security-and-owap.instructions.md` for comprehensive guidelines.

**Rationale**: Security vulnerabilities are costly to remediate in production. Proactive design prevents whole classes of attacks and complies with OWASP standards.

### VI. Testing Tiers (Aligned with CI)
Testing follows a three-tier strategy in both local development and CI:
1. **Unit & Integration Tests** (primary): Located in `crates/*/tests/` and `src/lib.rs` test modules. MUST run fast (<5s total). Cover happy paths, edge cases, error conditions, and schema variations.
2. **Smoke Tests** (secondary): End-to-end CLI validation (e.g., `make test-smoke`). Verify key workflows work in release mode. MUST complete in <30s.
3. **CI Checks** (tertiary): Full pipeline including formatting, clippy, build, and all tests. Gated behind branch protection.

All three tiers are required to pass before merging. Local developers run tiers 1 and 2; CI runs tier 3. Fixture dataset at `docs/fixtures/minimal_static_data.db` is used for reproducible testing.

**Rationale**: Three tiers balance speed (catch bugs early locally) with confidence (CI validates production readiness). Smoke tests catch integration issues that unit tests may miss.

### VII. Refactoring & Technical Debt Management
Refactoring MUST be performed regularly to prevent technical debt accumulation:
- Refactoring PRs MUST NOT introduce behavior changes (green tests before and after)
- Large refactorings MUST be split into multiple PRs, each focused on one concern
- Complexity is justified in ADRs; avoid "just because" rewrites without business value
- Legacy code MUST be refactored when touched (leave the code cleaner than you found it)

Technical debt is tracked in `docs/TODO.md` with priority and effort estimates. At the start of each quarter, dedicate at least 20% of capacity to debt reduction.

**Rationale**: Regular refactoring prevents code rot and keeps velocity high. Small, focused refactorings are easier to review and less risky than large rewrites.

## Rust Best Practices & Standards

### Toolchain & Reproducibility
- Rust version MUST match `.rust-toolchain` (currently 1.91.1)
- All crates MUST build cleanly with `cargo build --workspace` and `cargo build --release`
- All crates MUST pass `cargo fmt --all -- --check` (automatic formatting)
- All crates MUST pass `cargo clippy --workspace --all-targets -D warnings` (strict lints enforced as errors)
- Dependencies MUST be kept current; run `cargo outdated` monthly and update PATCH versions immediately, MINOR versions quarterly

### Error Handling
- MUST use `Result<T, E>` for fallible operations; never panic in library code (except in tests or unrecoverable states)
- Custom `Error` type MUST implement `thiserror` derive for structured error handling
- Error messages MUST be actionable and include context (file paths, system names, constraint values)
- Transparent error wrapping MUST preserve root cause (use `.context()` from `anyhow` or explicit From impls)

### Documentation
- All public types, functions, and modules MUST have Rustdoc comments with examples
- Library crate MUST maintain comprehensive usage guide in `docs/USAGE.md` with code examples
- Complex algorithms MUST include algorithm name and reference (e.g., "BFS per Cormen et al.")
- Schema-specific code MUST document supported formats and detection logic (see `db.rs` for examples)

### Performance & Profiling
- Performance-critical paths MUST have benchmarks in `crates/evefrontier-lib/benches/` using criterion
- Spatial indexing (KD-tree per ADR 0009) MUST be used for range queries >100 systems
- Memory usage MUST be documented for Lambda cold-start context (ephemeral storage limited to 512 MB)
- Profile before optimizing; document optimization rationale in comments and ADRs

## Development Workflow & Review Process

### Branching & PR Workflow
- Feature branches MUST use naming: `feature/<short-description>` or `fix/<short-description>` (e.g., `feature/fuzzy-system-match`)
- All PRs MUST include a descriptive title and link to related issues
- PRs MUST pass all CI checks (format, clippy, tests, security audit) before review is requested
- PRs MUST include updated `CHANGELOG.md` entry under Unreleased with date, author, and [manual] or [auto-llm] tag

### Code Review Requirements
- All PRs MUST be reviewed by at least one maintainer before merging
- Review checklist:
  - ✅ Tests pass locally and in CI
  - ✅ TDD cycle followed (tests → implementation → refactor)
  - ✅ No debug code or `println!` macros left (use `tracing::*` for logging)
  - ✅ ADR created if architecturally significant
  - ✅ Security review if handling external data or secrets
  - ✅ Changelog entry accurate and clear
  - ✅ No unnecessary complexity introduced

### Commit Hygiene
- Commits MUST be signed with GPG (`git config commit.gpgsign true`)
- Commit messages MUST follow conventional commits: `<type>(<scope>): <description>` (e.g., `feat(routing): implement A* pathfinding`)
- Force push to protected branches (`main`, `release/*`) is forbidden; use rebase or squash workflows only
- Atomic commits preferred: each commit should compile and pass tests independently

## Versioning & Release Policy

### Semantic Versioning
All crates follow semantic versioning (MAJOR.MINOR.PATCH):
- **MAJOR**: Incompatible API changes, schema changes, breaking behavior changes
- **MINOR**: New features, optimizations, additions that are backward compatible
- **PATCH**: Bug fixes, documentation updates, performance improvements

CHANGELOG.md MUST be updated before release with clear categorization (Breaking, Features, Fixes).

### Release Approval
- Releases MUST be tagged with GPG signature: `git tag -s vX.Y.Z`
- Release artifacts MUST be signed with cosign/GPG and include SBOMs (Software Bill of Materials)
- Release notes MUST be published on GitHub with migration guidance for breaking changes
- Releases MUST be tested against the current dataset and Lambda runtime

## Governance

### Constitution Supersedes All Other Guidance
This Constitution is the source of truth for development practices. All other documents (CONTRIBUTING.md, TESTING.md, ADRs) are subordinate and MUST align with these principles. If conflicts arise, the Constitution takes precedence.

### Amendment Process (RFC-Style ADRs)
Changes to the Constitution require:
1. A new RFC-style ADR in `docs/adrs/` proposing the change
2. Community discussion and consensus (approved in PR review)
3. Update to this Constitution document with new version
4. Ratification by project maintainer(s)

Version MUST increment using semantic versioning:
- **Major**: Principle removed or fundamentally redefined (backward incompatible governance)
- **Minor**: New principle added or existing principle significantly expanded
- **Patch**: Clarifications, wording refinements, typo fixes

### Compliance Validation
Every PR MUST be validated against this Constitution:
- Code reviews MUST verify TDD, Library-First, and Security-First principles
- Linting and formatting MUST pass automatically (non-negotiable quality gates)
- New ADRs MUST follow Nygard format and be approved before code changes
- Technical debt tracking MUST be updated in `docs/TODO.md` when new debt is incurred

### Escalation & Exceptions
Exceptions to this Constitution require explicit maintainer approval and MUST be documented with a comment in the relevant code/PR. Exceptions MUST be rare and justified with business context. Repeated exceptions indicate the Constitution should be amended.

---

**Version**: 1.0.0 | **Ratified**: 2025-12-05 | **Last Amended**: 2025-12-05

For runtime development guidance, see `.github/copilot-instructions.md` and `CONTRIBUTING.md`.
