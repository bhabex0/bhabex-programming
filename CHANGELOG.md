# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-11-07

### Added - Core Language

- **Lexer**: Complete tokenizer with keywords, operators, literals, identifiers
  - Keywords: `fn`, `let`, `mut`, `const`, `struct`, `enum`, `trait`, `impl`, `type`, `if`, `else`, `match`, `for`, `while`, `loop`, `break`, `continue`, `return`, `async`, `await`, `spawn`, `try`, `catch`, `throw`, `import`, `export`, `pub`, `use`, `as`, `in`, `where`, `Self`, `self`, `super`, `crate`, `macro`, `macro_rules`, `unsafe`, `static`, `mod`, `module`, `move`, `defer`, `finally`, `extern`
  - Operators: `+`, `-`, `*`, `/`, `%`, `**`, `+=`, `-=`, `*=`, `/=`, `%=`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, `&`, `|`, `^`, `~`, `<<`, `>>`, `<<=`, `>>=`, `&=`, `|=`, `^=`, `=`, `->`, `=>`, `.`, `..`, `..=`, `...`, `,`, `;`, `:`, `::`, `?`
  - Delimiters: `(`, `)`, `{`, `}`, `[`, `]`, `@`, `#`, `$`
  - Literals: integers (decimal, hex, octal, binary), floats, strings, chars, raw strings, booleans, null
  - Comments: line (`//`), block (`/* */`), nested block comments

- **Parser**: Recursive descent parser with full expression precedence
  - Functions with params, return types, generics, async, extern, abi
  - Structs with fields, methods, visibility
  - Enums with tuple/struct variants, discriminants
  - Traits with bounds, associated items
  - Impl blocks with trait references
  - Type aliases, constants, statics
  - Modules, imports, exports
  - Control flow: if/else, while, for, loop, match, try/catch/finally
  - Expressions: binary/unary operators, calls, method calls, field access, indexing, closures, struct literals, arrays, tuples, ranges, paths
  - Patterns: literals, identifiers, wildcards, structs, tuples, enums, or-patterns, guards, references
  - Attributes: `#[name]`, `#[name(args)]`

- **AST**: Complete abstract syntax tree with spans
  - Programs, items, statements, expressions, patterns, types
  - Generics with bounds and defaults
  - Spans for error reporting

- **Interpreter**: Tree-walking interpreter
  - Functions, closures, async/await
  - Classes with inheritance, methods, properties
  - Structs, enums, pattern matching
  - Exception handling: try/catch/throw
  - Concurrency: spawn, channels (basic)
  - Modules and imports

### Added - Standard Library

- **Core**: `print`, `len`, `str`, `int`, `float`, `range`, `type`, `input`, `list`, `dict`, `assert`
- **File I/O**: `read_file`, `write_file`, `append_file`, `file_exists`, `is_file`, `is_dir`, `list_dir`, `mkdir`, `remove`, `rename`, `file_size`
- **JSON**: `json_parse`, `json_stringify`
- **HTTP**: `http_get`, `http_post`, `http_put`, `http_delete`, `http_patch`, `http_head`
- **Regex**: `regex_match`, `regex_findall`, `regex_replace`, `regex_split`
- **Math**: `sin`, `cos`, `tan`, `sqrt`, `pow`, `log`, `exp`, `floor`, `ceil`, `random`, `randint`, `pi`, `e`
- **Datetime**: `now`, `parse_datetime`, `timestamp`, `sleep`, `async_sleep`
- **OS**: `env`, `args`, `exit`, `cwd`, `chdir`, `platform`
- **SQLite**: `sqlite_open`, `sqlite_exec`, `sqlite_query`, `sqlite_close`
- **JSON Schema**: `json_schema_validate`, `json_schema_draft`

### Added - CLI Tool

- `bx run <file>` - Run Bhabex file
- `bx build <file>` - Build bytecode
- `bx build --native <file>` - Compile to native binary
- `bx check <file>` - Syntax/type check
- `bx fmt <file>` - Format code
- `bx tokenize <file>` - Show tokens
- `bx parse <file>` - Show AST
- `bx repl` - Interactive REPL
- `bx eval "code"` - Evaluate inline
- `bx init <project>` - Scaffold new project
- `bx test` - Run tests
- `bx pkg init/add/install/update/list/build/test/publish` - Package management
- `bx lsp` - Start LSP server
- `bx version` - Show version

### Added - Rust Components

- **bhabex_minimal**: Clean Rust port with lexer + parser + AST
  - 19 tests passing (9 lexer + 10 parser)
  - CLI: build, run, check, tokenize, parse
  - No LLVM dependency

### Added - Tooling & Infrastructure

- **VS Code Extension**: Syntax highlighting, snippets, IntelliSense, formatting, linting, debugging
- **Installers**: Windows (NSIS), macOS (pkgbuild), Linux (AppImage, .deb, .rpm), Android (Termux), iOS (XCFramework)
- **Documentation**: Tutorial, Language Spec, API Reference, CLI Reference
- **PyPI Publishing**: pyproject.toml, setup.cfg, MANIFEST.in, README.md, LICENSE
- **CI/CD**: GitHub Actions workflows for CI, Release, Rust builds
- **Dependabot**: Automated dependency updates for pip, cargo, npm
- **Code Owners**: CODEOWNERS file
- **Contributing Guide**: CONTRIBUTING.md
- **Makefile**: Development workflow automation

### Added - Testing

- **Python**: 9 lexer tests, integration tests for stdlib
- **Rust**: 19 tests for bhabex_minimal (9 lexer + 10 parser)

---

## [Unreleased]

### Planned

- Standard library expansion (async HTTP, crypto, database)
- Language features: actors, channels, traits, generics, macros
- JIT compiler for VM
- Native compilation backend (LLVM)
- Package registry
- Debugger support
- Language server improvements
- Cross-compilation fixes

---

## Version History

- **0.1.0** (2026-11-07): Initial release with complete interpreter, CLI, standard library, VS Code extension, installers, documentation, and CI/CD

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*