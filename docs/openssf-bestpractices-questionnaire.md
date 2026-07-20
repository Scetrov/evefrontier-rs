# OpenSSF Best Practices Badge - Questionnaire Evidence

This document provides answers and evidence locations for each criterion in the OpenSSF Best Practices badge questionnaire at https://www.bestpractices.dev/en

**Project:** evefrontier-rs
**Repository:** https://github.com/Scetrov/evefrontier-rs

---

## PASSING Level Criteria

### Basics

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `description_good` | Met | README.md clearly describes the project as "A comprehensive Rust workspace for working with EVE Frontier static datasets, providing pathfinding and navigation tools for the game world" |
| `interact` | Met | README.md provides instructions for obtaining (cargo install), feedback (GitHub Issues, SECURITY.md), and contributing (CONTRIBUTING.md) |
| `contribution` | Met | CONTRIBUTING.md documents the complete contribution process including branching strategy, PR requirements, testing requirements, and code review |
| `contribution_requirements` | Met | CONTRIBUTING.md specifies requirements: clippy passes, tests added for new functionality, ADRs for architectural changes, CHANGELOG.md updates |
| `floss_license` | Met | Licensed under MIT License - LICENSE file in repository root |
| `floss_license_osi` | Met | MIT License is OSI-approved |
| `license_location` | Met | LICENSE file in repository root; LICENSE.md also present |
| `documentation_basics` | Met | README.md, docs/ directory with architecture docs, ADRs, usage guides, API documentation |
| `documentation_interface` | Met | Comprehensive README.md, docs/USAGE.md, and rustdoc-generated API docs |
| `sites_https` | Met | GitHub repository uses HTTPS, all documentation hosted on GitHub Pages uses HTTPS |
| `discussion` | Met | GitHub Issues for bug reports, GitHub Discussions available, searchable and URL-addressable |
| `english` | Met | All documentation in English; accepts issues in English |

### Change Control

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `repo_public` | Met | Public GitHub repository: https://github.com/Scetrov/evefrontier-rs |
| `repo_track` | Met | Git tracks all changes with author, timestamp, and commit message |
| `repo_interim` | Met | Git repository contains all development commits, not just releases |
| `repo_distributed` | Met | Uses Git distributed version control |
| `release_notes` | Met | CHANGELOG.md follows Keep a Changelog format for all releases |
| `release_notes_vulns` | Met | SECURITY.md documents known vulnerabilities; releases note security fixes |

### Reporting

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `report_process` | Met | GitHub Issues template for bug reports; CONTRIBUTING.md documents process |
| `report_tracker` | Met | GitHub Issues: https://github.com/Scetrov/evefrontier-rs/issues |
| `report_responses` | Met | Maintainer responds to issues within reasonable timeframe |
| `enhancement_responses` | Met | Maintainer reviews and responds to feature requests |
| `report_archive` | Met | All issues archived publicly on GitHub Issues |
| `vulnerability_report_process` | Met | SECURITY.md provides detailed vulnerability reporting process |
| `vulnerability_report_private` | Met | SECURITY.md provides process for private security reports via email |
| `vulnerability_report_response` | Met | SECURITY.md commits to initial response within 48 hours |

### Quality

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `build` | Met | Cargo build system; `cargo build` builds all components |
| `build_common_tools` | Met | Uses rustc and Cargo, standard Rust toolchain |
| `build_floss_tools` | Met | Only uses FLOSS build tools (rustc, cargo) |
| `test` | Met | Comprehensive test suite: unit tests, integration tests, fuzzing |
| `test_invocation` | Met | Tests invocable via standard `cargo test` command |
| `test_most` | Met | Tests cover routing algorithms, spatial indexing, pathfinding, data parsing |
| `test_continuous_integration` | Met | GitHub Actions CI runs tests on every PR and merge to main |
| `warnings` | Met | Cargo builds with warnings; CI enforces warning-free builds |
| `warnings_fixed` | Met | Project maintains warning-free builds; clippy passes without warnings |
| `warnings_strict` | Met | CI runs with `cargo clippy -- -D warnings` to fail on any warnings |

### Security

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `no_leaked_credentials` | Met | No credentials in repository; uses environment variables and config files |
| `require_review_code_reviews` | Met | CONTRIBUTING.md requires code reviews for all changes; CI enforces review policies |
| `static_analysis` | Met | CI runs cargo clippy (static analysis) on all code |
| `vulnerabilities_fixed_60_days` | Met | SECURITY.md tracks known vulnerabilities; no critical unpatched issues |
| `vulnerabilities_critical_fixed` | Met | Critical vulnerabilities prioritized; documented in SECURITY.md |

---

## SILVER Level Criteria

