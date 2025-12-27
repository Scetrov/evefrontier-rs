# Research: Automation Scripts & Nx Task Integration

**Feature**: 002-automation-scripts-nx-tasks  
**Created**: 2025-12-27

---

## Research Questions Addressed

### 1. How to pass arguments to Nx run-commands tasks?

**Decision**: Use `{args.paramName}` syntax in command strings

**Rationale**: This is the documented Nx pattern for argument forwarding. Users invoke with
`pnpm nx run scripts:task-name -- --args.paramName=value`.

**Alternatives Considered**:
- Positional arguments: Less explicit, harder to document
- Environment variables: More complex, not suitable for ad-hoc runs

**Source**: Nx documentation for `nx:run-commands` executor

---

### 2. Which scripts should be cached?

**Decision**: Only `fixture-verify` is cacheable; all others are non-cacheable

**Rationale**:
- `fixture-verify`: Deterministic output based on DB file + metadata file
- `fixture-status`: Ad-hoc query, no benefit from caching
- `fixture-sync`, `fixture-record`: Mutating operations, must not cache
- `inspect-db`: Ad-hoc query with variable input

**Alternatives Considered**:
- Cache all read-only tasks: Rejected because ad-hoc tasks have no stable inputs

---

### 3. What are the Python version requirements?

**Decision**: Python 3.10+ with venv (managed via requirements.txt)

**Rationale**: 
- All existing scripts use Python 3 features
- venv enables clean dependency isolation and future package additions
- sqlite3 module included in stdlib
- requirements.txt allows easy CI integration

**Alternatives Considered**:
- Python 3.8+: Too permissive, may allow older features
- Python 3.12+: Too restrictive for some developer machines
- stdlib-only (no venv): Less flexible for future dependencies

---

### 4. Should scripts project be in pnpm workspace?

**Decision**: No - scripts project is an Nx project only, not a Node package

**Rationale**:
- Scripts are Python + standalone Node.js files
- No npm dependencies specific to scripts
- Nx discovers projects via `project.json`, not `package.json`

**Alternatives Considered**:
- Create `scripts/package.json`: Unnecessary complexity

---

### 5. How should the project.json be structured?

**Decision**: Single project with multiple targets, one per script function

**Rationale**:
- Aligns with existing crate project.json pattern
- Clear, discoverable task names
- Supports future expansion

**Project Structure**:
```
scripts/
├── project.json          # Nx project configuration
├── README.md             # Documentation
├── requirements.txt      # Python dependencies (venv)
├── .venv/                # Python virtual environment (gitignored)
├── fixture_status.py     # Fixture management
├── extract_fixture_from_dataset.py
├── inspect_db.py
├── create_minimal_db.py
├── analyze_sample_routes.py
├── extract_route_fixture.py
├── run-audit.js          # Already integrated via crates
├── outdated-report.js    # Already in package.json
├── check-pnpm-outdated.js
├── run-markdownlint-if-exists.js
├── run-precommit-nx.js
└── run-prettier-if-exists.js
```

---

### 6. Should fixture-sync be runnable in CI?

**Decision**: YES - fixture-sync should work in both CI and local environments

**Rationale**:
- Enables automated fixture regeneration when datasets change
- CI can verify fixture consistency on PRs
- Must work non-interactively (no prompts)

**Implementation**:
- Scripts must not require interactive input
- Exit codes must be deterministic
- Error messages must be CI-friendly (no ANSI codes in non-TTY)

---

### 7. Should we add a meta-task for all verification?

**Decision**: YES - Add `verify-all` meta-task

**Rationale**:
- Single command to run all verification checks
- Useful for pre-commit and CI integration
- Uses Nx `dependsOn` for task orchestration

**Implementation**:
```json
"verify-all": {
  "dependsOn": ["fixture-verify"],
  "options": {
    "command": "echo 'All verification tasks passed'"
  }
}
```

---

## Existing Script Analysis

### Python Scripts (Need Nx Integration)

| Script | Purpose | Arguments | Output |
|--------|---------|-----------|--------|
| `fixture_status.py` | Fixture metadata management | `status\|record\|verify` | JSON/text |
| `extract_fixture_from_dataset.py` | Extract test fixture | `<source_db> <target_db>` | SQLite DB |
| `inspect_db.py` | Database inspection | `<db_path>` | Text report |
| `create_minimal_db.py` | Alternative fixture creator | `<source_db>` | SQLite DB |
| `analyze_sample_routes.py` | Route analysis | `<db> <csv>` | Analysis report |
| `extract_route_fixture.py` | Route fixture extraction | (hardcoded paths) | SQLite DB |

### Node.js Scripts (Already Integrated)

| Script | Integration Point |
|--------|-------------------|
| `run-audit.js` | Via evefrontier-lib project.json audit task |
| `outdated-report.js` | Via package.json `outdated:node` script |
| `check-pnpm-outdated.js` | Via package.json |
| `run-markdownlint-if-exists.js` | Via package.json lint:md |
| `run-precommit-nx.js` | Via husky pre-commit hook |
| `run-prettier-if-exists.js` | Via package.json format |

---

## Best Practices Applied

### From ADR 0006 (Software Components)

- ✅ Use Nx for task orchestration
- ✅ Document Python as development dependency
- ✅ Keep developer tooling separate from runtime

### From ADR 0007 (DevSecOps Practices)

- ✅ Scripts don't handle secrets
- ✅ No elevated permissions required
- ✅ Deterministic outputs where possible

### From Copilot Instructions

- ✅ Small, focused changes
- ✅ Prefer library code over scripts where possible
- ✅ Document changes in relevant files

---

## Technical Notes

### Nx Run-Commands Executor

The `nx:run-commands` executor supports:
- `command`: Shell command to execute
- `cwd`: Working directory (use `{workspaceRoot}`)
- `args`: Arguments forwarded via `{args.name}` interpolation
- `parallel`: Whether commands can run in parallel (default: true)

### Caching Configuration

Cacheable tasks need:
- `inputs`: Files that affect output
- `outputs`: Files produced by task (for restore)
- `cache: true` in target definition

### Argument Forwarding Pattern

```bash
# Definition in project.json
"command": "python3 scripts/foo.py {args.source} {args.target}"

# Invocation
pnpm nx run scripts:task -- --args.source=/a --args.target=/b
```

---

## Dependencies Verified

- [x] Python 3.10+ available in development environment
- [x] Nx 20+ installed via pnpm
- [x] All scripts use stdlib modules only (sqlite3, json, pathlib, argparse)
- [x] No circular dependencies between scripts
