# Bhabex Standard Library

Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

## Core Modules

```
std/
├── io.bx        # File I/O, stdin/stdout
├── fs.bx        # Filesystem operations
├── net.bx       # Networking
│   ├── http.bx  # HTTP client/server
│   ├── ws.bx    # WebSocket
│   └── tcp.bx   # TCP/UDP
├── collections.bx  # HashMap, HashSet, Vec, etc.
├── string.bx    # String manipulation
├── json.bx      # JSON serialization
├── time.bx      # Time, duration
├── thread.bx    # Threading, sync
├── async.bx     # Async runtime
├── crypto.bx    # Hashing, encryption
├── math.bx      # Mathematical functions
├── os.bx        # OS operations
├── ffi.bx       # Foreign function interface
└── test.bx      # Testing framework
```

## Example: std.io

```bx
module std.io;

import std.fs;
import std.os;

pub fn print(args: ...) -> void:
    for arg in args:
        stdout.write(arg.to_string())
    stdout.write("\n")

pub fn println(args: ...) -> void:
    print(args...)

pub fn read_line() -> Result<string, IOError>:
    stdin.read_line()

pub fn write_file(path: string, content: string) -> Result<void, IOError>:
    fs.write(path, content)

pub fn read_file(path: string) -> Result<string, IOError>:
    fs.read_to_string(path)

pub fn append_file(path: string, content: string) -> Result<void, IOError>:
    fs.append(path, content)

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin:
    fn read_line(&self) -> Result<string, IOError>
    fn read_all(&self) -> Result<string, IOError>
    fn bytes(&self) -> BytesIterator

impl Stdout:
    fn write(&self, s: string) -> Result<void, IOError>
    fn flush(&self) -> Result<void, IOError>
    fn write_bytes(&self, bytes: []u8) -> Result<void, IOError>

impl Stderr:
    fn write(&self, s: string) -> Result<void, IOError>
    fn flush(&self) -> Result<void, IOError>
```

## Example: std.collections

```bx
module std.collections;

pub struct Vec[T] {
    ptr: *T,
    len: usize,
    cap: usize,
}

impl[T] Vec[T]:
    fn new() -> Vec[T]
    fn with_capacity(cap: usize) -> Vec[T]
    fn push(&mut self, value: T)
    fn pop(&mut self) -> Option[T]
    fn get(&self, index: usize) -> Option[&T]
    fn get_mut(&mut self, index: usize) -> Option[&mut T]
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn clear(&mut self)
    fn remove(&mut self, index: usize) -> T
    fn insert(&mut self, index: usize, value: T)
    fn iter(&self) -> Iter[T]
    fn iter_mut(&mut self) -> IterMut[T]
    fn map[U](self, f: fn(T) -> U) -> Vec[U]
    fn filter(self, f: fn(&T) -> bool) -> Vec[T]
    fn collect[C: FromIterator[T]>(self) -> C

pub struct HashMap[K, V] {
    // ...
}

impl[K, V] HashMap[K, V]:
    fn new() -> HashMap[K, V]
    fn insert(&mut self, key: K, value: V) -> Option[V]
    fn get(&self, key: &K) -> Option[&V]
    fn get_mut(&mut self, key: &K) -> Option[&mut V]
    fn remove(&mut self, key: &K) -> Option[V]
    fn contains_key(&self, key: &K) -> bool
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn clear(&mut self)
    fn keys(&self) -> Keys[K, V]
    fn values(&self) -> Values[K, V]
    fn iter(&self) -> Iter[K, V]

pub struct HashSet[T] {
    // ...
}

impl[T] HashSet[T]:
    fn new() -> HashSet[T]
    fn insert(&mut self, value: T) -> bool
    fn contains(&self, value: &T) -> bool
    fn remove(&mut self, value: &T) -> bool
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn clear(&mut self)
    fn iter(&self) -> Iter[T]
    fn union(&self, other: &HashSet[T]) -> HashSet[T]
    fn intersection(&self, other: &HashSet[T]) -> HashSet[T]
    fn difference(&self, other: &HashSet[T]) -> HashSet[T]
    fn symmetric_difference(&self, other: &HashSet[T]) -> HashSet[T]
```

