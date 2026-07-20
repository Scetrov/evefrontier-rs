## Review
- Correct: The change maps all ten live alerts. GitHub API evidence identifies #43–#48 as the six unpinned `FROM` references in the three service Dockerfiles, #42 as Branch-Protection, #49 as Code-Review, #50 as CII-Best-Practices, and #51 as Fuzzing. The proposed container, governance, Best Practices, and Rust-fuzz work respectively cover those findings; `tasks.md:3`, `tasks.md:36`, and `tasks.md:42-46` require recording and dispositioning #42–#51.
- Correct: The three input-boundary defects are grounded in current code: `crates/evefrontier-lib/src/fmap.rs:220` truncates `waypoints.len()` to `u16`; `crates/evefrontier-lib/src/spatial.rs:1015-1018` subtracts file-section sizes before allocation; and `crates/evefrontier-lib/src/graph.rs:372` overflows for a nonzero `usize::MAX` neighbor count. Lambda validation occurs before graph construction (`crates/evefrontier-lambda-route/src/lib.rs:89-124`), so the requested client-facing validation point is valid.
- Correct: The Nx direction is consistent with ADR 0017. `tasks.md:29` expressly requires non-cacheable serial fuzz targets, matching the ADR’s requirement to avoid concurrent Cargo processes; `nx.json:36-41` already marks security audit non-cacheable. The plan also deliberately retains the accepted single-maintainer and Rust-Scorecard-fuzzer-detection limitations.

- Concern — Medium: `openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/specs/repository-change-governance/spec.md:24-26` requires actual maintainer review of every external contribution, but no task makes that behavior enforceable or verifiable. Task 5.1 only documents it and task 5.4 only exercises stale/failed checks. The live `mainline` ruleset has `required_approving_review_count: 0`, so it cannot prove that assertion. Requiring one approval globally would conflict with the intentional one-maintainer constraint. **Smallest correction:** make this a documentation/policy requirement (for example, “governance documentation SHALL require maintainer review of external contributions before merge”), matching task 5.1, rather than an unverifiable runtime guarantee.

- Concern — Medium: `tasks.md:30` requires a new scheduled/manual workflow but does not preserve the repository’s established supply-chain controls for new workflow dependencies. Every current workflow action is SHA-pinned (for example `.github/workflows/scorecard.yml:25,31`), and `docs/threat-model.md:36-38` explicitly requires SHA-pinned third-party actions and least-privilege tokens. As written, task 4.6 can be completed with tag-pinned actions or unnecessarily broad permissions, undermining the stated baseline. **Smallest correction:** add to task 4.6 that the workflow uses explicit minimum permissions and full-commit-SHA-pinned actions.

- Concern — Low: The proposal contradicts the finalized design/tasks on the release Dockerfile: `proposal.md:7` says the image is pinned “in the generated release Dockerfile,” while `design.md:29`, `design.md:57`, and `tasks.md:9` require replacing that generated file with a checked-in Dockerfile. It also calls the active setting the “`main` ruleset” at `proposal.md:9`; the live API identifies the active ruleset as `mainline` targeting the default branch. **Smallest correction:** change proposal line 7 to “checked-in release Dockerfile” and line 9 to “`mainline` ruleset.”

- Note: No implementation tests were added or runnable because this is an unimplemented OpenSpec change. `pnpm nx show project evefrontier-rs --json` could not run locally because `node_modules` is absent (`pnpm` reported `Command "nx" not found`); this does not invalidate the static ADR alignment review. No tracked or staged repository files were modified by this review.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Three concrete concerns cite exact OpenSpec paths/lines and live repository/API evidence; all ten alerts were independently enumerated."
    }
  ],
  "changedFiles": [
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/proposal.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/design.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/tasks.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/specs/container-build-integrity/spec.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/specs/input-boundary-resilience/spec.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/specs/repository-change-governance/spec.md",
    "openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/specs/security-posture-evidence/spec.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "git status --short && git diff --cached --name-only",
      "result": "passed",
      "summary": "OpenSpec change and reviewer artifact directory are untracked; no staged files."
    },
    {
      "command": "gh api repos/Scetrov/evefrontier-rs/rulesets and gh api repos/Scetrov/evefrontier-rs/code-scanning/alerts",
      "result": "passed",
      "summary": "Verified the active mainline ruleset and alerts #42–#51."
    },
    {
      "command": "pnpm nx show project evefrontier-rs --json",
      "result": "failed",
      "summary": "Nx is unavailable because node_modules is not installed (pnpm: Command nx not found)."
    }
  ],
  "validationOutput": [
    "Static inspection confirmed current fmap truncation, spatial file-size subtraction, Lambda pre-graph validation location, workflow/action pinning baseline, Nx target defaults, ruleset JSON, and alert mapping.",
    "No source files were modified; no implementation tests were applicable."
  ],
  "residualRisks": [
    "External-contribution review remains documentation-only unless the requirement is narrowed or an enforceable policy becomes feasible.",
    "A new fuzz workflow could introduce mutable action dependencies unless task 4.6 explicitly retains SHA pinning and least privilege.",
    "Nx execution was not available in this checkout."
  ],
  "noStagedFiles": true,
  "diffSummary": "Review-only assessment of the untracked finalized OpenSpec change; no repository implementation diff exists.",
  "reviewFindings": [
    "medium: specs/repository-change-governance/spec.md:24-26 has an external-review behavior that tasks cannot enforce or verify under the single-maintainer constraint.",
    "medium: tasks.md:30 omits required SHA pinning and minimum permissions for the new fuzz workflow.",
    "low: proposal.md:7 and :9 conflict with the checked-in-release-Dockerfile decision and the live mainline ruleset name."
  ],
  "manualNotes": "Single-maintainer independent approval and Rust Scorecard fuzz-detection limitations were treated as intentional constraints, not findings."
}
```