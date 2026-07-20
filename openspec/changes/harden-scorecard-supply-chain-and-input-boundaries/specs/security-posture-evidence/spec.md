## ADDED Requirements

### Requirement: Evidence-backed OpenSSF status
The repository SHALL register with the OpenSSF Best Practices program and SHALL publish an in-progress or passing status whose claims are supported by current repository documentation, automation, and public settings.

#### Scenario: Best Practices questionnaire is completed
- **WHEN** a maintainer answers a program criterion
- **THEN** the answer references verifiable project evidence or explicitly records that the criterion is not yet met

#### Scenario: Status is published
- **WHEN** the repository has an OpenSSF project record
- **THEN** the README or security documentation links to the current badge/status without claiming a higher level than awarded

### Requirement: Qualified Scorecard interpretation
Security documentation MUST distinguish OpenSSF Scorecard posture signals from confirmed exploitable vulnerabilities and SHALL retain each finding's evidence, repository context, disposition, and residual risk.

#### Scenario: Scorecard reports a new or recurring finding
- **WHEN** a Scorecard run emits an alert
- **THEN** maintainers verify the referenced code or setting before classifying it as actionable, accepted risk, detector limitation, or false positive

### Requirement: Post-remediation finding disposition
After the hardening changes are applied, the repository SHALL rerun Scorecard and SHALL document the disposition of alerts #42 through #51 without requiring detector score maximization as a release criterion.

#### Scenario: Technical finding is remediated
- **WHEN** a mutable container reference or other directly actionable condition is removed
- **THEN** the subsequent Scorecard result and code-scanning alert state are checked and recorded

#### Scenario: Single-maintainer review finding remains
- **WHEN** Code-Review or high-tier Branch-Protection still requires an unavailable independent reviewer
- **THEN** the alert is retained or dismissed with a specific accepted-risk rationale and the independent-review activation trigger

#### Scenario: Rust fuzzing is not detected
- **WHEN** repository fuzz targets run successfully but Scorecard still reports no recognized fuzzer integration
- **THEN** documentation records the detector limitation and treats OSS-Fuzz or ClusterFuzzLite as a separate future decision
