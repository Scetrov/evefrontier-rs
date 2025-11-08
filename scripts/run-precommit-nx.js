#!/usr/bin/env node
const fs = require('fs');
const { spawnSync } = require('child_process');

function nxIsConfigured() {
  try {
    const cfg = JSON.parse(fs.readFileSync('nx.json', 'utf8'));
    return (
      cfg &&
      cfg.tasksRunnerOptions &&
      cfg.tasksRunnerOptions.default &&
      cfg.tasksRunnerOptions.default.runner
    );
  } catch (e) {
    return false;
  }
}

if (!nxIsConfigured()) {
  console.log('Nx runner not configured; skipping nx affected step.');
  process.exit(0);
}

// Run nx affected format then lint. Collect failures to report at the end.
const failures = [];
const run = (args) => {
  const cmd = `npx ${["nx", ...args].join(" ")}`;
  console.log(new Date().toISOString(), `running: ${cmd}`);
  // Clear NODE_OPTIONS for child processes to avoid inheriting --inspect flags
  const env = Object.assign({}, process.env, { NODE_OPTIONS: "" });
  // Use shell:true so 'npx' command resolution works on Windows (npx.cmd)
  // Add a timeout (5 minutes) to avoid infinite hangs; capture errors.
  const res = spawnSync(cmd, { stdio: "inherit", env, shell: true, timeout: 300000 });
  const code = res && typeof res.status === "number" ? res.status : res && res.error ? 1 : 0;
  if (code !== 0) {
    const errMsg = res && res.error ? ` error=${res.error && res.error.message}` : "";
    const sig = res && res.signal ? ` signal=${res.signal}` : "";
    console.error(`${cmd} exited with code ${code}${errMsg}${sig}`);
    failures.push({
      cmd,
      code,
      err: res && res.error ? String(res.error) : undefined,
      signal: res && res.signal
    });
  }
  return code;
};

// Run pnpm outdated check at root (node script) then run cargo audits across crates
run(['run', 'evefrontier-pathfinder:outdated']);
run(['run-many', '--target=audit', '--all']);

// Then run affected fmt and clippy as defined in project.json
run(['affected', '--target=fmt', '--base=main', '--head=HEAD']);
run(['affected', '--target=clippy', '--base=main', '--head=HEAD']);

if (failures.length > 0) {
  console.error("Precommit checks failed for the following commands:");
  failures.forEach((f) => console.error(`  - ${f.cmd} (exit ${f.code})`));
  process.exit(1);
}
process.exit(0);
