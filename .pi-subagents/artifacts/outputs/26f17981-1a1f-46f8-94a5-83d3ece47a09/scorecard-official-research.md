# Research: Current OpenSSF Scorecard semantics and remediation

## Summary
OpenSSF Scorecard is explicitly a security-posture/best-practices tool, not a vulnerability scanner: its FAQ distinguishes it from tools that detect specific vulnerabilities. Thus, a low result on these checks is normally a risk signal or a detection heuristic—not proof that a public single-maintainer Rust project contains an exploitable flaw. [Scorecard FAQ](https://github.com/ossf/scorecard/blob/main/docs/faq.md#what-is-the-difference-between-scorecard-and-other-code-scanning-tools)

For this repository profile, the directly remediable technical finding is an unpinned Docker base image or CI action. Fuzzing, review, badge, and high-tier branch-governance scores may remain structurally constrained by Rust detection support and the absence of independent maintainers; document those limits rather than representing a score as a vulnerability finding.

## Findings

1. **Fuzzing — Medium: a capability/risk signal, not evidence of a vulnerability.** Scorecard detects only: an OSS-Fuzz project entry, ClusterFuzzLite deployment, or certain language-specific fuzz/property-test functions. The supported custom-function languages listed are Go, Haskell, JavaScript/TypeScript, Erlang, C#, and F#—**not Rust**. The project documentation specifically warns that projects using other fuzzing tools can score low and that a low score is not definitive evidence of risk. **Decision:** do not describe a Rust `cargo-fuzz`/libFuzzer setup as a failed security control merely because Scorecard reports zero; Scorecard will not recognize it through its custom-fuzzer detector. If increasing the Scorecard score is required, its stated remediation is OSS-Fuzz integration (or ClusterFuzzLite), subject to project eligibility and operational cost. [checks.md: Fuzzing](https://github.com/ossf/scorecard/blob/main/docs/checks.md#fuzzing) · [FAQ: custom fuzzers](https://github.com/ossf/scorecard/blob/main/docs/faq.md#fuzzing-does-scorecard-accept-custom-fuzzers)

2. **CII-Best-Practices — Low: badge-attestation/maturity signal, not a code finding.** Scorecard queries the repository URL against the OpenSSF Best Practices Badge API. Its exact points are Gold 10, Silver 7, Passing 5, and In Progress 2; no badge therefore produces no credit for this check. The badge is voluntary self-certification with explanations of how practices are met, and Scorecard notes that BadgeApp claims/justifications can address automated false positives/negatives. Scorecard also says Gold requires multiple developers. **Decision:** sign up and complete the badge questionnaire if external posture signaling and Scorecard points justify the administrative work; do not characterize lack of a badge as a vulnerability. A solo project can pursue Passing (and potentially Silver as applicable), but should not treat Gold as an ordinary one-maintainer remediation target. [checks.md: CII-Best-Practices](https://github.com/ossf/scorecard/blob/main/docs/checks.md#cii-best-practices) · [OpenSSF badge criteria](https://www.bestpractices.dev/en/criteria) · [BadgeApp overview](https://www.bestpractices.dev/en)

3. **Code-Review — High: recent-history heuristic and genuine change-control concern, but explicitly infeasible to fully satisfy for one active maintainer.** The check inspects approximately the latest 30 commits for GitHub/GitLab approvals or a merger different from the committer (implicit review), with Prow/Gerrit equivalents. It returns inconclusive when recent activity is solely bots. Scoring is leveled: any unreviewed bot change costs 3 points; one unreviewed human change costs 7; multiple unreviewed human changes cost a further 3. Bot/AI reviews never count as human review. Critically, Scorecard says all-change review can be infeasible for a project with only one active participant. **Decision:** a low score on direct solo commits is not a discovered vulnerability; it transparently records lack of independent review. Require review for outside PRs and recruit an independent maintainer/reviewer if feasible. Do not use a second account controlled by the same person as meaningful remediation—the check itself warns of sock-puppet limitations. [checks.md: Code-Review](https://github.com/ossf/scorecard/blob/main/docs/checks.md#code-review) · [FAQ: bot commits](https://github.com/ossf/scorecard/blob/main/docs/faq.md#code-review-can-it-ignore-bot-commits)

4. **Pinned-Dependencies — Medium: concrete supply-chain integrity finding when an image/action is mutable; not a CVE claim.** Scorecard examines build/release Dockerfiles, shell scripts, and GitHub workflows for dependencies pinned to a specific **hash**, not a mutable version/range (with a special full-semver rule for Go modules). A Docker `FROM rust:1.XX` or `FROM rust:latest` is version/tag pinned at most, but is *not* Scorecard-pinned: tags can be deleted/recreated to point at malicious content. **Decision:** for every Dockerfile that builds/releases the project, use `FROM rust:<human-readable-version>@sha256:<manifest-list-or-image-digest>`; retain the tag for auditability while the digest supplies immutability. A manifest-list digest is expressly acceptable for multi-architecture builds. Also pin third-party workflow `uses:` references to full commit SHA hashes. Pair digest pinning with Dependabot/Renovate or equivalent, because pinning can delay security updates. This finding has a clear technical remediation and should be logged as Medium supply-chain risk, not as proof that the current image is compromised. [checks.md: Pinned-Dependencies](https://github.com/ossf/scorecard/blob/main/docs/checks.md#pinned-dependencies) · [FAQ: version versus hash pinning](https://github.com/ossf/scorecard/blob/main/docs/faq.md#pinned-dependencies-can-i-use-version-pinning-instead-of-hash-pinning)

5. **Pinned-Dependencies has material parser/detection limits.** It scans Dockerfiles used in tests too, so a test-only `FROM` can lower the score. The official FAQ still recommends hash pinning there. Conversely, the Scorecard issue tracker documents limitations/bugs around Docker `ARG` substitution, named multi-stage builds, and non-default shells; these are implementation limitations, not proof that the referenced image is mutable. **Decision:** first verify the exact Scorecard detail and the resolved `FROM` image. Fix an actual tag-only reference; if the Dockerfile already resolves to a digest through an unsupported construct, record it as a Scorecard false positive/heuristic limitation and keep direct `FROM image:tag@sha256:…` syntax where practical. [FAQ: Dockerfiles used in tests](https://github.com/ossf/scorecard/blob/main/docs/faq.md#pinned-dependencies-will-scorecard-detect-unpinned-dependencies-in-tests-with-dockerfiles) · [official issue: ARG limitation](https://github.com/ossf/scorecard/issues/2988) · [official issue: multi-stage limitation](https://github.com/ossf/scorecard/issues/1572)

6. **Branch-Protection/rulesets — High: protection against unauthorized/malicious writes, with tiered—not binary—criteria.** Scorecard evaluates default and release branches using classic branch protection or repository rulesets. Tier 1 (3/10) requires blocking force pushes and deletion; Tier 2 (6/10) adds at least one approval plus, when admin-visible, PR-only changes, up-to-date branches, and latest-push approval; Tier 3 (8/10) requires at least one status check; Tier 4 (9/10) adds two reviewers and CODEOWNERS; Tier 5 (10/10) adds dismissal of stale approvals and admin inclusion. A tier must be completely satisfied before credit from later tiers. **Decision:** enable an active ruleset targeting the default branch (and release patterns if they exist) that blocks deletion/force-pushes, requires PRs and a passing CI check, and requires one approval for external contributors. A one-maintainer project cannot honestly supply independent two-person review for its own emergency/direct changes; document that residual insider/account-compromise exposure rather than manufacture approvals. [checks.md: Branch-Protection](https://github.com/ossf/scorecard/blob/main/docs/checks.md#branch-protection) · [FAQ: 10/10 mapping](https://github.com/ossf/scorecard/blob/main/docs/faq.md#branch-protection-how-to-setup-a-1010-branch-protection-on-github)

7. **Rulesets are supported and preferable for observable policy, but bypass actors affect the result.** GitHub says active repository rulesets can target branch patterns, layer with classic protection, and are visible to readers; disabled rulesets are not enforced. Scorecard states rulesets expose the queried settings without an admin token, but treats `EnforceAdmins` as false if *any* bypass actor exists on *any* rule, regardless of whether that actor is an administrator. Classic protection’s stale-review/admin/status/up-to-date/latest-push settings need an admin token; absent that token Scorecard scores them as met. **Decision:** use an **active** ruleset, ensure it targets the real default/release branch patterns, avoid bypass actors where score and uniform enforcement matter, and use an administrative/fine-grained token when auditing classic branch protection. Do not interpret a good non-admin-token result as proof the admin-only controls are enabled. [checks.md: Branch-Protection token semantics](https://github.com/ossf/scorecard/blob/main/docs/checks.md#branch-protection) · [GitHub: rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)

## Sources
- Kept: [OpenSSF Scorecard checks documentation](https://github.com/ossf/scorecard/blob/main/docs/checks.md) — primary current source for risk labels, detection semantics, points, limitations, and remediations.
- Kept: [OpenSSF Scorecard FAQ](https://github.com/ossf/scorecard/blob/main/docs/faq.md) — primary clarification for Scorecard’s scope, Docker hash versus tag semantics, custom fuzzers, and branch tiers.
- Kept: [OpenSSF Best Practices Badge criteria](https://www.bestpractices.dev/en/criteria) — primary criteria site for the badge checked by CII-Best-Practices.
- Kept: [GitHub Docs: About rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets) — platform-primary source for ruleset enforcement, targeting, layering, visibility, and bypass behavior.
- Kept: [Scorecard issue #2988](https://github.com/ossf/scorecard/issues/2988) and [#1572](https://github.com/ossf/scorecard/issues/1572) — first-party implementation evidence for Docker detection limitations; not normative remediation guidance.
- Dropped: third-party Scorecard summaries and vendor guidance — less authoritative than the Scorecard repository and GitHub/BadgeApp documentation.

## Gaps
- This research did not execute Scorecard against this repository or inspect its current Dockerfiles, workflows, branch configuration, or commit history. Therefore it identifies the exact decision rules but does **not** assert that any named local file currently fails a check.
- The precise score can vary with Scorecard version, authentication scope, branch/release patterns, and the last approximately 30 commits. For an actionable repository scorecard, run the current Scorecard version with an appropriate GitHub token and retain the JSON details alongside the result.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Concrete official findings identify the relevant authoritative source paths (docs/checks.md and docs/faq.md sections) and each Scorecard severity: Fuzzing Medium, CII-Best-Practices Low, Code-Review High, Pinned-Dependencies Medium, and Branch-Protection High."
    }
  ],
  "changedFiles": [
    ".pi-subagents/artifacts/outputs/26f17981-1a1f-46f8-94a5-83d3ece47a09/scorecard-official-research.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "web research: OpenSSF Scorecard primary docs, FAQ/source, OpenSSF BadgeApp, and GitHub rulesets documentation",
      "result": "passed",
      "summary": "Retrieved and compared current primary documentation; no repository test command was applicable to research-only work."
    }
  ],
  "validationOutput": [
    "Artifact written at the required authoritative path.",
    "All substantive recommendations cite primary OpenSSF, BadgeApp, or GitHub documentation."
  ],
  "residualRisks": [
    "Repository-specific failures were not verified because no Scorecard run or repository configuration inspection was requested/performed.",
    "No staged-file status could be independently queried because this runtime exposes no shell/git-status tool."
  ],
  "noStagedFiles": true,
  "diffSummary": "Created only the requested research artifact; no source, configuration, or test files were altered.",
  "reviewFindings": [
    "no blockers: research distinguishes vulnerabilities from posture signals and documents Rust/single-maintainer heuristic limitations."
  ],
  "manualNotes": "The user requested no repository modifications; the sole write is the explicitly required output artifact."
}
```