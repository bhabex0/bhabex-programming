# Bhabex Tutorial - Getting Started

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

---

## 1. Installation

### Windows
```powershell
# Via installer
# Download bhabex-installer.exe from https://bhabex.dev/downloads

# Or via package manager
choco install bhabex
```

### macOS
```bash
# Via installer
# Download Bhabex-0.1.0-macos.pkg from https://bhabex.dev/downloads

# Or via Homebrew
brew install bhabex
```

### Linux
```bash
# AppImage (universal)
wget https://bhabex.dev/downloads/Bhabex-0.1.0-x86_64.AppImage
chmod +x Bhabex-0.1.0-x86_64.AppImage
sudo mv Bhabex-0.1.0-x86_64.AppImage /usr/local/bin/bx

# Debian/Ubuntu
wget https://bhabex.dev/downloads/bhabex_0.1.0_amd64.deb
sudo dpkg -i bhabex_0.1.0_amd64.deb

# Fedora/RHEL
wget https://bhabex.dev/downloads/bhabex-0.1.0-1.x86_64.rpm
sudo rpm -i bhabex-0.1.0-1.x86_64.rpm
```

### Android (Termux)
```bash
dpkg -i bhabex_0.1.0_aarch64.deb
```

### Verify Installation
```bash
bx version
# Bhabex v0.1.0
```

---

## 2. Your First Program

Create `hello.bx`:
```bx
fn main() {
    print("Hello, Bhabex!")
}
```

Run it:
```bash
bx run hello.bx
# Hello, Bhabex!
```

---

## 3. Variables and Types

### Basic Types
```bx
fn main() {
    # Integers
    let x = 42           // i64 (default)
    let y: i32 = 100     // explicit 32-bit
    let z: u8 = 255      // unsigned 8-bit

    # Floats
    let pi = 3.14159     // f64
    let ratio: f32 = 1.5 // 32-bit float

    # Booleans
    let is_true = true
    let is_false = false

    # Strings
    let name = "Bhabex"
    let multiline = """Line 1
Line 2
Line 3"""

    print(x, y, z, pi, ratio, is_true, name)
}
```

### Type Annotations
```bx
let explicit: i64 = 42
let items: [i32] = [1, 2, 3]
let user: {name: string, age: i32} = {name: "Alice", age: 30}
```

---

## 4. Control Flow

### If Expressions
```bx
fn main() {
    let score = 85

    // If as expression (returns value)
    let grade = if score >= 90: "A"
    elif score >= 80: "B"
    elif score >= 70: "C"
    else: "F"

    print("Grade:", grade)

    // If as statement
    if score >= 60 {
        print("Passed!")
    } else {
        print("Failed!")
    }
}
```

### Pattern Matching
```bx
fn describe(value: any) -> string {
    match value {
        0 -> "zero"
        1 -> "one"
        n if n > 0 -> "positive: {n}"
        n if n < 0 -> "negative: {n}"
        "hello" -> "greeting"
        [first, ..rest] -> "list starting with {first}"
        {name: string, ..} -> "object with name: {name}"
        _ -> "unknown"
    }
}

fn main() {
    print(describe(42))        // positive: 42
    print(describe("hello"))   // greeting
    print(describe([1,2,3]))   // list starting with 1
    print(describe({name: "Alice"})) // object with name: Alice
}
```

### Loops
```bx
fn main() {
    # For over range
    for i in 0..10 {      // 0 to 9
        print(i)
    }

    for i in 0..=10 {     // 0 to 10 (inclusive)
        print(i)
    }

    # For over collection
    let items = ["apple", "banana", "cherry"]
    for item in items {
        print(item)
    }

    # With index
    for i, item in items.enumerate() {
        print("{i}: {item}")
    }

    # While loop
    let mut count = 0
    while count < 5 {
        print(count)
        count += 1
    }

    # Loop with break/continue
    loop {
        if some_condition() { break }
        if skip_condition() { continue }
        do_work()
    }
}
```

---

## 5. Functions

### Basic Functions
```bx
# Simple function with explicit return
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

# Expression body (implicit return)
fn mul(a: i32, b: i32) -> i32: a * b

# Multiple parameters
fn greet(name: string, times: i32 = 1) -> string {
    "Hello, {name}! ".repeat(times)
}

fn main() {
    print(add(2, 3))        // 5
    print(mul(4, 5))        // 20
    print(greet("Alice"))   // Hello, Alice!
    print(greet("Bob", 3))  // Hello, Bob! Hello, Bob! Hello, Bob!
}
```

### Async Functions
```bx
async fn fetch_user(id: i32) -> User {
    let resp = await http.get("/api/users/{id}")
    resp.json()
}

async fn main() {
    let users = await all([
        fetch_user(1),
        fetch_user(2),
        fetch_user(3)
    ])
    for user in users {
        print(user.name)
    }
}
```

### Generics
```bx
fn first[T](items: [T]) -> Option[T] {
    if items.is_empty(): Option.None
    else: Option.Some(items[0])
}

fn map[T, U](items: [T], f: fn(T) -> U) -> [U] {
    let mut result = []
    for item in items {
        result.push(f(item))
    }
    result
}

fn main() {
    let nums = [1, 2, 3]
    print(first(nums))        // Option.Some(1)
    print(map(nums, |x| x * 2))  // [2, 4, 6]
}
```

---

## 6. Data Structures

