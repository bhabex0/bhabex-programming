# Bhabex Language Specification v1.0

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

---

## 1. Overview

Bhabex (.bx) is a modern, high-performance, multi-paradigm programming language designed for speed, safety, and expressiveness. It combines the simplicity of Python with the performance of C++/Rust.

### Design Goals
- **Fastest**: Bytecode VM + native compilation (Rust backend)
- **Shortest syntax**: Minimal boilerplate, maximum expressiveness
- **Most powerful**: Systems programming to web development
- **Cross-platform**: Windows, macOS, Linux, Android, iOS, WebAssembly
- **Secure by default**: Memory safety, capability-based security
- **Zero external dependencies**: Built from scratch

### File Extension
- Source files: `.bx`
- Bytecode files: `.bxc`
- Native binaries: Platform-specific (`.exe`, no extension, `.apk`, `.ipa`)

---

## 2. Syntax Design

### 2.1 Hello World
```bx
print("Hello, World!")
```

### 2.2 Variables & Types
```bx
# Type inference
x = 42           # i64
y = 3.14         # f64
name = "bhabex"  # string
flag = true      # bool

# Explicit types
age: i32 = 25
pi: f64 = 3.14159
items: [i32] = [1, 2, 3]
user: {string: any} = {"name": "bx", "version": 1}
```

### 2.3 Functions
```bx
# Simple function
fn add(a: i32, b: i32) -> i32:
    return a + b

# Expression body (implicit return)
fn mul(a: i32, b: i32) -> i32: a * b

# Async function
async fn fetch(url: string) -> Result<string, Error>:
    http.get(url)

# Generic function
fn first[T](items: [T]) -> T:
    items[0]
```

### 2.4 Control Flow
```bx
# If expressions (returns value)
result = if x > 0: "positive" else: "negative"

# Pattern matching
match value:
    0 -> "zero"
    n if n > 0 -> "positive"
    _ -> "negative"

# Loops
for item in items:
    print(item)

while condition:
    do_work()

# Range loops
for i in 0..10:
    print(i)
for i in 0..=10:  # inclusive
    print(i)
```

### 2.5 Data Structures
```bx
# Structs
struct User:
    name: string
    age: i32
    email: string?

# Enums
enum Result[T, E]:
    Ok(T)
    Err(E)

enum Option[T]:
    Some(T)
    None

# Usage
user = User("bxLabs", 1, "contact@bhabex.dev")
result = Result.Ok(42)
opt = Option.Some("value")
```

### 2.6 Error Handling
```bx
# Try-catch with pattern matching
try:
    risky_operation()
catch Error as e:
    log(e)
    return default_value

# Result type propagation
fn read_file(path: string) -> Result<string, IOError>:
    file.open(path)?
    file.read()?
```

### 2.7 Concurrency
```bx
# Actors (lightweight threads)
actor Counter:
    var count = 0
    fn increment(): count += 1
    fn get() -> i32: count

# Channels
ch = chan[i32]()
spawn || ch.send(42)
value = ch.recv()

# Async/await
async fn main():
    results = await all([
        fetch("/api/1"),
        fetch("/api/2"),
        fetch("/api/3")
    ])
```

### 2.8 Modules & Imports
```bx
# Import standard library
import std.io
import std.net.http as http
import std.collections.{HashMap, HashSet}

# Import local modules
import ./utils
import ../core as core

# Re-export
pub export fn helper(): ...
```

### 2.9 Memory Management
```bx
# Ownership (Rust-like)
fn take_ownership(data: Box<i32>):
    print(*data)

# Borrowing
fn borrow(data: &i32):  # immutable borrow
    print(*data)

fn borrow_mut(data: &mut i32):  # mutable borrow
    *data += 1

# Reference counting
shared = Rc<String>("shared")
cloned = shared.clone()

# Arena allocation (for performance)
arena = Arena::new()
obj = arena.alloc(MyStruct { ... })
```

### 2.10 Metaprogramming
```bx
# Compile-time execution
const MAX_SIZE = compute_max_size()

# Macros
macro println!($msg: expr):
    std.io.stdout.write($msg + "\n")

# Attributes
@inline
@no_mangle
@export
fn public_api(): ...
```

---

## 3. Type System

