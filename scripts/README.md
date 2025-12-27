# Scripts

This folder contains utility scripts for managing the evefrontier-rs workspace. All scripts are
registered as Nx targets and can be executed via `pnpm nx run scripts:<target>`.

## Setup

Before running Python scripts, set up the virtual environment:

```bash
pnpm nx run scripts:venv-setup
```

This creates a `.venv` folder in the scripts directory and installs dependencies from
`requirements.txt`.

## Available Tasks

### Fixture Management

| Task | Command | Description |
|------|---------|-------------|
| `fixture-verify` | `pnpm nx run scripts:fixture-verify` | Verify fixture integrity against recorded metadata |
| `fixture-status` | `pnpm nx run scripts:fixture-status` | Display current fixture status and statistics |
| `fixture-record` | `pnpm nx run scripts:fixture-record` | Record current fixture metadata (after updates) |
| `fixture-sync` | `pnpm nx run scripts:fixture-sync <source> <target>` | Sync fixture from source dataset |
| `fixture-create` | `pnpm nx run scripts:fixture-create <source>` | Create minimal fixture database |
| `route-fixture-extract` | `pnpm nx run scripts:route-fixture-extract` | Extract route testing fixtures |

### Database Inspection

| Task | Command | Description |
|------|---------|-------------|
| `inspect-db` | `pnpm nx run scripts:inspect-db <path>` | Inspect SQLite database schema and contents |
| `analyze-routes` | `pnpm nx run scripts:analyze-routes <db> <csv>` | Analyze sample routes from CSV |

### Meta Tasks

| Task | Command | Description |
|------|---------|-------------|
| `verify-all` | `pnpm nx run scripts:verify-all` | Run all verification tasks |
| `venv-setup` | `pnpm nx run scripts:venv-setup` | Set up Python virtual environment |

## Script Reference

### Python Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `fixture_status.py` | Fixture verification & status | `fixture_status.py [verify\|status\|record]` |
| `extract_fixture_from_dataset.py` | Extract fixture from dataset | `extract_fixture_from_dataset.py <src> <tgt>` |
| `create_minimal_db.py` | Create minimal test database | `create_minimal_db.py <source>` |
| `extract_route_fixture.py` | Extract route testing scenarios | `extract_route_fixture.py` |
| `inspect_db.py` | Database schema inspection | `inspect_db.py <path>` |
| `analyze_sample_routes.py` | Route analysis from sample data | `analyze_sample_routes.py <db> <csv>` |

### Node.js Scripts

These scripts are used primarily by CI and pre-commit hooks:

| Script | Purpose |
|--------|---------|
| `check-pnpm-outdated.js` | Check for outdated pnpm dependencies |
| `outdated-report.js` | Generate dependency update reports |
| `run-audit.js` | Run security audit |
| `run-markdownlint-if-exists.js` | Conditional markdown linting |
| `run-precommit-nx.js` | Pre-commit Nx task runner |
| `run-prettier-if-exists.js` | Conditional code formatting |

## CI Integration

The `fixture-verify` task is cached based on inputs and should be included in CI pipelines:

```yaml
- name: Verify fixtures
  run: pnpm nx run scripts:fixture-verify
```

For local development, run `fixture-status` to check current state without failing on mismatches.

## Adding New Scripts

1. Add the script to this folder
2. If it's a Python script requiring dependencies, add them to `requirements.txt`
3. Register the script as an Nx target in `project.json`
4. Document the script in this README
5. Test the task: `pnpm nx run scripts:<new-target>`

## Virtual Environment

The `.venv` folder is gitignored. Each developer/CI run creates its own environment via
`venv-setup`. The `requirements.txt` file should be kept minimal - currently all scripts use
Python stdlib only.
