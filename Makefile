.PHONY: help build build-release test test-smoke test-all clean install check fmt lint ci audit bench fixture-status fixture-verify fixture-record

# Default target
help:
	@echo "Available targets:"
	@echo "  make build          - Build debug binaries"
	@echo "  make build-release  - Build optimized release binaries"
	@echo "  make test           - Run all Rust tests (unit + integration)"
	@echo "  make test-smoke     - Quick CLI smoke test with release binary"
	@echo "  make test-all       - Run tests + smoke tests (comprehensive)"
	@echo "  make bench          - Run Criterion benchmarks"
	@echo "  make ci             - Run full CI checks locally (fmt, clippy, build, test)"
	@echo "  make audit          - Run security audit with cargo-audit"
	@echo "  make check          - Run clippy lints"
	@echo "  make fmt            - Format code with rustfmt"
	@echo "  make lint           - Run clippy with warnings as errors"
	@echo "  make install        - Install CLI to ~/.cargo/bin"
	@echo "  make clean          - Clean build artifacts"

# Build targets
build:
	cargo build --workspace

build-release:
	cargo build --release --workspace

# Test targets
test:
	cargo test --workspace

test-smoke: build-release
	@echo "=== Smoke Test: Running CLI with real e6c3 fixture ==="
	@echo "Systems: Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G"
	@echo ""
	@echo "1. Download command"
	@mkdir -p /tmp/evefrontier-smoke
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --data-dir /tmp/evefrontier-smoke download --no-logo --no-footer
	@echo ""
	@echo "2. Basic route (Nod → Brana)"
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --data-dir /tmp/evefrontier-smoke route \
		--from "Nod" --to "Brana" \
		--no-logo --no-footer
	@echo ""
	@echo "3. JSON output validation"
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --data-dir /tmp/evefrontier-smoke --format json route \
		--from "Nod" --to "Brana" \
		--no-logo | jq -e '.kind == "route"' > /dev/null || (echo "❌ JSON validation failed" && exit 1)
	@echo "   ✓ JSON output valid"
	@echo ""
	@echo "✅ Smoke tests passed! For comprehensive validation, run: make test"

bench:
	cargo bench -p evefrontier-lib

# Fixture helpers
fixture-status:
	python3 scripts/fixture_status.py status

fixture-verify:
	python3 scripts/fixture_status.py verify

fixture-record:
	python3 scripts/fixture_status.py record

# Code quality targets
check:
	cargo clippy --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace -- -D warnings

# Install target
install:
	cargo install --path crates/evefrontier-cli

# Comprehensive testing
test-all: test test-smoke
	@echo ""
	@echo "✅ All tests passed (unit, integration, and smoke tests)"

# CI simulation (matches pre-commit hook and CI workflow)
ci:
	@echo "=== Running full CI checks ==="
	@echo ""
	@echo "1️⃣  Formatting check..."
	cargo fmt --all -- --check
	@echo "   ✅ Format OK"
	@echo ""
	@echo "2️⃣  Clippy (lints)..."
	cargo clippy --workspace --all-targets -- -D warnings
	@echo "   ✅ Clippy OK"
	@echo ""
	@echo "3️⃣  Build..."
	cargo build --workspace --all-targets
	@echo "   ✅ Build OK"
	@echo ""
	@echo "4️⃣  Tests..."
	cargo test --workspace
	@echo "   ✅ Tests OK"
	@echo ""
	@echo "✅ All CI checks passed!"

# Clean target
clean:
	cargo clean

# Security audit
audit:
	cargo audit