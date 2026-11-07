# Bhabex CLI Reference

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

---

## Global Options

```bash
bx [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help |
| `-v, --version` | Show version |
| `--color <when>` | Color output: auto, always, never |
| `--config <path>` | Config file path |

---

## Commands Overview

```
bx <COMMAND>
```

| Command | Description |
|---------|-------------|
| `run` | Run a .bx file |
| `build` | Build bytecode or native binary |
| `check` | Syntax/type check |
| `fmt` | Format source code |
| `tokenize` | Show tokens |
| `parse` | Show AST |
| `repl` | Interactive REPL |
| `eval` | Evaluate inline code |
| `init` | Scaffold new project |
| `test` | Run tests |
| `pkg` | Package management |
| `lsp` | Start LSP server |
| `completion` | Generate shell completions |

---

## `bx run`

Run a Bhabex file directly.

```bash
bx run <file.bx> [ARGS...]
```

### Options
| Option | Description |
|--------|-------------|
| `--release` | Run optimized build |
| `--native` | Compile to native and run |
| `--target <triple>` | Cross-compile target |

### Examples
```bash
bx run hello.bx
bx run main.bx -- arg1 arg2
bx run app.bx --release
bx run app.bx --native --target x86_64-pc-windows-msvc
```

---

## `bx build`

Build Bhabex project.

```bash
bx build [file.bx] [OPTIONS]
```

### Options
| Option | Description |
|--------|-------------|
| `-o, --output <path>` | Output file/directory |
| `--release` | Optimized build |
| `--native` | Compile to native binary |
| `--target <triple>` | Target triple (e.g., `x86_64-unknown-linux-gnu`) |
| `--lib` | Build as library |
| `--emit <type>` | Emit: bytecode, asm, llvm-ir, obj |
| `--features <list>` | Enable features |
| `--no-stdlib` | Don't link stdlib |

### Output Formats
| Flag | Description |
|------|-------------|
| `--emit bytecode` | `.bxc` bytecode (default) |
| `--emit asm` | Assembly (`.s`) |
| `--emit llvm-ir` | LLVM IR (`.ll`) |
| `--emit obj` | Object file (`.o`) |

### Examples
```bash
bx build main.bx
bx build main.bx --release -o app.bxc
bx build main.bx --native -o app
bx build main.bx --target aarch64-linux-android
bx build --lib src/lib.bx -o libbx.so
bx build main.bx --emit asm -o main.s
```

---

## `bx check`

Syntax and type check without building.

```bash
bx check <file.bx>
```

### Options
| Option | Description |
|--------|-------------|
| `--strict` | Enable strict warnings |
| `--no-type-check` | Only syntax check |

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Syntax error |
| 2 | Type error |

### Examples
```bash
bx check main.bx
bx check src/ --strict
```

---

## `bx fmt`

Format Bhabex source code.

```bash
bx fmt [file.bx] [OPTIONS]
```

### Options
| Option | Description |
|--------|-------------|
| `-w, --write` | Write changes to file |
| `--check` | Check formatting without writing |
| `--indent <n>` | Indent spaces (default: 4) |
| `--width <n>` | Max line width (default: 100) |

### Examples
```bash
bx fmt main.bx          # Show formatted output
bx fmt main.bx -w       # Format in place
bx fmt src/ -w          # Format all .bx files
bx fmt --check src/     # CI check
```

---

## `bx tokenize`

Show lexical tokens.

```bash
bx tokenize <file.bx>
```

### Output Format
```
TOKEN_TYPE 'lexeme' (line:column)
```

### Example
```bash
bx tokenize hello.bx
# FN 'fn' (1:1)
# IDENTIFIER 'main' (1:4)
# LPAREN '(' (1:8)
# ...
```

---

## `bx parse`

Show abstract syntax tree.

```bash
bx parse <file.bx>
```

### Example
```bash
bx parse hello.bx
# Program { items: [Function(Function { name: "main", ... })] }
```

---

## `bx repl`

Interactive Read-Eval-Print Loop.

```bash
bx repl
```

### REPL Commands
| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:quit` / `:exit` | Exit REPL |
| `:clear` | Clear buffer |
| `:load <file>` | Load and run file |
| `:type <expr>` | Show type of expression |
| `:time <expr>` | Time expression |

### Example Session
```
>>> let x = 42
>>> x * 2
84
>>> fn double(n) = n * 2
>>> double(21)
42
>>> :quit
Goodbye!
```

---

## `bx eval`

Evaluate inline code.

```bash
bx eval "code"
```

### Examples
```bash
bx eval "print(2 + 2)"
bx eval "let x = 10; x * x"
bx eval "fn main() { print('Hello') }"
```

---

## `bx init`

Scaffold new project.

```bash
bx init <name> [OPTIONS]
```

### Options
| Option | Description |
|--------|-------------|
| `--lib` | Create library project |
| `--bin` | Create binary project (default) |
| `--vcs <git\|none>` | Initialize git repo |
| `--path <dir>` | Target directory |

### Templates
```
my-app/
├── bxpkg.toml
├── main.bx
├── src/
│   └── lib.bx
├── tests/
│   └── test_main.bx
└── .gitignore
```

### Examples
```bash
bx init my-app
bx init my-lib --lib
bx init my-app --path ./projects
```

---

## `bx test`

Run tests.

```bash
bx test [OPTIONS] [FILTER]
```

### Options
| Option | Description |
|--------|-------------|
| `--release` | Test optimized build |
| `--native` | Test native binary |
| `--jobs <n>` | Parallel jobs |
| `--nocapture` | Show stdout/stderr |
| `--test-threads <n>` | Test threads |

