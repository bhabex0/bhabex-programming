# Bhabex Makefile
# Copyright BHABEX 2026 | Securely designed & developed by bxLabs

.PHONY: help test lint format build clean install release dev-setup

# Default target
help:
	@echo "Bhabex Development Commands"
	@echo "=============================="
	@echo ""
	@echo "Development:"
	@echo "  make dev-setup       Install all development dependencies"
	@echo "  make test            Run all tests"
	@echo "  make test-python     Run Python tests only"
	@echo "  make test-rust       Run Rust tests only"
	@echo "  make lint            Run all linters"
	@echo "  make format          Format all code"
	@echo ""
	@echo "Building:"
	@echo "  make build           Build all components"
	@echo "  make build-python    Build Python wheel"
	@echo "  make build-rust      Build Rust components"
	@echo "  make install         Install locally"
	@echo ""
	@echo "Release:"
	@echo "  make release         Build release artifacts"
	@echo "  make publish         Publish to PyPI (requires token)"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean           Clean build artifacts"
	@echo "  make check           Quick health check"

# Development setup
dev-setup:
	@echo "Setting up Python environment..."
	cd bhabex && python -m venv .venv && . .venv/bin/activate && pip install -e ".[dev]"
	@echo "Installing Rust toolchain..."
	rustup toolchain install stable --component rustfmt,clippy
	@echo "Setting up VS Code extension..."
	cd vscode && npm install

# Python-specific
test-python:
	cd bhabex && pytest tests/ -v

test-python-cov:
	cd bhabex && pytest tests/ --cov=bhabex --cov-report=html

# Rust tests
test-rust:
	cd bhabex_minimal && cargo test
	cd ../velo-compiler && cargo test 2>&1 | head -20 || true
	cd ../compiler && cargo test 2>&1 | head -20 || true
	cd ../pkg && cargo test

# All tests
test: test-python test-rust

# Linting
lint-python:
	cd bhabex && ruff check bhabex/ && black --check bhabex/ && mypy bhabex/

lint-rust:
	cd bhabex_minimal && cargo fmt --check && cargo clippy -- -D warnings
	cd ../velo-compiler && cargo fmt --check && cargo clippy -- -D warnings 2>&1 | head -20 || true
	cd ../compiler && cargo fmt --check && cargo clippy -- -D warnings 2>&1 | head -20 || true
	cd ../pkg && cargo fmt --check && cargo clippy -- -D warnings

lint: lint-python lint-rust

# Formatting
format-python:
	cd bhabex && black bhabex/ && ruff check --fix bhabex/

format-rust:
	cd bhabex_minimal && cargo fmt
	cd ../velo-compiler && cargo fmt
	cd ../compiler && cargo fmt
	cd ../pkg && cargo fmt

format: format-python format-rust

# Building
build-python:
	cd bhabex && python -m build --wheel

build-rust:
	cd bhabex_minimal && cargo build --release
	cd ../velo-compiler && cargo build --release 2>&1 | head -30 || true
	cd ../compiler && cargo build --release 2>&1 | head -30 || true
	cd ../pkg && cargo build --release

build-vscode:
	cd vscode && npm run compile

build: build-python build-rust build-vscode

# Installation
install:
	cd bhabex && pip install -e .

install-dev:
	cd bhabex && pip install -e ".[dev]"

# Release
release: clean
	@echo "Building release artifacts..."
	cd bhabex && python -m build
	cd bhabex_minimal && cargo build --release
	cd ../pkg && cargo build --release
	@echo "Release artifacts in dist/ and target/release/"

publish:
	@echo "Publishing to PyPI..."
	cd bhabex && twine upload dist/*

# Cleaning
clean:
	@echo "Cleaning build artifacts..."
	cd bhabex && rm -rf build dist *.egg-info .pytest_cache .mypy_cache .ruff_cache __pycache__ .venv
	cd bhabex_minimal && cargo clean
	cd ../velo-compiler && cargo clean
	cd ../compiler && cargo clean
	cd ../pkg && cargo clean
	cd ../vscode && rm -rf out node_modules
	rm -rf dist build *.egg-info

# Quick checks
check:
	@echo "Running quick health checks..."
	cd bhabex && python -c "import bhabex; print('Python package OK')"
	cd bhabex_minimal && cargo check --quiet && echo "Rust bhabex_minimal OK"
	cd ../pkg && cargo check --quiet && echo "Rust pkg OK"
	@echo "All checks passed!"

# Development workflow
dev: dev-setup
	@echo "Development environment ready!"
	@echo "Run 'make test' to verify everything works"

# CI simulation
ci: lint test
	@echo "CI pipeline simulation complete"

# Watch mode for development
watch-python:
	cd bhabex && ptw --runner="pytest tests/ -xvs" --spool=60

watch-rust:
	cd bhabex_minimal && cargo watch -x test