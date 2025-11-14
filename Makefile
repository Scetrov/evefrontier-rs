.PHONY: help build build-release test test-smoke clean install check fmt lint

# Default target
help:
	@echo "Available targets:"
	@echo "  make build          - Build debug binaries"
	@echo "  make build-release  - Build optimized release binaries"
	@echo "  make test           - Run all tests"
	@echo "  make test-smoke     - Run quick smoke tests"
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
	@echo "=== Smoke Test 1: Download command ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli download --no-logo --no-footer
	@echo ""
	@echo "=== Smoke Test 2: Basic route (text format) ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli route \
		--from "Y:170N" --to "BetaTest" \
		--no-logo --no-footer
	@echo ""
	@echo "=== Smoke Test 3: Route with emoji format ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --format emoji route \
		--from "Y:170N" --to "AlphaTest" \
		--no-logo --no-footer
	@echo ""
	@echo "=== Smoke Test 4: Route with notepad format ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --format note route \
		--from "Y:170N" --to "BetaTest" \
		--no-logo --no-footer
	@echo ""
	@echo "=== Smoke Test 5: Route with Dijkstra algorithm ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli route \
		--from "Y:170N" --to "BetaTest" \
		--algorithm dijkstra \
		--no-logo --no-footer
	@echo ""
	@echo "=== Smoke Test 6: JSON output ==="
	EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
	./target/release/evefrontier-cli --format json route \
		--from "Y:170N" --to "BetaTest" \
		--no-logo | head -20
	@echo ""
	@echo "✅ All smoke tests passed!"

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

# Clean target
clean:
	cargo clean
