# Project Consistency Analysis Report

**Feature**: 002-automation-scripts-nx-tasks  
**Generated**: 2025-12-27  
**Purpose**: Validate existing patterns before implementing scripts Nx integration

---

## Environment Check

| Component | Status | Value |
|-----------|--------|-------|
| Python Version | ✅ AVAILABLE | 3.13.5 (exceeds 3.10+ requirement) |
| Python Path | ✅ SYSTEM | `/usr/bin/python3` |
| Nx Workspace | ✅ CONFIGURED | 7 projects discovered |
| Scripts Directory | ✅ EXISTS | 12 files (6 Python, 6 Node.js) |
| .gitignore venv | ✅ CONFIGURED | `scripts/.venv/`, `.venv/`, `venv/` |

---

## Existing Nx Projects

| Project | Type | Tags |
|---------|------|------|
| evefrontier-lib | library | `type:library, lang:rust` |
| evefrontier-cli | application | `type:application, lang:rust` |
| evefrontier-lambda-shared | library | `type:library, lang:rust, scope:lambda` |
| evefrontier-lambda-route | application | `type:application, lang:rust, scope:lambda` |
| evefrontier-lambda-scout-gates | application | `type:application, lang:rust, scope:lambda` |
| evefrontier-lambda-scout-range | application | `type:application, lang:rust, scope:lambda` |
| evefrontier-rs | (root) | - |

### Tag Pattern Analysis

- `type:` prefix for project type (library, application)
- `lang:` prefix for language (rust)
- `scope:` prefix for domain scope (lambda)

**Recommended for scripts project**: `type:tooling, lang:mixed, scope:scripts`

---

## Scripts Inventory & Status

### Python Scripts

| Script | Shebang | Args Pattern | Help | Status |
|--------|---------|--------------|------|--------|
| `fixture_status.py` | ✅ `#!/usr/bin/env python3` | argparse | ✅ `-h` works | ✅ READY |
| `extract_fixture_from_dataset.py` | ✅ `#!/usr/bin/env python3` | positional | ⚠️ Custom usage | ✅ READY |
| `inspect_db.py` | ✅ `#!/usr/bin/env python3` | positional | ⚠️ Custom usage | ⚠️ BUG: crashes on modern schema |
| `create_minimal_db.py` | ✅ `#!/usr/bin/env python3` | positional | ✅ Error message | ✅ READY |
| `analyze_sample_routes.py` | ✅ `#!/usr/bin/env python3` | hardcoded | ❌ No help | ⚠️ Works but uses hardcoded paths |
| `extract_route_fixture.py` | ✅ `#!/usr/bin/env python3` | argparse | ✅ `-h` works | ✅ READY |

### Node.js Scripts (Already Integrated)

| Script | Integration |
|--------|-------------|
| `run-audit.js` | Via Nx crate targets |
| `outdated-report.js` | Via package.json |
| `check-pnpm-outdated.js` | Via package.json |
| `run-markdownlint-if-exists.js` | Via package.json |
| `run-precommit-nx.js` | Via husky |
| `run-prettier-if-exists.js` | Via package.json |

---

## Script Functionality Tests

### fixture_status.py
```
✅ status: Returns JSON with fixture metadata
✅ verify: Returns "Fixture metadata verified."
✅ --help: Shows argparse help
```

### inspect_db.py
```
⚠️ BUG: Script queries both SolarSystems AND mapSolarSystems
   - Modern e6c3 schema only has SolarSystems
   - Crashes with: sqlite3.OperationalError: no such table: mapSolarSystems
   - RECOMMENDATION: Fix script or document limitation
```

### extract_fixture_from_dataset.py
```
✅ Works with positional args: <source_db> <output_db>
✅ Shows helpful usage on error
⚠️ No --help flag support
```

---

## nx.json Configuration Analysis

### targetDefaults Applied
- `build`: cached, parallel=false, depends on ^build
- `test`: cached, parallel=false, depends on build
- `lint`: cached, parallel=false
- `clippy`: cached, parallel=false
- `audit`: NOT cached
- `outdated`: NOT cached

### namedInputs Available
- `default`: All project files + sharedGlobals
- `production`: default minus test/spec files and .md
- `sharedGlobals`: `.rust-toolchain`, `.nvmrc`

### Caching Strategy for Scripts
Based on existing patterns:
- **Cacheable**: `fixture-verify` (deterministic, read-only)
- **Not cacheable**: `fixture-sync`, `fixture-record`, `venv-setup` (mutating)
- **Not cacheable**: `inspect-db`, `analyze-routes` (ad-hoc queries)

---

## Consistency Recommendations

### 1. Project Configuration
```json
{
  "name": "scripts",
  "$schema": "../node_modules/nx/schemas/project-schema.json",
  "projectType": "application",  // Not library - scripts are executable
  "sourceRoot": "scripts",
  "tags": ["type:tooling", "lang:mixed", "scope:scripts"]
}
```

### 2. Schema Path
- Use `../node_modules/nx/schemas/project-schema.json` (matches crates pattern)
- NOT `../../node_modules/...` since scripts/ is at root level

### 3. Task Naming Convention
- Use kebab-case: `fixture-verify`, `venv-setup`
- Group related tasks with common prefix: `fixture-*`

### 4. cwd Pattern
- All tasks use `"cwd": "{workspaceRoot}"` (matches existing crates)

### 5. parallel Setting
- Align with existing: `parallel: false` for sequential execution
- Only needed if commands array used

---

## Issues Found

### Issue 1: inspect_db.py Schema Bug
**Severity**: Medium  
**Impact**: Script crashes on modern e6c3 databases  
**Fix**: Wrap mapSolarSystems query in try/except or check table existence

### Issue 2: Inconsistent Argument Handling
**Severity**: Low  
**Impact**: Some scripts use argparse, others use sys.argv  
**Recommendation**: Document in scripts/README.md, no code change needed

### Issue 3: analyze_sample_routes.py Hardcoded Paths
**Severity**: Low  
**Impact**: Works but relies on default file locations  
**Recommendation**: Make paths configurable via Nx args

---

## Pre-Implementation Checklist

- [x] Python 3.10+ available (3.13.5)
- [x] .gitignore configured for venv
- [x] Existing project.json patterns documented
- [x] nx.json targetDefaults reviewed
- [x] Script functionality tested
- [x] Bug in inspect_db.py identified
- [x] Tag conventions established

---

## Recommended Plan Adjustments

1. **Fix inspect_db.py** before or during implementation (add try/except)
2. **Use schema path** `../node_modules/nx/schemas/project-schema.json`
3. **Add scope tag** `scope:scripts` for consistency with lambda projects
4. **Test fixture-verify caching** after implementation

---

**Analysis Complete**: Ready to proceed with implementation