### 3.1 Primitive Types
| Type | Size | Description |
|------|------|-------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | 8-128 bits | Signed integers |
| `u8`, `u16`, `u32`, `u64`, `u128` | 8-128 bits | Unsigned integers |
| `isize`, `usize` | Pointer size | Platform-dependent |
| `f32`, `f64` | 32/64 bits | IEEE 754 floats |
| `bool` | 1 byte | `true` / `false` |
| `char` | 4 bytes | Unicode scalar value |
| `string` | Variable | UTF-8 owned string |
| `&str` | 16 bytes | String slice |

### 3.2 Composite Types
- **Tuples**: `(i32, string, bool)`
- **Arrays**: `[T; N]` (fixed size)
- **Slices**: `[T]` (dynamic view)
- **Vectors**: `Vec<T>` (heap-allocated)
- **Maps**: `HashMap<K, V>`
- **Sets**: `HashSet<T>`

### 3.3 Generics
```bx
fn map[T, U](items: [T], f: fn(T) -> U) -> [U]:
    items.iter().map(f).collect()

struct Container[T]:
    value: T
    metadata: HashMap<string, string>
```

### 3.4 Traits (Interfaces)
```bx
trait Display:
    fn fmt(&self) -> string

trait Iterator[T]:
    fn next(&mut self) -> Option[T]
    fn collect[C: FromIterator[T]](self) -> C

impl Display for User:
    fn fmt(&self) -> string:
        "User(" + self.name + ")"
```

---

## 4. Bytecode VM Specification

### 4.1 Instruction Set
```
# Stack manipulation
PUSH_CONST    # Push constant onto stack
POP           # Pop top of stack
DUP           # Duplicate top
SWAP          # Swap top two

# Arithmetic
ADD, SUB, MUL, DIV, MOD
NEG           # Negate

# Comparison
EQ, NE, LT, GT, LE, GE

# Control flow
JUMP offset
JUMP_IF_FALSE offset
JUMP_IF_TRUE offset
CALL fn_index argc
RETURN
TAIL_CALL fn_index argc

# Variables
LOAD_LOCAL idx
STORE_LOCAL idx
LOAD_GLOBAL name
STORE_GLOBAL name

# Objects
NEW_OBJECT class_id
GET_FIELD field_id
SET_FIELD field_id
CALL_METHOD method_id argc

# Collections
NEW_ARRAY size
NEW_VECTOR
NEW_MAP
ARRAY_GET
ARRAY_SET
MAP_GET
MAP_SET

# Async/Concurrency
SPAWN fn_index
AWAIT
CHAN_SEND
CHAN_RECV
YIELD

# Exception handling
TRY_BEGIN handler_offset
TRY_END
THROW
```

### 4.2 Bytecode Format (.bxc)
```
Magic: 0x42 0x48 0x42 0x58  ('BHBX')
Version: u16
Flags: u16
Constant Pool: [Constant]
Functions: [Function]
Classes: [Class]
Globals: [Global]
Metadata: [Metadata]
```

---

## 5. Compiler Architecture

### 5.1 Pipeline
```
Source (.bx)
    ↓
Lexer → Tokens
    ↓
Parser → AST
    ↓
Semantic Analysis → Typed AST
    ↓
Optimization (AST-level)
    ↓
Bytecode Generation → .bxc
    ↓
    ├→ VM Interpreter (JIT)
    └→ Native Compiler (Rust/LLVM) → Binary
```

### 5.2 Compilation Modes
```bash
bx build          # Debug bytecode
bx build --release # Optimized bytecode
bx build --native  # Native binary (Rust backend)
bx build --target=android --release
bx build --target=ios --release
bx build --target=wasm --release
```

---

## 6. Standard Library (std)

### 6.1 Core Modules
```
std.io          # File I/O, stdin/stdout
std.fs          # Filesystem operations
std.net         # Networking (TCP, UDP, HTTP)
std.net.http    # HTTP client/server
std.net.ws      # WebSocket
std.collections # HashMap, HashSet, Vec, etc.
std.string      # String manipulation
std.json        # JSON serialization
std.time        # Time, duration, formatting
std.thread      # Threading, synchronization
std.async       # Async runtime
std.crypto      # Hashing, encryption, signing
std.math        # Mathematical functions
std.os          # OS-specific operations
std.ffi         # Foreign function interface
std.test        # Testing framework
```

