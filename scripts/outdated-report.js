#!/usr/bin/env node
const { spawnSync } = require("child_process");

// Run pnpm outdated and pretty-print the JSON if present
const res = spawnSync("pnpm", ["outdated", "--depth", "0", "--json"], {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"]
});

const stdout = res.stdout ? res.stdout.toString().trim() : "";
const stderr = res.stderr ? res.stderr.toString().trim() : "";

if (!stdout || stdout === "{}" || stdout === "[]") {
  console.log("No outdated dependencies found.");
  process.exit(0);
}

try {
  const parsed = JSON.parse(stdout || stderr);
  console.error("Outdated dependencies found:");
  console.error(JSON.stringify(parsed, null, 2));
  process.exit(1);
} catch (e) {
  console.error("pnpm outdated returned non-JSON output:");
  console.error(stdout || stderr);
  process.exit(res.status || 1);
}
