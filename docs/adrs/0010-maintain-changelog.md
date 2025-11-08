---
title: "Maintain a repository CHANGELOG.md"
date: 2025-11-08
status: proposed
---

Context
-------

A clear, human-readable changelog at the repository root helps contributors, maintainers,
and automation understand what changed and why. Many tools and release processes rely on
structured or well-formed changelogs; currently this repository does not enforce a single
canonical changelog file.

Motivation
----------

- Ensure every change (including small automated edits made by LLMs/agents) is recorded with a
  short, searchable summary.
- Provide a single place for release notes and human-friendly history that complements Git
  history and attached release artifacts.
- Make it easy for reviewers and downstream integrators to quickly see what changed between
  releases or commits.

Decision
--------

We will add a canonical `CHANGELOG.md` at the repository root and adopt a lightweight process
that requires the modifying agent (human or automated) to append a short summary entry for each
change they introduce. The LLM/agent must include a one-line summary plus an optional short
paragraph describing intent and touchpoints (files changed, ADRs referenced). Automated edits
made by an LLM should be clearly marked with an `[auto-llm]` tag and include a generated
summary that a human maintainer can expand if needed.

The changelog will follow a conservative, human-centric format inspired by "Keep a Changelog"
but simplified for our needs:

- Top-level sections per release (e.g., `Unreleased`, `v0.1.0`), with `Unreleased` being the
  default working area.
- Each entry includes: date (YYYY-MM-DD), author (name or agent id), tag (manual/auto-llm), a
  one-line summary, and optional details.

Consequences
------------

- Pros:
  - Improves discoverability of changes and rationale.
  - Makes LLM/agent edits auditable and easier to review.
  - Simplifies release note generation.

- Cons:
  - Requires discipline from contributors and tooling support for automation.
  - May duplicate some information already present in commit messages.

Implementation
--------------

1. Create `CHANGELOG.md` in the repository root with an initial `Unreleased` section and a
   short example entry.
2. Update contributor guidance (in `CONTRIBUTING.md`) to require that any PR or LLM/agent patch
   include a matching changelog entry under `Unreleased` before merging. Automated agents must
   append a changelog entry when they apply a change. Human reviewers should verify the
   changelog entry for clarity and accuracy.
3. For LLM/agent edits, require the following mini-contract to be added as a single paragraph
   appended to `CHANGELOG.md`:

   - Date: YYYY-MM-DD
   - Author: `auto-llm:<model-id>` or the contributor name
   - Tag: `[auto-llm]` for machine-generated entries, `[manual]` for human entries
   - Summary: one-line description
   - Details: optional 1-3 sentence explanation and files changed

4. Add an optional CI check (GitHub Actions) that warns if a PR modifies code/docs but doesn't
   include a changelog entry in `Unreleased`. The check should be advisory at first and may be
   promoted to blocking once workflows and contributors have adapted.
5. Provide an example entry format and small helper script in `scripts/` (optional) to
   automatically append or validate entries. The script may be used by LLM agents to add a
   pre-formatted entry.

Example entry
-------------

Unreleased

- 2025-11-08 — auto-llm: Added ADR `0009-kd-tree-spatial-index.md` and updated `USAGE.md`.
  [auto-llm]
  - Details: Added ADR proposing precomputed spatial index; clarified CLI invocation examples
    in `docs/USAGE.md`. Files changed: `docs/adrs/0009-kd-tree-spatial-index.md`,
    `docs/USAGE.md`.

Notes and future work
---------------------

- Decide whether to automatically generate release notes from `CHANGELOG.md` when creating a
  release tag (recommended).
- Consider keeping machine-generated entries flagged in a separate index or metadata file if
  privacy or volume becomes a concern.

See also
--------

- `docs/adrs/0007-devsecops-practices.md` (CI & release guidance)
- `CONTRIBUTING.md`
