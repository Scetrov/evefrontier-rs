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
| `fixture-sync` | `pnpm nx run scripts:fixture-sync -- --args.source=<path> --args.target=<path>` | Sync fixture from source dataset |
| `fixture-create` | `pnpm nx run scripts:fixture-create -- --args.source=<path>` | Create minimal fixture database from source |
| `route-fixture-extract` | `pnpm nx run scripts:route-fixture-extract` | Extract route testing fixtures |

### Database Inspection

| Task | Command | Description |
|------|---------|-------------|
| `inspect-db` | `pnpm nx run scripts:inspect-db -- --args.path=<path>` | Inspect SQLite database schema and contents |
| `analyze-routes` | `pnpm nx run scripts:analyze-routes -- --args.db=<path> --args.csv=<path>` | Analyze sample routes from CSV |

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
| `extract_fixture_from_dataset.py` | Extract fixture from e6c3 dataset | `extract_fixture_from_dataset.py` (hardcoded targets) |
| `create_minimal_db.py` | Create minimal test database from e6c3 | `create_minimal_db.py` (wrapper for extract_fixture) |
| `extract_route_fixture.py` | Extract route testing scenarios | `extract_route_fixture.py` |
| `inspect_db.py` | Database schema inspection | `inspect_db.py <path>` (use via Nx: `pnpm nx run scripts:inspect-db -- --args.path=<path>`) |
| `analyze_sample_routes.py` | Route analysis from sample data | `analyze_sample_routes.py <db> <csv>` (use via Nx: `pnpm nx run scripts:analyze-routes -- --args.db=<db> --args.csv=<csv>`) |

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

## Examples

### Inspect a Database

```bash
# Inspect the fixture database
pnpm nx run scripts:inspect-db -- --args.path=docs/fixtures/minimal/static_data.db

# Inspect any SQLite database
pnpm nx run scripts:inspect-db -- --args.path=/path/to/database.db
```

### Analyze Routes

```bash
# Analyze routes from sample data
pnpm nx run scripts:analyze-routes -- --args.db=docs/fixtures/minimal/static_data.db --args.csv=docs/SampleRoutes.csv
```

### Create Fixture from Source

```bash
# Create fixture from downloaded e6c3 dataset
pnpm nx run scripts:fixture-create -- --args.source=/tmp/e6c3_source/static_data.db
```

### Sync Fixture

```bash
# Extract and sync fixture from source to target
pnpm nx run scripts:fixture-sync -- --args.source=/tmp/e6c3_source/static_data.db --args.target=docs/fixtures/minimal/static_data.db
```

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
