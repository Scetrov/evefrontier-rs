# Task for reviewer

Review the finalized OpenSpec change at openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/ against the live repository and the ten Scorecard alerts #42-#51. Do not modify files. Find only concrete blockers/concerns: internal contradictions among proposal/design/specs/tasks; claims not grounded in current APIs/files; impossible or unsafe requirements; missed mapping of any alert; violations of Nx/ADR 0017; tasks that cannot verify a requirement. Return severity, exact artifact path/section, evidence, and smallest correction. Treat single-maintainer independent approval and Rust Scorecard fuzz detection limitations as intentional constraints.

---
**Output:**
Write your findings to exactly this path: /home/scetrov/source/evefrontier-rs/.pi-subagents/artifacts/outputs/4ae34ea3-f0e9-4978-86c1-24756ebf495e/openspec-scorecard-review.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
`criteriaSatisfied[].status` must be exactly one of: satisfied, not-satisfied, not-applicable.
`commandsRun[].result` must be exactly one of: passed, failed, not-run.
`manualNotes` and `notes` are optional strings; an empty string means no note and does not satisfy `manual-notes` evidence.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```