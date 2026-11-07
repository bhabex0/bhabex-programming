# Bhabex

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

---

Bhabex is a modern, high-performance, multi-paradigm programming language designed for speed, safety, and expressiveness. It combines the simplicity of Python with the performance of C++/Rust.

## Features

- **Fast**: Bytecode VM with JIT + native compilation via Rust/LLVM
- **Safe**: Memory safety, capability-based security, no null pointers
- **Expressive**: Pattern matching, algebraic data types, generics, traits
- **Concurrent**: Actors, channels, async/await, structured concurrency
- **Cross-platform**: Windows, macOS, Linux, Android, iOS, WebAssembly
- **Zero dependencies**: Built from scratch, minimal runtime

## Quick Start

### Installation

```bash
# Windows
choco install bhabex

# macOS
brew install bhabex

# Linux
wget https://bhabex.dev/downloads/bhabex_0.1.0_amd64.deb
sudo dpkg -i bhabex_0.1.0_amd64.deb

# Python package (for development)
pip install bhabex
```

### Hello World

```bx
fn main() {
    print("Hello, Bhabex!")
}
```

```bash
bx run hello.bx
# Hello, Bhabex!
```

## Documentation

- [Tutorial](docs/TUTORIAL.md) - Step-by-step guide
- [Language Specification](docs/LANGUAGE_SPEC.md) - Complete language reference
- [API Reference](docs/API_REFERENCE.md) - Standard library API
- [CLI Reference](docs/CLI_REFERENCE.md) - Command-line tools

## Examples

```bash
# Fibonacci
bx run examples/fibonacci.bx

# Factorial
bx run examples/factorial.bx

# Classes and methods
bx run examples/counter.bx
```

## CLI Commands

```bash
bx run file.bx           # Run a file
bx build file.bx         # Build bytecode
bx build --native file.bx # Compile to native binary
bx check file.bx         # Syntax check
bx fmt file.bx           # Format code
bx repl                  # Interactive REPL
bx eval "print(2+2)"     # Evaluate inline
bx init my-project       # Create new project
bx test                  # Run tests
bx pkg add http@0.5      # Add dependency
```

## Project Structure

```
bhabex/
├── bhabex/              # Python interpreter package
│   ├── main.py          # CLI entry point
│   ├── lexer.py         # Tokenizer
│   ├── parser.py        # Parser
│   ├── interpreter.py   # Bytecode VM
│   ├── builtins.py      # Standard library
│   ├── ast_nodes.py     # AST definitions
│   ├── environment.py   # Variable environment
│   ├── errors.py        # Error types
│   ├── tokens.py        # Token definitions
│   ├── formatter.py     # Code formatter
│   ├── lsp.py           # Language server
│   └── repl.py          # REPL
├── compiler/            # Rust native compiler
├── installer/           # Platform installers
├── pkg/                 # Package manager
├── docs/                # Documentation
├── examples/            # Example programs
├── tests/               # Test suite
├── vscode/              # VS Code extension
├── pyproject.toml       # Python package config
└── setup.cfg            # Setuptools config
```

## Development

```bash
# Clone
git clone https://github.com/bxLabs/bhabex
cd bhabex

# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/

# Format code
ruff check .
black .

# Type check
mypy bhabex/
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Community

- Website: https://bhabex.dev
- GitHub: https://github.com/bxLabs/bhabex
- Discord: https://discord.gg/bhabex
- Twitter: @bhabex_lang

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*