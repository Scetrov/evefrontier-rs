# Coding guidelines

These guidelines supplement the instructions in `.github/copilot-instructions.md` and
`CONTRIBUTING.md`. Use them as a quick reference when reviewing or writing code for the
EVE Frontier workspace.

## Code complexity

This project enforces code complexity limits to ensure maintainability. The thresholds are
configured in `clippy.toml` and enforced via CI and Nx tasks.

### Complexity thresholds

| Metric | Threshold | Description |
|--------|-----------|-------------|
| Cognitive complexity | 15 | Maximum complexity score per function (clippy's `cognitive_complexity`) |
| Lines per function | 100 | Maximum lines in a function body (`too_many_lines`) |
| Nesting depth | 8 | Maximum control flow nesting (`excessive_nesting`) |
| Function arguments | 8 | Maximum parameters per function (`too_many_arguments`) |

### Running complexity checks

```bash
# Check entire workspace
pnpm nx run-many -t complexity --exclude evefrontier-rs

# Check a single crate
pnpm nx run evefrontier-lib:complexity

# Or using cargo directly
cargo clippy --workspace --all-targets -- \
  -W clippy::cognitive_complexity \
  -W clippy::too_many_lines \
  -W clippy::excessive_nesting \
  -D warnings
```

### Reducing complexity

When a function exceeds these thresholds:

1. **Extract helper functions**: Break large functions into smaller, focused helpers.
2. **Use early returns**: Replace nested `if-else` chains with guard clauses.
3. **Simplify conditionals**: Use `match` patterns or boolean combinators to reduce branches.
4. **Introduce intermediate types**: If a function has too many arguments, consider a builder or
   options struct.

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

## Testing

- Always finish a development session with a clean run of `cargo fmt`,
  `cargo clippy --all-targets --all-features`, and `cargo test --workspace --locked`.
  Document any deviations in the pull request so reviewers understand why a
  check could not be executed locally.
