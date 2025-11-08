#!/usr/bin/env node
const { execSync } = require('child_process');

try {
  const out = execSync('pnpm outdated --depth 0 --json', {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();

  if (!out || out === '{}' || out === '[]') {
    console.log('No outdated dependencies found.');
    process.exit(0);
  }

  const parsed = JSON.parse(out);
  if (parsed && Object.keys(parsed).length > 0) {
    console.error('Outdated dependencies found:');
    console.error(JSON.stringify(parsed, null, 2));
    process.exit(1);
  }

  process.exit(0);
} catch (e) {
  if (e && e.stdout) {
    try {
      const parsed = JSON.parse(e.stdout.toString());
      if (parsed && Object.keys(parsed).length > 0) {
        console.error('Outdated dependencies found:');
        console.error(JSON.stringify(parsed, null, 2));
        process.exit(1);
      }
    } catch (err) {
      // fallthrough
    }
  }

  console.error('Failed to run `pnpm outdated`:', e && e.message ? e.message : e);
  process.exit(1);
}
