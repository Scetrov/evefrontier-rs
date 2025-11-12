---
agent: agent
name: adr-alignment-check
description: Review Architecture Decision Records (ADRs) for alignment with the current codebase.
model: Auto (copilot)
---

Review the documentation in the #file:../../docs folder, especially the **Architecture Decision Records (ADRs)**, and compare them against the current state of the codebase.

Specifically:

1. Identify which ADRs have corresponding implementations in  the #file:../../crates directory.
2. For each ADR, determine whether the current implementation **fully**, **partially**, or **does not** align with the decision described.
3. Note any **deviations** or **inconsistencies**, explaining what differs and where in the code.
4. Suggest **corrective actions** or updates (to either code or ADRs) if alignment is off.

Summarize your findings in a concise markdown report in docs with the filename `docs/adr-alignment-report_YYYY-MM-DD.md` and then update #file:../../docs/TODO.md to include any missing tasks.
