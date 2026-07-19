# Threat model

## Scope

This document covers the repository's Rust route-planning library, CLI, Lambda functions,
Kubernetes services, and WASM modules. The system calculates routes and scouting paths from
EVE Frontier data, including temperature and fuel-aware route information. It does not replace
the vulnerability-reporting process in [SECURITY.md](../SECURITY.md).

## Assets and security objectives

- Source code, dependency lockfiles, CI/CD workflow definitions, and release configuration must
  be protected from unauthorized modification.
- Route and scouting inputs, downloaded data, and generated outputs must retain integrity so that
  users can make decisions from trustworthy results.
- Published container images, binaries, SBOMs, scans, and signatures must be traceable to the
  reviewed build that produced them.
- Repository and CI credentials, including GitHub tokens and configured secrets, must not be
  exposed or granted more access than the task requires.

## Trust boundaries and entry points

- CLI arguments, configuration, datasets, and downloaded input files cross from users or external
  data sources into route calculation and parsing code.
- Lambda, Kubernetes, and WASM deployment interfaces cross from callers and hosting platforms into
  service code; deployment-specific authentication, authorization, and network controls are owned
  by the environment and must be reviewed before deployment.
- Pull requests, workflow-dispatch inputs, tags, dependencies, and GitHub Actions cross into CI/CD.
  They are untrusted until validation and review complete.
- OCI registries and release consumers are external to the repository. Image digests, signatures,
  SBOMs, and provenance are the integrity evidence at this boundary.

## Primary threats and mitigations

### Untrusted route, scouting, or dataset input

Validate and bound inputs in the relevant parser and service entry point, and add regression tests
for rejected input. Maintainers must define deployment-specific request limits and monitoring.

### Compromised dependencies or mutable CI actions

Lock dependencies, review dependency changes, pin Actions to reviewed commits, and use Dependabot
updates. Administrators must enable dependency alerts and require dependency-review checks.

### Unreviewed workflow or release changes

Require PR review, protect workflow and release paths, use least-privilege tokens, and record build
provenance. Administrators must enforce branch protection, CODEOWNERS, and release permissions.

### Unsafe workflow-dispatch input

Pass dispatch values through step environment variables, quote shell expansion, and accept only
v-prefixed semantic versions. Maintainers must keep validation aligned with supported release-tag
formats.

### Forged or substituted OCI images

Publish image provenance, retain SBOMs, scan images, and verify cosign signatures and digests
before deployment. Release owners must verify registry-attached attestations and restrict
publishing.

### Secret exposure in source or CI logs

Use GitHub secret expressions rather than hardcoded values and limit token permissions.
Administrators must enable secret scanning and push protection.

## Security-sensitive decisions

- Docker releases use GitHub OIDC-backed cosign signing and now request Buildx provenance for
  published images. Consumers should verify signatures, digests, and provenance before deployment.
- CI workflows use explicit permissions and SHA-pinned third-party actions. Pin updates require
  review of the upstream commit and its release notes.
- The accepted DevSecOps practices in
  [ADR 0007](adrs/0007-devsecops-practices.md) govern CI checks, dependency scanning, and release
  integrity expectations.

## Assumptions and review cadence

This model deliberately does not assert deployment architecture, authentication, data
classification, retention, or operational ownership that is not documented in the repository.
Maintainers should review it when adding an external interface, changing release or credential
flows, introducing a new data source, or at least annually. Deployment owners should record the
applicable environment-specific controls alongside their deployment configuration.