## Example: std.async

```bx
module std.async;

pub async fn sleep(duration: Duration) -> void:
    // Sleep for duration

pub async fn timeout[T](duration: Duration, future: Future[T]) -> Result<T, TimeoutError>:
    // Timeout a future

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where F: Future + Send + 'static:
    // Spawn a task

pub async fn join_all<F>(futures: Vec<F>) -> Vec<F::Output>
where F: Future:
    // Wait for all futures

pub async fn select_all<F>(futures: Vec<F>) -> (usize, F::Output, Vec<F>)
where F: Future:
    // Wait for first future to complete

pub struct Channel[T] {
    // MPSC channel
}

impl[T] Channel[T]:
    fn new() -> (Sender[T], Receiver[T])
    fn bounded(capacity: usize) -> (Sender[T], Receiver[T])

pub struct Sender[T]:
    fn send(&self, value: T) -> Result<void, SendError>
    fn try_send(&self, value: T) -> Result<void, TrySendError>

pub struct Receiver[T]:
    fn recv(&self) -> Result[T, RecvError>
    fn try_recv(&self) -> Result<T, TryRecvError>
    fn iter(&self) -> Iter[T]

pub struct Mutex[T]:
    fn new(value: T) -> Mutex[T]
    fn lock(&self) -> MutexGuard[T]

pub struct RwLock[T]:
    fn new(value: T) -> RwLock[T]
    fn read(&self) -> RwLockReadGuard[T]
    fn write(&self) -> RwLockWriteGuard[T]

pub struct Condvar:
    fn new() -> Condvar
    fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T>
    fn notify_one(&self)
    fn notify_all(&self)

pub struct Semaphore:
    fn new(permits: usize) -> Semaphore
    fn acquire(&self) -> SemaphoreGuard
    fn try_acquire(&self) -> Option<SemaphoreGuard>
    fn add_permits(&self, n: usize)
```

## Example: std.crypto

```bx
module std.crypto;

pub mod hash:
    fn sha256(data: []u8) -> [u8; 32]
    fn sha512(data: []u8) -> [u8; 64]
    fn blake3(data: []u8) -> [u8; 32]
    fn md5(data: []u8) -> [u8; 16]

pub mod hmac:
    fn sign(key: []u8, data: []u8) -> []u8
    fn verify(key: []u8, data: []u8, sig: []u8) -> bool

pub mod cipher:
    fn aes_gcm_encrypt(key: [u8; 32], nonce: [u8; 12], data: []u8) -> Result<[]u8, CryptoError>
    fn aes_gcm_decrypt(key: [u8; 32], nonce: [u8; 12], data: []u8) -> Result<[]u8, CryptoError>
    fn chacha20_poly1305_encrypt(key: [u8; 32], nonce: [u8; 12], data: []u8) -> Result<[]u8, CryptoError>
    fn chacha20_poly1305_decrypt(key: [u8; 32], nonce: [u8; 12], data: []u8) -> Result<[]u8, CryptoError>

pub mod kdf:
    fn pbkdf2(password: []u8, salt: []u8, iterations: usize, key_len: usize) -> []u8
    fn argon2id(password: []u8, salt: []u8, key_len: usize) -> []u8
    fn scrypt(password: []u8, salt: []u8, n: u32, r: u32, p: u32, key_len: usize) -> []u8

pub mod rand:
    fn random_bytes(len: usize) -> []u8
    fn random_u64() -> u64
    fn random_range(min: u64, max: u64) -> u64

pub mod sign:
    fn ed25519_sign(private_key: [u8; 32], message: []u8) -> [u8; 64]
    fn ed25519_verify(public_key: [u8; 32], message: []u8, signature: [u8; 64]) -> bool
    fn generate_keypair() -> ([u8; 32], [u8; 32])
```

---

Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®