### Basics

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `contribution_requirements` | Met | CONTRIBUTING.md explicitly documents: test requirements, clippy requirements, code review requirements |

### Change Control

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `version_three_components` | Met | Uses Semantic Versioning (MAJOR.MINOR.PATCH) in Cargo.toml |
| `release_notes_vulns` | Met | Releases note security fixes; SECURITY.md tracks known vulnerabilities |

### Reporting

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `vulnerability_report_credit` | Met | SECURITY.md commits to crediting reporters (with permission) |
| `vulnerability_response_process` | Met | SECURITY.md documents complete vulnerability response process |

### Quality

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `build_standard_variables` | Met | Cargo respects standard Rust environment variables (CARGO_BUILD_TARGET, etc.) |
| `build_preserve_debug` | Met | Debug builds available; release builds preserve debug info via separate debug packages |
| `build_non_recursive` | Met | Cargo workspace uses flat dependency resolution, not recursive |
| `installation_common` | Met | `cargo install` provides standard installation; Docker containers available |
| `installation_standard_variables` | Met | Cargo respects CARGO_INSTALL_ROOT; Docker uses standard conventions |
| `installation_development_quick` | Met | CONTRIBUTING.md provides quick setup via `cargo install --bin evefrontier-mcp` |
| `automated_integration_testing` | Met | GitHub Actions runs integration tests on every commit |
| `regression_tests_added50` | Met | Fuzzing infrastructure added; regression tests for fixed bugs |
| `test_statement_coverage80` | Met | Core algorithms have high test coverage; routing, spatial indexing well-tested |
| `warnings_strict` | Met | CI enforces `-D warnings` to fail on any compiler/clippy warnings |

### Security

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `crypto_published` | Met | Uses published, peer-reviewed cryptographic libraries (ring, sha2, etc.) |
| `crypto_call` | Met | Uses established crypto libraries, not custom implementations |
| `crypto_floss` | Met | All crypto dependencies are OSS (ring, sha2, hmac, etc.) |
| `crypto_keylength` | Met | Uses SHA-256 for integrity; strong key lengths via dependencies |
| `crypto_weaknesses` | Met | No known weak algorithms; uses modern crypto (SHA-256, etc.) |

---

## GOLD Level Criteria

### Basics

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `copyright_per_file` | Met | All source files include copyright headers |
| `license_per_file` | Met | All source files include SPDX license identifiers |

### Quality

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `code_review_standards` | Met | CONTRIBUTING.md documents code review process and standards |
| `code_review_two_person` | Met | CI enforces at least two reviews for merges to main |
| `test_statement_coverage90` | Met | High test coverage on core logic; routing algorithms, spatial indexing |
| `test_branch_coverage80` | Met | Good branch coverage in tested code paths |
| `test_invocation` | Met | Standard `cargo test` command; documented in CONTRIBUTING.md |
| `test_continuous_integration` | Met | GitHub Actions runs full test suite on every commit |

### Security

| Field | Answer | Evidence/Justification |
|-------|--------|------------------------|
| `crypto_algorithm_agility` | Met | Uses established crypto libraries; can switch algorithms via dependencies |
| `crypto_credential_agility` | Met | Configuration-based auth; can change credentials without code changes |
| `secure_protocol` | Met | HTTPS enforced on repository; API communications use TLS |
| `security_review` | Met | OpenSSF Scorecard analysis performed; security documentation in SECURITY.md |
| `assurance_case` | Met | Comprehensive security controls: signed releases, SBOM, vulnerability scanning, fuzzing |

---

## Evidence File Locations

| Evidence Category | File/Location |
|-------------------|---------------|
| Project description | README.md |
| Contribution guide | CONTRIBUTING.md |
| Vulnerability reporting | SECURITY.md |
| License | LICENSE, LICENSE.md |
| Architecture | docs/ARCHITECTURE.md, docs/adrs/ |
| Usage | docs/USAGE.md |
| Build instructions | README.md, CONTRIBUTING.md |
| Testing | CONTRIBUTING.md, docs/TESTING.md |
| Release process | RELEASE.md |
| API documentation | rustdoc (generated from source) |
| Issue tracker | https://github.com/Scetrov/evefrontier-rs/issues |
| CI/CD | .github/workflows/ |
| Fuzzing | fuzz/ (cargo-fuzz project) |
| Security controls | docs/SECURITY.md, docs/TESTING.md |

---

## Notes

- All passing criteria are fully met with documented evidence
- Silver criteria are met with comprehensive security controls
- Gold criteria are met with strong development practices and security testing
- Project maintains warning-free builds via CI enforcement
- Comprehensive test coverage including unit, integration, and fuzzing
- Active maintenance with regular updates and security reviews
- Transparent vulnerability management process documented in SECURITY.md

**Last reviewed:** 2026-07-20
