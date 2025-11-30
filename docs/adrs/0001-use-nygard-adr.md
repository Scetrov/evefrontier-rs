# ADR 0001: Use Nygard-style Architecture Decision Records (modified test)

## Status

Accepted

## Context

This repository contains multiple components (a library crate and a CLI) with designs and decisions
that should be documented and discoverable. We need a consistent, minimal, and well-known ADR
format.

## Decision

Adopt the Michael Nygard / Martin Fowler ADR format and numbering scheme for this repository. Each
record will live under `docs/adrs/` and be named with a zero-padded ID, a short slug, and `.md`
extension (for example: `docs/adrs/0001-use-nygard-adr.md`).

## Rationale

- Nygard ADRs are widely used and simple to write.
- Storing ADRs in `docs/adrs/` keeps them with the codebase and under version control.
- Zero-padded IDs make ordering obvious and cross-references easy.

## Consequences

- Future decisions should add new ADR files incrementally. Do not edit historical ADRs — append
  follow-ups as new ADRs.

## References

- Architecture Decision Record (ADR) template and guidance: Joel Parker Henderson's ADR repository
  (search for "architecture_decision_record" on GitHub).
