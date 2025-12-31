# Quickstart — Ship Data & Fuel Calculations

## Prerequisites
- Rust 1.91.1 (from `.rust-toolchain`)
- Dataset cache writeable (default OS cache dir)
- Fixture DB available at `docs/fixtures/minimal_static_data.db`

## Build & Download
```bash
pnpm nx run evefrontier-lib:build
pnpm nx run evefrontier-cli:build
# Download dataset + ship_data.csv to cache
cargo run -p evefrontier-cli -- download
```

## List Ships
```bash
cargo run -p evefrontier-cli -- --list-ships
```
Expected: table of ship names, base mass, fuel/cargo capacity from cached `ship_data.csv`.

## Plan a Route with Fuel Projection
```bash
cargo run -p evefrontier-cli -- route "Nod" "Brana" \
  --ship "Reflex" \
  --fuel-quality 10 \
  --cargo-mass 5000 \
  --fuel-load 1750 \
  --dynamic-mass
```
Expected: route output includes per-hop `fuel` section (cost, cumulative, remaining) and summary fuel totals; warnings if fuel insufficient.

## Lambda Invocation (local example)
```bash
# Assuming binary or bootstrap built
aws lambda invoke --function-name evefrontier-route \
  --payload '{"from":"Nod","to":"Brana","ship":"Reflex","fuel_quality":10,"cargo_mass":5000,"fuel_load":1750,"dynamic_mass":true}' \
  response.json
cat response.json
```
Expected: JSON includes `fuel_projection` per step and `route_summary.fuel` totals; backwards-compatible when ship omitted.

## Tests
```bash
pnpm nx run-many --targets=test --projects=evefrontier-lib,evefrontier-cli,evefrontier-lambda-route
```
- Unit tests: ship catalog parsing, fuel calculators (static/dynamic)
- Integration: CLI route output with fixture, Lambda handler response shape

## Regenerate Fixtures (when dataset updates)
- Add/update `ship_data.csv` fixture under `docs/fixtures/` with checksum marker
- Re-run ship parsing/fuel calculation tests
