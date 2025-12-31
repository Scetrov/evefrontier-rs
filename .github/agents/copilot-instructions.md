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
- 015-ship-data-plan: Added Rust 1.91.1 + `rusqlite`, `serde`, `csv`, `thiserror`, `kiddo` (existing spatial index), `clap`, `aws_lambda_events`
- 010-ghcr-container-repository-paths: Added YAML, Markdown, TOML (configuration files only) + GitHub Actions, Helm, Docker/Podman
- 009-spatial-index-freshness-ci: Added Rust 1.91.1 (per `.rust-toolchain`) + sha2 (checksum), postcard+zstd (serialization), clap (CLI)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