### Structs
```bx
struct User {
    name: string
    age: i32
    email: string?
    active: bool = true
}

struct Point {
    x: f64
    y: f64
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx
        self.y += dy
    }
}

fn main() {
    let p1 = Point::new(0.0, 0.0)
    let p2 = Point::new(3.0, 4.0)
    print(p1.distance(&p2))  // 5.0

    let mut p = Point::new(1.0, 2.0)
    p.translate(2.0, 3.0)
    print(p.x, p.y)  // 3.0 5.0
}
```

### Enums
```bx
enum Result[T, E] {
    Ok(T)
    Err(E)
}

enum Option[T] {
    Some(T)
    None
}

enum Shape {
    Circle { radius: f64 }
    Rectangle { width: f64, height: f64 }
    Triangle { a: f64, b: f64, c: f64 }
}

fn area(shape: Shape) -> f64 {
    match shape {
        Shape.Circle { radius } => 3.14159 * radius * radius
        Shape.Rectangle { width, height } => width * height
        Shape.Triangle { a, b, c } => {
            let s = (a + b + c) / 2.0
            (s * (s - a) * (s - b) * (s - c)).sqrt()
        }
    }
}

fn main() {
    let shapes = [
        Shape.Circle { radius: 5.0 },
        Shape.Rectangle { width: 3.0, height: 4.0 },
        Shape.Triangle { a: 3.0, b: 4.0, c: 5.0 }
    ]

    for shape in shapes {
        print(area(shape))
    }
}
```

### Collections
```bx
fn main() {
    # Vectors (dynamic arrays)
    let mut vec = [1, 2, 3]
    vec.push(4)
    vec.pop()
    print(vec)  // [1, 2, 3]

    # Maps
    let mut map = {"a": 1, "b": 2}
    map.insert("c", 3)
    print(map["a"])  // 1

    # Sets
    let mut set = {1, 2, 3}
    set.insert(4)
    set.remove(2)
    print(set)  // {1, 3, 4}
}
```

---

## 7. Error Handling

### Result Type
```bx
fn divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 {
        Result.Err("Division by zero")
    } else {
        Result.Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Result.Ok(val) => print("Result:", val)
        Result.Err(e) => print("Error:", e)
    }

    // Using ? operator for propagation
    let result = divide(10.0, 0.0)?  // This would propagate the error
    print(result)
}
```

### Try-Catch
```bx
fn risky_operation() -> string {
    if random() < 0.5 {
        throw "Random failure!"
    }
    "Success"
}

fn main() {
    try {
        let result = risky_operation()
        print(result)
    } catch Error as e {
        print("Caught:", e)
    }

    # Multiple catch blocks
    try {
        do_something()
    } catch IOError as e {
        print("IO error:", e)
    } catch NetworkError as e {
        print("Network error:", e)
    } catch _ as e {
        print("Other error:", e)
    }
}
```

---

## 8. Modules and Imports

### Project Structure
```
my-project/
├── bxpkg.toml
├── main.bx
├── utils/
│   ├── mod.bx
│   ├── math.bx
│   └── string.bx
└── lib/
    └── helper.bx
```

### Importing
```bx
# main.bx
import std.io
import std.net.http as http
import std.collections.{HashMap, HashSet}

import ./utils/math
import ./utils/string as str_utils

# Re-export
pub export fn helper(): i32 { 42 }

fn main() {
    let result = utils.math.add(2, 3)
    print(result)
}
```

### Module Definition
```bx
# utils/math.bx
pub fn add(a: i32, b: i32) -> i32: a + b
pub fn sub(a: i32, b: i32) -> i32: a - b

# Private function (not exported)
fn internal(): i32 { 42 }
```

---

## 9. Building and Running

### Commands
```bash
# Run a file
bx run main.bx

# Build bytecode
bx build main.bx

# Build optimized bytecode
bx build main.bx --release

# Build native binary
bx build main.bx --native

# Check syntax
bx check main.bx

# Format code
bx fmt main.bx

# Start REPL
bx repl

# Evaluate inline
bx eval "print(2+2)"

# Create new project
bx init my-app

# Run tests
bx test
```

### Project with bxpkg
```bash
# Initialize
bx init my-app

# Add dependencies
bx pkg add http@0.5
bx pkg add serde@1.0

# Build
bx pkg build

# Run tests
bx pkg test

# Publish
bx pkg publish
```

---

## 10. Standard Library Quick Reference

```bx
# File I/O
read_file("path.txt")
write_file("path.txt", "content")
append_file("path.txt", "more")
file_exists("path.txt")
list_dir(".")

# JSON
json_parse('{"a":1}')
json_stringify({"a": 1})

# HTTP
http_get("https://api.example.com")
http_post("https://api.example.com", {"key": "value"})

# Regex
regex_findall("\d+", "abc123def456")  // ["123", "456"]
regex_replace("\d+", "#", "v1.2.3")    // "v#.#.#"

# Math
sin(0), cos(0), sqrt(16), pow(2, 10)
random(), randint(1, 6)

# Datetime
now(), timestamp(), sleep(1.0)

# OS
env("HOME"), cwd(), platform()

# SQLite
sqlite_open("test.db")
sqlite_exec(conn, "CREATE TABLE ...")
sqlite_query(conn, "SELECT * FROM users")
sqlite_close(conn)
```

---

## Next Steps

- Read the [Language Specification](LANGUAGE_SPEC.md)
- Explore the [API Reference](API_REFERENCE.md)
- Check out [Examples](../examples/)
- Join the community at https://bhabex.dev/community

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*