### 6.2 Example APIs
```bx
# File I/O
std.fs.read_to_string("file.txt")
std.fs.write("file.txt", "content")
std.fs.copy("src", "dst")

# HTTP
resp = http.get("https://api.example.com")
json = resp.json()

# Async
async fn main():
    server = http.Server()
    server.get("/", || "Hello!")
    await server.listen(":8080")

# Collections
map = HashMap::new()
map.insert("key", "value")
vec = vec![1, 2, 3].filter(|x| x > 1).collect()
```

---

## 7. Package Manager (bxpkg)

### 7.1 Manifest (bxpkg.toml)
```toml
[package]
name = "my-app"
version = "1.0.0"
description = "A bhabex application"
license = "MIT"
authors = ["bxLabs <contact@bhabex.dev>"]
repository = "https://github.com/bxLabs/my-app"

[dependencies]
std = "1.0"
http = "0.5"
serde = "1.0"

[dev-dependencies]
test = "1.0"

[build]
targets = ["native", "wasm", "android"]
```

### 7.2 Commands
```bash
bxpkg new my-app          # Create new project
bxpkg add http@0.5        # Add dependency
bxpkg build               # Build project
bxpkg test                # Run tests
bxpkg publish             # Publish to registry
bxpkg install my-tool     # Install binary globally
```

---

## 8. Installer

### 8.1 Windows (.msi)
- Installs to `%LOCALAPPDATA%\Bhabex\`
- Adds to PATH
- Registers `.bx` file association
- Installs VS Code extension

### 8.2 macOS (.pkg)
- Installs to `/usr/local/bhabex/`
- Symlinks `bx` to `/usr/local/bin/`
- Registers `.bx` UTI

### 8.3 Linux (AppImage, .deb, .rpm, pacman)
- AppImage for universal distros
- Native packages for major distros
- Installs to `/opt/bhabex/`

### 8.4 Android/iOS
- Termux package for Android
- TestFlight/TestFairy for iOS development
- Cross-compilation from desktop

---

## 9. VS Code Extension

### 9.1 Features
- Syntax highlighting (TextMate grammar)
- IntelliSense (completion, hover, signature help)
- Diagnostics (errors, warnings)
- Go to definition/references
- Formatting (bx fmt)
- Debugging (DAP adapter)
- Project scaffolding

### 9.2 Configuration
```json
{
    "bhabex.path": "bx",
    "bhabex.formatOnSave": true,
    "bhabex.lintOnSave": true,
    "bhabex.target": "native"
}
```

---

## 10. Security Model (bxSecurity)

### 10.1 Capability-Based Security
```bx
# Capabilities required at compile-time
@capability(std.fs.read)
@capability(std.net.http)
fn main():
    # Only allowed operations compile
```

### 10.2 Sandboxing
- VM runs in sandboxed environment
- Native binaries use seccomp (Linux), sandbox (macOS), AppContainer (Windows)
- No ambient authority

---

## 11. Testing

### 11.1 Unit Tests
```bx
#[test]
fn test_add():
    assert_eq!(add(2, 3), 5)

#[test]
#[should_panic]
fn test_divide_by_zero():
    div(1, 0)
```

### 11.2 Integration Tests
```bx
#[integration]
fn test_http_server():
    server = spawn_test_server()
    resp = http.get(server.url("/health"))
    assert_eq!(resp.status, 200)
```

### 11.3 Property-Based Testing
```bx
#[quickcheck]
fn test_reverse_twice(xs: [i32]) -> bool:
    xs.reverse().reverse() == xs
```

---

## 12. Performance Targets

| Metric | Target |
|--------|--------|
| Startup time (VM) | < 10ms |
| Startup time (native) | < 1ms |
| Memory overhead (VM) | < 5MB |
| Throughput vs C | 80-95% |
| Compile time (10k LOC) | < 2s |

---

## 13. Roadmap

### v1.0 (MVP)
- [ ] Lexer, Parser, AST
- [ ] Bytecode VM
- [ ] Basic std library
- [ ] Native compilation (Rust backend)
- [ ] Package manager
- [ ] Windows/macOS/Linux installer

### v1.1
- [ ] Async runtime
- [ ] Android/iOS support
- [ ] VS Code extension
- [ ] WebAssembly target

### v2.0
- [ ] JIT compiler
- [ ] Incremental compilation
- [ ] Language server protocol
- [ ] Distributed compilation

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*