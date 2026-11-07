# Contributing to Bhabex

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

Thank you for your interest in contributing to Bhabex! This document provides guidelines for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Making Changes](#making-changes)
5. [Testing](#testing)
6. [Code Style](#code-style)
7. [Pull Request Process](#pull-request-process)
8. [Architecture Overview](#architecture-overview)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code.

---

## Getting Started

### Prerequisites

- **Python 3.10+** (for interpreter)
- **Rust 1.75+** (for compiler components)
- **Node.js 18+** (for VS Code extension)
- **LLVM 18.1.8** (for native compilation)

### Fork and Clone

```bash
git clone https://github.com/bxLabs/bhabex.git
cd bhabex
git remote add upstream https://github.com/bxLabs/bhabex.git
```

---

## Development Setup

### Python Interpreter

```bash
cd bhabex
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -e ".[dev]"
```

### Rust Components

```bash
# bhabex_minimal
cd bhabex_minimal
cargo build --release
cargo test

# velo-compiler (requires LLVM 18.1.8)
cd ../velo-compiler
cargo build --release

# compiler
cd ../compiler
cargo build --release

# pkg (package manager)
cd ../pkg
cargo build --release
```

### VS Code Extension

```bash
cd vscode
npm install
npm run compile
```

---

## Making Changes

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring
- `test/description` - Test additions
- `chore/description` - Maintenance tasks

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`

Examples:
```
feat(lexer): add support for raw string literals
fix(parser): handle nested match expressions correctly
docs(api): add HTTP client documentation
test(interpreter): add async/await tests
```

---

## Testing

### Python Tests

```bash
cd bhabex
pytest tests/ -v
pytest tests/ -k "not slow"  # Skip slow tests
pytest tests/ --cov=bhabex  # With coverage
```

### Rust Tests

```bash
cd bhabex_minimal
cargo test

cd ../velo-compiler
cargo test

cd ../compiler
cargo test

cd ../pkg
cargo test
```

### Run All Tests

```bash
# From root
make test
# Or manually:
cd bhabex && pytest
cd ../bhabex_minimal && cargo test
cd ../pkg && cargo test
```

### Specific Test Examples

```bash
# Python
pytest tests/test_lexer.py -v
pytest tests/ -k "test_parse" -v

# Rust
cargo test test_parse_function
cargo test -- --nocapture  # Show println output
```

---

## Code Style

### Python

- **Formatter**: `black` (line length 100)
- **Linter**: `ruff` (with custom rules)
- **Type checker**: `mypy` (strict mode)

```bash
ruff check bhabex/
black --check bhabex/
mypy bhabex/
```

### Rust

- **Formatter**: `rustfmt`
- **Linter**: `clippy` (with `-D warnings`)

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

### TypeScript (VS Code Extension)

```bash
cd vscode
npm run lint
npm run format
```

---

## Pull Request Process

### Before Submitting

1. ✅ All tests pass
2. ✅ Code formatted (`black`, `cargo fmt`, `prettier`)
3. ✅ No linting errors (`ruff`, `cargo clippy`, `eslint`)
4. ✅ Type checking passes (`mypy`, `tsc`)
5. ✅ Commit messages follow conventions
6. ✅ Branch is up to date with `main`

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Refactoring
- [ ] Performance improvement
- [ ] Test addition

## Testing
- [ ] All existing tests pass
- [ ] New tests added for changes
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
```

### Review Process

1. Automated checks run (CI/CD)
2. Code owners review
3. At least 1 approval required
4. All conversations resolved
5. Branch up to date with main
5. Squash and merge

---

## Architecture Overview

### Python Interpreter (`bhabex/`)

```
bhabex/
├── main.py           # CLI entry point
├── lexer.py          # Tokenizer (logos-based)
├── parser.py         # Recursive descent parser
├── interpreter.py    # Bytecode VM
├── builtins.py       # Standard library
├── ast_nodes.py      # AST definitions
├── environment.py    # Variable scoping
├── errors.py         # Error types
├── tokens.py         # Token definitions
├── formatter.py      # Code formatter
├── lsp.py            # Language server
└── repl.py           # REPL
```

### Rust Components

- **bhabex_minimal**: Clean Rust port (lexer + parser + AST)
- **velo-compiler**: Full compiler with LLVM backend
- **compiler**: Bytecode compiler + native backend
- **pkg**: Package manager (`bxpkg`)

### Key Design Principles

1. **Simplicity First** - Prefer simple solutions over clever ones
2. **Explicit Over Implicit** - Clear code over magic
3. **Fail Fast** - Validate early, error with context
4. **Performance Matters** - Profile before optimizing
5. **Security by Default** - Safe defaults, opt-in danger

---

## Areas for Contribution

### High Priority

- [ ] Standard library expansion
- [ ] Error message improvements
- [ ] Performance optimization
- [ ] Test coverage increase
- [ ] Documentation completeness

### Medium Priority

- [ ] VS Code extension features
- [ ] Language server improvements
- [ ] Debugger support
- [ ] Package registry
- [ ] Cross-compilation fixes

### Good First Issues

Look for labels: `good first issue`, `help wanted`, `documentation`

---

## Getting Help

- **Discord**: https://discord.gg/bhabex
- **GitHub Discussions**: https://github.com/bxLabs/bhabex/discussions
- **Issues**: https://github.com/bxLabs/bhabex/issues

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*