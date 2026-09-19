## 1. Diagnose and remediate dependency resolution

- [x] 1.1 Confirm the affected lockfile, advisory range, dependency source, and patched threshold.
- [x] 1.2 Update only the vulnerable fuzz-workspace `rustls` entry to version 0.23.45.
- [x] 1.3 Review the lockfile diff and confirm no unrelated dependency resolution changed.

## 2. Validate security and regression behavior

- [x] 2.1 Confirm every committed Cargo lockfile excludes versions affected by RUSTSEC-2026-0285.
- [x] 2.2 Validate locked fuzz-workspace metadata and run applicable repository checks.
- [x] 2.3 Run configured pre-commit validation.

## 3. Deliver review evidence

- [x] 3.1 Archive this completed OpenSpec change before the final commit.
- [x] 3.2 Prepare a conventional commit and pull request with security impact, validation, and
      backout details.