### Filter Syntax
```bash
bx test                    # All tests
bx test my_test            # Tests containing "my_test"
bx test module::test_fn    # Specific test
bx test -- --skip slow     # Skip slow tests
```

### Test Attributes
```bx
#[test]
fn test_add() { assert_eq!(add(2, 3), 5) }

#[test]
#[should_panic]
fn test_divide_by_zero() { div(1, 0) }

#[integration]
fn test_database() { ... }

#[quickcheck]
fn test_reverse(xs: [i32]) -> bool { xs.reverse().reverse() == xs }
```

---

## `bx pkg`

Package management.

```bash
bx pkg <SUBCOMMAND> [OPTIONS]
```

### Subcommands

#### `bx pkg init`
```bash
bx pkg init [name]
```
Initializes `bxpkg.toml` in current directory.

#### `bx pkg add`
```bash
bx pkg add <name>[@version] [OPTIONS]
```
Adds dependency.

| Option | Description |
|--------|-------------|
| `--dev` | Add as dev-dependency |
| `--optional` | Mark as optional |
| `--target <triple>` | Target-specific dependency |

```bash
bx pkg add http@0.5
bx pkg add serde --dev
bx pkg add sqlite --target x86_64-unknown-linux-gnu
```

#### `bx pkg remove`
```bash
bx pkg remove <name>
```

#### `bx pkg install`
```bash
bx pkg install [OPTIONS]
```
Installs all dependencies.

| Option | Description |
|--------|-------------|
| `--locked` | Use exact versions from lock file |
| `--offline` | Use cached packages only |

#### `bx pkg update`
```bash
bx pkg update [name]
```
Updates dependencies.

#### `bx pkg list`
```bash
bx pkg list [OPTIONS]
```
| Option | Description |
|--------|-------------|
| `--tree` | Show dependency tree |
| `--outdated` | Show outdated packages |

#### `bx pkg build`
```bash
bx pkg build [OPTIONS]
```
Builds project with dependencies.

| Option | Description |
|--------|-------------|
| `--release` | Optimized build |
| `--target <triple>` | Cross-compile |

#### `bx pkg test`
```bash
bx pkg test [OPTIONS] [FILTER]
```

#### `bx pkg publish`
```bash
bx pkg publish [OPTIONS]
```
| Option | Description |
|--------|-------------|
| `--dry-run` | Validate without publishing |
| `--token <token>` | Registry token |
| `--registry <url>` | Custom registry |

```bash
bx pkg publish
bx pkg publish --dry-run
bx pkg publish --token $BHABEX_TOKEN
```

#### `bx pkg install-tool`
```bash
bx pkg install-tool <name>[@version]
```
Installs binary globally.

```bash
bx pkg install-tool bxfmt
bx pkg install-tool bxlsp@0.1.0
```

---

## `bx lsp`

Start Language Server Protocol server.

```bash
bx lsp [OPTIONS]
```

### Options
| Option | Description |
|--------|-------------|
| `--stdio` | Use stdio transport (default) |
| `--tcp <port>` | Use TCP on port |
| `--log <level>` | Log level |

### Client Configuration (VS Code)
```json
{
    "bhabex.path": "bx",
    "bhabex.lsp.enabled": true
}
```

---

## `bx completion`

Generate shell completions.

```bash
bx completion <shell>
```

### Supported Shells
- `bash`
- `zsh`
- `fish`
- `powershell`
- `elvish`

### Installation
```bash
# Bash
bx completion bash > /usr/share/bash-completion/completions/bx

# Zsh
bx completion zsh > ~/.zsh/completions/_bx

# Fish
bx completion fish > ~/.config/fish/completions/bx.fish

# PowerShell
bx completion powershell > bx.ps1
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `BHABEX_PATH` | Path to bx executable |
| `BHABEX_STDLIB` | Stdlib directory |
| `BHABEX_CACHE` | Cache directory |
| `BHABEX_TARGET` | Default target triple |
| `BHABEX_RUSTFLAGS` | Extra Rust compiler flags |
| `CARGO_HOME` | Cargo home (for native builds) |
| `RUSTUP_TOOLCHAIN` | Rust toolchain |

---

## Configuration Files

### `bxpkg.toml`
```toml
[package]
name = "my-app"
version = "1.0.0"
description = "A Bhabex application"
license = "MIT"
authors = ["Name <email>"]
repository = "https://github.com/user/repo"
edition = "2026"

[dependencies]
std = "1.0"
http = "0.5"

[dev-dependencies]
test = "1.0"

[build]
targets = ["native", "wasm"]
```

### `.bxfmt.toml`
```toml
indent = 4
width = 100
newline_style = "lf"
spaces_around_operators = true
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Syntax error |
| 4 | Type error |
| 5 | Compilation error |
| 6 | Link error |
| 7 | Runtime error |
| 8 | Test failure |
| 9 | Package error |
| 10 | IO error |

---

## Examples

### Development Workflow
```bash
# Create project
bx init my-app
cd my-app

# Add dependencies
bx pkg add http@0.5
bx pkg add serde@1.0

# Run
bx run main.bx

# Test
bx test

# Build release
bx build --release --native -o my-app

# Format
bx fmt -w .

# Check
bx check .
```

### Cross-compilation
```bash
# Android
bx build --target aarch64-linux-android --release

# iOS
bx build --target aarch64-apple-ios --release

# WebAssembly
bx build --target wasm32-unknown-unknown --release

# Windows from Linux
bx build --target x86_64-pc-windows-msvc --release
```

### CI Pipeline
```yaml
# .github/workflows/ci.yml
- name: Check syntax
  run: bx check .

- name: Format check
  run: bx fmt --check .

- name: Run tests
  run: bx test --release

- name: Build
  run: bx build --release --native
```

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*