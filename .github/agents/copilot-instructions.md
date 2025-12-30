# evefrontier-rs Development Guidelines

Partially auto-generated from feature plans; manual additions allowed between MANUAL ADDITIONS markers. Last updated: 2025-12-29

## Active Technologies
- Markdown with Mermaid syntax (no Rust code changes required) + Mermaid (renders natively in GitHub, VS Code, most Markdown viewers) (006-architecture-diagrams)
- Rust 1.91.1 (per `.rust-toolchain`) + axum 0.8, tracing, tracing-subscriber (json feature), metrics, (008-microservices-observability)
- N/A (metrics are in-memory with Prometheus scraping) (008-microservices-observability)
- Rust 1.91.1 (per `.rust-toolchain`) + sha2 (checksum), postcard+zstd (serialization), clap (CLI) (009-spatial-index-freshness-ci)
- File-based (spatial index `.spatial.bin`, dataset `.db`) (009-spatial-index-freshness-ci)

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
- 009-spatial-index-freshness-ci: Added Rust 1.91.1 (per `.rust-toolchain`) + sha2 (checksum), postcard+zstd (serialization), clap (CLI)
- 008-microservices-observability: Added Rust 1.91.1 (per `.rust-toolchain`) + axum 0.8, tracing, tracing-subscriber (json feature), metrics,
- 006-architecture-diagrams: Added Markdown with Mermaid syntax (no Rust code changes required) + Mermaid (renders natively in GitHub, VS Code, most Markdown viewers)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
