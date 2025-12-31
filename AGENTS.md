<!-- nx configuration start-->
<!-- Leave the start & end comments to automatically receive updates. -->

# General Guidelines for working with Nx

- When running tasks (for example build, lint, test, e2e, etc.), always prefer running the task
  through `nx` (i.e. `nx run`, `nx run-many`, `nx affected`) instead of using the underlying tooling
  directly
- You have access to the Nx MCP server and its tools, use them to help the user
- When answering questions about the repository, use the `nx_workspace` tool first to gain an
  understanding of the workspace architecture where applicable.
- When working in individual projects, use the `nx_project_details` mcp tool to analyze and
  understand the specific project structure and dependencies
- For questions around nx configuration, best practices or if you're unsure, use the `nx_docs` tool
  to get relevant, up-to-date docs. Always use this instead of assuming things about nx
  configuration
- If the user needs help with an Nx configuration or project graph error, use the `nx_workspace`
  tool to get any errors
- **Important**: For task configuration guidance, workspace patterns, and cache behavior, consult
  [ADR 0017: NX Repository Orchestration Strategy](docs/adrs/0017-nx-orchestration-strategy.md).
  This ADR documents the rationale for `parallel: false`, named inputs, target defaults, and 
  provides examples for configuring new projects.

<!-- nx configuration end-->

Note: Before starting work on a change or task, always read the repository-level guidance in
`.github/copilot-instructions.md` and then check the `docs/` and `docs/adrs/` folders for any
relevant documentation or ADRs that could affect your design or implementation choices. These
documents often include important conventions, schema notes, and CI/packaging guidance that should
be considered before making changes.

