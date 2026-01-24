# evefrontier-rs Development Guidelines

Partially auto-generated from feature plans; manual additions allowed between MANUAL ADDITIONS markers. Last updated: 2025-12-29

## Active Technologies
- Markdown with Mermaid syntax (no Rust code changes required) + Mermaid (renders natively in GitHub, VS Code, most Markdown viewers) (006-architecture-diagrams)
- Rust 1.91.1 (per `.rust-toolchain`) + axum 0.8, tracing, tracing-subscriber (json feature), metrics, (008-microservices-observability)
- N/A (metrics are in-memory with Prometheus scraping) (008-microservices-observability)
- Rust 1.91.1 (per `.rust-toolchain`) + sha2 (checksum), postcard+zstd (serialization), clap (CLI) (009-spatial-index-freshness-ci)
- File-based (spatial index `.spatial.bin`, dataset `.db`) (009-spatial-index-freshness-ci)
- YAML, Markdown, TOML (configuration files only) + GitHub Actions, Helm, Docker/Podman (010-ghcr-container-repository-paths)
- Rust 1.91.1 + `rusqlite`, `serde`, `csv`, `thiserror`, `kiddo` (existing spatial index), `clap`, `aws_lambda_events` (015-ship-data-plan)
- SQLite dataset (`static_data.db`) plus cached `ship_data.csv` in `evefrontier_datasets/` (015-ship-data-plan)
- Rust 1.91.1 (per `.rust-toolchain`) + `rmcp` (official MCP SDK), `tokio`, `serde`, `schemars` (JSON Schema) (016-mcp-server-integration)
- SQLite database (bundled EVE Frontier dataset from `evefrontier_datasets` repo) (016-mcp-server-integration)
- Rust 1.91.1 (per `.rust-toolchain`) + flate2 (gzip), base64 (encoding) - both already transitive deps (017-fmap-url-support)
- N/A (stateless encoding/decoding) (017-fmap-url-support)
- Rust 1.91.1 + reqwest (blocking client, already used), zip, csv, serde, (015-ship-data-downloader)
- OS cache directory under `evefrontier_datasets/` (same as DB cache) (015-ship-data-downloader)
- Rust 1.91.1 (per `.rust-toolchain`) + clap (CLI parsing), serde/serde_json (JSON output), tracing (logging), evefrontier-lib (core logic) (025-scout-cli-subcommand)
- SQLite dataset (`static_data.db`) + spatial index (`.spatial.bin`) (025-scout-cli-subcommand)

- Documentation (Markdown) with shell command examples + GPG, cosign, cargo-sbom, sha256sum (005-release-documentation)

## Project Structure

```text
src/
tests/
```

## Commands

# Add commands for Documentation (Markdown) with shell command examples

## Code Style

Documentation (Markdown) with shell command examples: Follow standard conventions

## Recent Changes
- 025-scout-cli-subcommand: Added Rust 1.91.1 (per `.rust-toolchain`) + clap (CLI parsing), serde/serde_json (JSON output), tracing (logging), evefrontier-lib (core logic)
- 015-ship-data-downloader: Added Rust 1.91.1 + reqwest (blocking client, already used), zip, csv, serde,
- 017-fmap-url-support: Added Rust 1.91.1 (per `.rust-toolchain`) + flate2 (gzip), base64 (encoding) - both already transitive deps


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
