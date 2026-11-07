# Bhabex Standard Library API Reference

**Copyright BHABEX 2026** | **Securely designed & developed by bxLabs** | **Protected by bxSecurity** | **Powered by BHABEX®**

---

## Table of Contents

1. [Core Functions](#core-functions)
2. [File I/O](#file-io)
3. [JSON](#json)
4. [HTTP](#http)
5. [Regex](#regex)
6. [Math](#math)
7. [Datetime](#datetime)
8. [OS Utilities](#os-utilities)
9. [SQLite](#sqlite)
10. [JSON Schema](#json-schema)
11. [Built-in Types](#built-in-types)

---

## Core Functions

### `print(...values: any) -> nil`
Prints values to stdout separated by spaces, followed by newline.
```bx
print("Hello", 42, true)  // Hello 42 true
```

### `len(value: string | [any] | {any: any}) -> i32`
Returns length of string, array, or dict.
```bx
len("hello")      // 5
len([1,2,3])      // 3
len({"a": 1})     // 1
```

### `str(value: any) -> string`
Converts value to string.
```bx
str(42)        // "42"
str(true)      // "true"
str([1,2,3])   // "[1, 2, 3]"
```

### `int(value: f64 | string | i32) -> i32`
Converts to integer.
```bx
int(3.14)       // 3
int("42")       // 42
int("0xFF")     // 255
```

### `float(value: i32 | string | f64) -> f64`
Converts to float.
```bx
float(42)       // 42.0
float("3.14")   // 3.14
```

### `range(stop: i32) -> [i32]`
### `range(start: i32, stop: i32) -> [i32]`
### `range(start: i32, stop: i32, step: i32) -> [i32]`
Creates range of integers.
```bx
range(5)        // [0, 1, 2, 3, 4]
range(2, 5)     // [2, 3, 4]
range(0, 10, 2) // [0, 2, 4, 6, 8]
```

### `type(value: any) -> string`
Returns type name as string.
```bx
type(42)        // "i64"
type("hello")   // "string"
type([1,2,3])   // "list"
```

### `input(prompt?: string) -> string`
Reads line from stdin.
```bx
let name = input("Enter name: ")
```

### `list(...items: any) -> [any]`
Creates array from arguments.
```bx
list(1, 2, 3)   // [1, 2, 3]
```

### `dict(...pairs: any) -> {any: any}`
Creates dict from key-value pairs.
```bx
dict("a", 1, "b", 2)  // {"a": 1, "b": 2}
```

### `assert(condition: bool, message?: string) -> nil`
Panics if condition is false.
```bx
assert(x > 0, "x must be positive")
```

---

## File I/O

### `read_file(path: string, encoding?: string) -> string`
Reads entire file as string.
```bx
let content = read_file("config.txt")
let content = read_file("data.bin", "latin1")
```

### `write_file(path: string, content: string, encoding?: string) -> nil`
Writes string to file (overwrites).
```bx
write_file("output.txt", "Hello, World!")
write_file("data.txt", content, "utf-8")
```

### `append_file(path: string, content: string, encoding?: string) -> nil`
Appends string to file.
```bx
append_file("log.txt", "New line\n")
```

### `file_exists(path: string) -> bool`
Checks if file/directory exists.
```bx
if file_exists("config.txt") { ... }
```

### `is_file(path: string) -> bool`
Checks if path is a file.
```bx
is_file("config.txt")  // true
is_file("dir")         // false
```

### `is_dir(path: string) -> bool`
Checks if path is a directory.
```bx
is_dir("src")  // true
```

### `list_dir(path?: string) -> [string]`
Lists directory contents.
```bx
let files = list_dir(".")        // current dir
let files = list_dir("src/")     // specific dir
```

### `mkdir(path: string) -> nil`
Creates directory (including parents).
```bx
mkdir("output/data")
```

### `remove(path: string) -> nil`
Deletes file or empty directory.
```bx
remove("temp.txt")
remove("empty_dir")
```

### `rename(src: string, dst: string) -> nil`
Renames/moves file or directory.
```bx
rename("old.txt", "new.txt")
```

### `file_size(path: string) -> i64`
Returns file size in bytes.
```bx
let size = file_size("large.bin")
```

---

## JSON

### `json_parse(text: string) -> any`
Parses JSON string.
```bx
let obj = json_parse('{"name": "Alice", "age": 30}')
print(obj.name)  // Alice
```

### `json_stringify(value: any, indent?: i32) -> string`
Converts value to JSON string.
```bx
json_stringify({"a": 1})           // {"a":1}
json_stringify({"a": 1}, 2)        // pretty-printed with 2-space indent
```

---

## HTTP

### `http_get(url: string, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
GET request.
```bx
let resp = http_get("https://api.example.com/users")
if resp.status == 200 {
    let users = json_parse(resp.body)
}
```

### `http_post(url: string, data?: any, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
POST request. Dict/list data auto-encoded as JSON.
```bx
let resp = http_post("https://api.example.com/users", {"name": "Alice"})
let resp = http_post("https://api.example.com/raw", "raw data")
```

### `http_put(url: string, data?: any, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
PUT request.

### `http_delete(url: string, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
DELETE request.

### `http_patch(url: string, data?: any, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
PATCH request.

### `http_head(url: string, headers?: {string: string}, timeout?: f64) -> {status: i32, headers: {string: string}, body: string}`
HEAD request (no body).

---

## Regex

### `regex_match(pattern: string, text: string) -> [string] | nil`
First match with capture groups.
```bx
regex_match(r"(\d+)-(\d+)", "123-456")  // ["123-456", "123", "456"]
```

### `regex_findall(pattern: string, text: string) -> [string]`
All non-overlapping matches.
```bx
regex_findall(r"\d+", "v1.2.3")  // ["1", "2", "3"]
```

### `regex_replace(pattern: string, replacement: string, text: string, count?: i32) -> string`
Replaces matches.
```bx
regex_replace(r"\d+", "#", "v1.2.3")          // "v#.#.#"
regex_replace(r"\s+", " ", "a   b   c", 1)    // "a b   c"
```

### `regex_split(pattern: string, text: string, maxsplit?: i32) -> [string]`
Splits by pattern.
```bx
regex_split(r"\s+", "a b c")  // ["a", "b", "c"]
```

---

## Math

### Trigonometric
```bx
sin(x: f64) -> f64
cos(x: f64) -> f64
tan(x: f64) -> f64
```
Arguments in radians.

### Power & Roots
```bx
sqrt(x: f64) -> f64
pow(x: f64, y: f64) -> f64
```

### Logarithmic & Exponential
```bx
log(x: f64, base?: f64) -> f64  // natural log if base omitted
exp(x: f64) -> f64
```

### Rounding
```bx
floor(x: f64) -> i32
ceil(x: f64) -> i32
```

### Random
```bx
random() -> f64                    // [0, 1)
random(max: f64) -> f64            // [0, max)
random(min: f64, max: f64) -> f64  // [min, max)

randint(min: i32, max: i32) -> i32  // [min, max] inclusive
```

### Constants
```bx
pi() -> f64    // 3.141592653589793
e() -> f64     // 2.718281828459045
```

---

## Datetime

### `now(format?: string) -> string`
Current datetime formatted.
```bx
now()                    // "2026-11-07 16:30:45"
now("%Y-%m-%d")         // "2026-11-07"
now("%H:%M:%S")         // "16:30:45"
```

### `parse_datetime(text: string, format?: string) -> string`
Parses datetime string.
```bx
parse_datetime("2026-11-07 16:30:45")
parse_datetime("07/11/2026", "%d/%m/%Y")
```

### `timestamp() -> f64`
Unix timestamp (seconds since epoch).
```bx
let ts = timestamp()
```

### `sleep(seconds: f64) -> nil`
Blocking sleep.
```bx
sleep(1.0)
sleep(0.5)
```

### `async_sleep(seconds: f64) -> nil`
Async sleep (yields to runtime).
```bx
async fn main() {
    await async_sleep(1.0)
}
```

---

## OS Utilities

### `env(key: string, default?: string) -> string | nil`
Gets environment variable.
```bx
let home = env("HOME")
let port = env("PORT", "8080")
```

### `args() -> [string]`
Command line arguments (excluding program name).
```bx
let args = args()  // ["--verbose", "file.txt"]
```

### `exit(code?: i32) -> nil`
Exits process.
```bx
exit(0)
exit(1)
```

### `cwd() -> string`
Current working directory.
```bx
let dir = cwd()
```

### `chdir(path: string) -> nil`
Changes working directory.
```bx
chdir("/tmp")
```

### `platform() -> string`
OS platform identifier.
```bx
platform()  // "win32", "darwin", "linux", "android", "ios"
```

---

## SQLite

### `sqlite_open(path: string) -> i64`
Opens database, returns connection ID.
```bx
let conn = sqlite_open("app.db")
```

### `sqlite_exec(conn: i64, sql: string) -> i32`
Executes SQL (DDL/DML), returns affected rows.
```bx
sqlite_exec(conn, "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
sqlite_exec(conn, "INSERT INTO users (name) VALUES ('Alice')")
```

### `sqlite_query(conn: i64, sql: string) -> [any]`
Executes SELECT, returns array of row dicts.
```bx
let users = sqlite_query(conn, "SELECT * FROM users")
for user in users {
    print(user.name)
}
```

### `sqlite_close(conn: i64) -> nil`
Closes connection.
```bx
sqlite_close(conn)
```

---

## JSON Schema

### `json_schema_validate(schema: any, instance: any) -> bool`
Validates instance against JSON Schema.
```bx
let schema = {
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0}
    },
    "required": ["name", "age"]
}

json_schema_validate(schema, {"name": "Alice", "age": 30})  // true
json_schema_validate(schema, {"name": "Bob", "age": -5})    // false
```

### `json_schema_draft(schema: any) -> string`
Returns schema draft identifier.
```bx
json_schema_draft(schema)  // "http://json-schema.org/draft-07/schema#"
```

---

## Built-in Types

### Primitive Types
| Type | Description |
|------|-------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | Signed integers |
| `u8`, `u16`, `u32`, `u64`, `u128` | Unsigned integers |
| `isize`, `usize` | Pointer-sized |
| `f32`, `f64` | IEEE 754 floats |
| `bool` | `true` / `false` |
| `char` | Unicode scalar |
| `string` | UTF-8 owned string |

### Composite Types
```bx
# Tuple
let t: (i32, string, bool) = (1, "hello", true)

# Fixed array
let arr: [i32; 3] = [1, 2, 3]

# Vector (dynamic)
let vec: [i32] = [1, 2, 3]

# Map
let map: {string: i32} = {"a": 1}

# Option
let opt: Option[i32] = Option.Some(42)
let none: Option[i32] = Option.None

# Result
let res: Result<string, Error> = Result.Ok("success")
let err: Result<string, Error> = Result.Err(IOError("not found"))
```

### Type Aliases
```bx
type UserId = i64
type UserMap = {string: User}
type Callback = fn(string) -> Result<(), Error>
```

---

## Built-in Constants

```bx
true    // boolean true
false   // boolean false
nil     // null/none value
```

---

## Error Types

```bx
# Common errors thrown by stdlib
IOError(message: string)
ParseError(message: string)
RuntimeError(message: string)
NetworkError(message: string)
```

---

*Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®*