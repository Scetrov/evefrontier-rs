# ADR 0002: Workspace structure (library + CLI crates)

## Status

Accepted

## Context

This repository is structured as a Rust workspace to separate concerns: core logic in a library
crate and CLI glue in a thin binary crate. Examples and helpers exist under `examples/`.

## Decision

Maintain the following structure:

- `crates/evefrontier-lib/` — core logic and public API.
- `crates/evefrontier-cli/` — CLI that depends on the library and contains only argument parsing and
  I/O.

## Rationale

- Clear separation of business logic and CLI makes testing and reuse easier.
- Library crate can be used programmatically by other tools or tests.

## Consequences

- Keep CLI code minimal. If functionality is needed in other consumers, add it to the library and
  keep CLI focused on parsing and user interaction.
