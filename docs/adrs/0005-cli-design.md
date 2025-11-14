# ADR 0005: CLI responsibilities — keep business logic in the library

## Status

Accepted

## Context

The project provides a CLI for users to download datasets and compute routes. Earlier exploratory
subcommands (`search`, `path`) have been merged into a single `route` subcommand that exposes all
routing capabilities (algorithm selection, spatial traversal, formatting) via flags. There are
choices about where to place parsing, validation and core logic.

## Decision

Keep the CLI (`crates/evefrontier-cli`) as a thin layer responsible for argument parsing,
configuration resolution, I/O and display. Implement core business logic in `crates/evefrontier-lib`
so it can be used programmatically by other tools and tests. Display concerns (output formats,
emoji/notes/basic forms) remain in the CLI, while route computation, graph selection and
summary building live in the library.

## Rationale

- Easier testing of logic without invoking the CLI.
- Reuse of library functions by other binaries or integration tests.

## Consequences

- New features that affect core behavior should be added to the library first.
- Additional output styles can be added without changing library APIs.
- Simplified user mental model: `download` manages data, `route` does all routing.
