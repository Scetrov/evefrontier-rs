#!/usr/bin/env node
const { spawnSync } = require("child_process");
const { mkdirSync, writeFileSync } = require("fs");
const { join } = require("path");

function usage() {
  console.error("Usage: node run-audit.js <projectName>");
  process.exit(2);
}

const args = process.argv.slice(2);
if (args.length < 1) usage();
const projectName = args[0];

const outDir = join(process.cwd(), ".cache", "audit");
mkdirSync(outDir, { recursive: true });
const outFile = join(outDir, `${projectName}.json`);

console.log(`Running cargo audit for project ${projectName} (output -> ${outFile})`);

const res = spawnSync("cargo", ["audit", "--json"], {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"]
});
let stdout = "";
if (res.stdout) stdout = res.stdout.toString();
let stderr = "";
if (res.stderr) stderr = res.stderr.toString();

// Write whatever we captured so the result can be inspected and cached
try {
  writeFileSync(outFile, stdout || stderr || "", { encoding: "utf8" });
} catch (e) {
  console.error("Failed to write audit output file:", e && e.message ? e.message : e);
}

if (res.error) {
  console.error(
    "Failed to run cargo audit:",
    res.error && res.error.message ? res.error.message : res.error
  );
  process.exit(1);
}

// Exit with the same code cargo-audit returned (0 when no vulnerabilities, non-zero otherwise)
process.exit(typeof res.status === "number" ? res.status : 0);
