## Description
<!-- Provide a brief description of the changes in this PR -->

## Related Issue(s)
<!-- Link to related issues (e.g., Fixes #123, Related to #456) -->

## Type of Change
<!-- Mark the relevant option with an [x] -->

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] CI/tooling change

## Checklist

### General
- [ ] My code follows the project's coding standards and Rust best practices
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings or errors
- [ ] I have run `cargo fmt` and `cargo clippy` locally
- [ ] All existing tests pass locally (`cargo test --workspace`)

### Testing (if applicable)
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally
-- [ ] I have tested my changes with the fixture dataset (`docs/fixtures/minimal/static_data.db`)

### Documentation (if applicable)
- [ ] I have updated `CHANGELOG.md` with my changes under the `Unreleased` section
- [ ] I have updated relevant documentation in `docs/` or Rustdoc comments
- [ ] I have updated `README.md` or `USAGE.md` if user-facing behavior changed

### Architecture Decision Records (ADRs)
<!-- See Constitution Principle III and ADR 0001 for ADR governance -->
- [ ] **If this PR adds/modifies ADRs**: I understand that ADRs are immutable after ratification
  - New ADRs must follow the naming pattern: `docs/adrs/NNNN-slug-title.md`
  - Editing existing ADRs requires the `allow-adr-edits` label (typos/corrections only)
  - Substantive changes require a new ADR that supersedes the original
- [ ] **If architecturally significant**: I have created a new ADR documenting this decision
- [ ] **If updating an existing ADR for typos**: I have requested the `allow-adr-edits` label

### Security (if applicable)
- [ ] I have followed security best practices (input validation, parameterized queries, no hardcoded secrets)
- [ ] I have reviewed `.github/instructions/security-and-owasp.instructions.md`
- [ ] `cargo audit` passes with no new vulnerabilities

## Additional Notes
<!-- Add any additional context, screenshots, or notes for reviewers -->
