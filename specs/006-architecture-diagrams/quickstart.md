# Quickstart: Adding/Updating Architecture Diagrams

This guide explains how to maintain the architecture diagrams in `docs/ARCHITECTURE.md`.

## Prerequisites

- A Markdown editor with Mermaid preview (VS Code with Mermaid extension, or GitHub web editor)
- Understanding of the EVE Frontier codebase structure

## Adding a New Diagram

1. **Choose the diagram type**:
   - `graph TD` or `graph LR` for component/flow diagrams
   - `sequenceDiagram` for time-ordered interactions

2. **Add to ARCHITECTURE.md** using a fenced code block:
   ````markdown
   ```mermaid
   graph LR
       A[Input] --> B[Process] --> C[Output]
   ```
   ````

3. **Test rendering**:
   - In VS Code: Use Markdown preview with Mermaid extension
   - In GitHub: Commit and view in GitHub's Markdown renderer
   - Locally: Use `npx @mermaid-js/mermaid-cli mmdc -i docs/ARCHITECTURE.md -o output.png` (optional)

## Diagram Style Guidelines

### Node Naming
- Use `PascalCase` for Rust crates: `EvefrontierLib`, `EvefrontierCli`
- Use `snake_case` for modules: `github_rs`, `db_rs`
- Use descriptive labels in brackets: `[Route Planner]`

### Grouping
Use subgraphs to group related components:
```mermaid
subgraph "Library Crate"
    A[Module A]
    B[Module B]
end
```

### Direction
- `TD` (top-down): For hierarchies and dependencies
- `LR` (left-right): For data flows and sequences

### Linking
- Solid arrows `-->` for direct calls/dependencies
- Dashed arrows `-.->` for optional/async relationships
- Labeled arrows `-->|label|` for named relationships

## Common Updates

### When adding a new module to evefrontier-lib

1. Add node to "Module Dependency Diagram" section
2. Connect to dependent/dependency modules
3. Update "Component Overview" if externally visible

### When adding a new Lambda function

1. Add to "Component Overview Diagram" Lambda section
2. Add sequence diagram showing handler flow
3. Reference existing Lambda patterns for consistency

### When changing data flow

1. Update "Data Flow Diagram" section
2. Verify sequence diagrams still accurate
3. Check that module arrows reflect new dependencies

## Verification Checklist

- [ ] Diagram renders in VS Code Markdown preview
- [ ] Diagram renders in GitHub (push to branch and view)
- [ ] Node names match actual crate/module names
- [ ] Arrows accurately represent dependencies
- [ ] Subgraph labels match documentation sections
- [ ] No Mermaid syntax errors (diagrams fail silently if broken)

## Resources

- [Mermaid Documentation](https://mermaid.js.org/intro/)
- [Mermaid Live Editor](https://mermaid.live/) - Test diagrams before committing
- [GitHub Mermaid Support](https://github.blog/2022-02-14-include-diagrams-markdown-files-mermaid/)
