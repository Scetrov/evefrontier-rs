# Coding guidelines

These guidelines supplement the instructions in `.github/copilot-instructions.md` and
`CONTRIBUTING.md`. Use them as a quick reference when reviewing or writing code for the
EveFrontier workspace.

## Control flow

- Prefer early returns and guard clauses to keep functions shallow and avoid excessive nesting.
- Extract helper functions when a block of logic no longer fits comfortably on a screen or when a
  descriptive name clarifies intent.

## Error handling

- Propagate errors with context using the existing error types rather than swallowing failures.
- Reserve `unwrap`/`expect` for tests and places where a panic conveys a logic bug.

## Style

- Follow `cargo fmt` for formatting and `cargo clippy --all-targets --all-features` to catch common
  mistakes.
- Keep user-facing strings localized in a single place when practical so the CLI and Lambdas remain
  consistent.
