#!/usr/bin/env node
/**
 * ADR Validation Script
 * Rules:
 *  - New ADR filenames must match /^\d{4}-[a-z0-9-]+\.md$/
 *  - Historical ADRs (present on origin/main) must not be modified, renamed or deleted unless PR has 'adr-override' label.
 *  - Header first line must match `# ADR <ID>:` where <ID> matches filename prefix.
 *  - TEMPLATE.md is exempt from immutability & naming regex.
 *  - Renames treated as modification + delete => both blocked unless override.
 */

const { execSync } = require('node:child_process');
const { readFileSync, existsSync } = require('node:fs');
const path = require('node:path');

function sh(cmd) {
  return execSync(cmd, { encoding: 'utf8' }).trim();
}

function safeSh(cmd) {
  try { return sh(cmd); } catch (e) { return ''; }
}

function main() {
  // Ensure we have origin/main
  safeSh('git fetch origin main --depth=1');

  const override = process.env.ADR_OVERRIDE_LABEL_PRESENT === 'true';
  const adrDir = 'docs/adrs';
  const adrRegex = /^\d{4}-[a-z0-9-]+\.md$/;
  const errors = [];

  // Baseline ADR files from main
  const baselineListRaw = safeSh(`git ls-tree -r origin/main --name-only ${adrDir}`);
  const baselineFiles = new Set(baselineListRaw.split('\n').filter(Boolean));

  // Diff status between origin/main and HEAD limited to adrs dir
  const diffRaw = safeSh(`git diff --name-status origin/main...HEAD -- ${adrDir}`);
  if (!diffRaw) {
    console.log('[ADR Guard] No ADR changes detected.');
    process.exit(0);
  }

  console.log('[ADR Guard] Evaluating changes:\n' + diffRaw);

  const lines = diffRaw.split('\n').filter(Boolean);

  for (const line of lines) {
    // Handle rename lines: R100\told\tnew or R\d+ similar
    const parts = line.split('\t');
    const status = parts[0];
    if (status.startsWith('R')) {
      const oldPath = parts[1];
      const newPath = parts[2];
      const oldName = path.basename(oldPath);
      const newName = path.basename(newPath);
      const oldIsAdr = adrRegex.test(oldName);
      const newIsAdr = adrRegex.test(newName);
      if (oldIsAdr || newIsAdr) {
        if (!override) {
          errors.push(`Rename detected '${oldName}' -> '${newName}' (status ${status}) is not allowed without 'adr-override' label.`);
        } else {
          validateNewFile(newPath, newName, true);
        }
      }
      continue;
    }

    const statusCode = status;
    const filePath = parts[1];
    const fileName = path.basename(filePath);

    const isTemplate = fileName === 'TEMPLATE.md';
    const isAdr = adrRegex.test(fileName);

    if (statusCode === 'A') {
      if (!isTemplate && !isAdr) {
        errors.push(`Added file '${fileName}' does not match ADR filename pattern.`);
        continue;
      }
      if (isAdr) {
        validateNewFile(filePath, fileName, false);
      }
      continue;
    }

    if (statusCode === 'M') {
      if (isTemplate) {
        // Allowed modifications to template
        continue;
      }
      if (isAdr) {
        if (!override) {
          errors.push(`Modification to historical ADR '${fileName}' is not allowed without 'adr-override' label.`);
        } else {
          // If override, still validate header consistency
          validateHeader(filePath, fileName, errors);
        }
      }
      continue;
    }

    if (statusCode === 'D') {
      if (isAdr && !override) {
        errors.push(`Deletion of ADR '${fileName}' is not allowed without 'adr-override' label.`);
      }
      continue;
    }

    // Other statuses (e.g., copies) treat conservatively
    if (isAdr && !override) {
      errors.push(`Change '${line}' for ADR '${fileName}' not permitted without override.`);
    }
  }

  function validateNewFile(filePath, fileName, fromRename) {
    if (!existsSync(filePath)) {
      errors.push(`File '${filePath}' not found in workspace after addition; checkout depth may be insufficient.`);
      return;
    }
    validateHeader(filePath, fileName, errors);
    // Additional future checks (e.g., uniqueness of ID) could go here.
    if (fromRename && !override) {
      errors.push(`Renamed ADR '${fileName}' requires override label though rename already flagged.`);
    }
  }

  function validateHeader(filePath, fileName, errorsArr) {
    try {
      const content = readFileSync(filePath, 'utf8');
      const firstLine = content.split(/\r?\n/, 1)[0];
      const idPrefix = fileName.slice(0, 4);
      const headerRe = new RegExp(`^#\\s*ADR\\s+${idPrefix}:`);
      if (!headerRe.test(firstLine)) {
        errorsArr.push(`Header mismatch in '${fileName}': first line must start with '# ADR ${idPrefix}:' (found: '${firstLine}').`);
      }
    } catch (e) {
      errorsArr.push(`Failed reading '${fileName}' for header validation: ${e.message}`);
    }
  }

  if (errors.length) {
    console.error('\n[ADR Guard] Validation FAILED with ' + errors.length + ' issue(s):');
    for (const err of errors) console.error(' - ' + err);
    console.error('\nTo override (e.g., for intentional historical ADR edit), add the PR label: adr-override');
    process.exit(1);
  }

  console.log('[ADR Guard] All checks passed.');
}

main();
