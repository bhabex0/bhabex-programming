//! Bhabex Bytecode - Portable instruction set for the Bhabex VM
//!
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, Condvar, RwLock, MutexGuard};
use crate::types::*;
use crate::vm::VMError;

/// Bytecode program container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    pub version: u16,
    pub flags: u16,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    pub globals: Vec<Global>,
    pub metadata: HashMap<String, String>,
    pub entry_point: Option<usize>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            version: 1,
            flags: 0,
            constants: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            globals: Vec::new(),
            metadata: HashMap::new(),
            entry_point: None,
        }
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn add_function(&mut self, function: Function) -> usize {
        self.functions.push(function);
        self.functions.len() - 1
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

/// Constants in the constant pool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Function(usize),  // Index into functions table
    Type(usize),      // Index into type table
}

/// Object reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectRef {
    pub class_id: usize,
    pub fields: Vec<Value>,
}

impl ObjectRef {
    pub fn new(class_id: usize, field_count: usize) -> Self {
        Self {
            class_id,
            fields: vec![Value::Null; field_count],
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Self> {
        // This is a placeholder - in real implementation, ObjectRef would be wrapped in Arc<Mutex<Self>>
        // For now, we'll use a dummy implementation
        panic!("ObjectRef::lock() not implemented - ObjectRef should be wrapped in Arc<Mutex<_>>")
    }
    
    pub fn get_field(&self, index: usize) -> Option<Value> {
        self.fields.get(index).cloned()
    }
    
    pub fn set_field(&mut self, index: usize, value: Value) -> Result<(), crate::vm::VMError> {
        if index < self.fields.len() {
            self.fields[index] = value;
            Ok(())
        } else {
            Err(crate::vm::VMError::IndexOutOfBounds(index))
        }
    }
}

/// Closure reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClosureRef {
    pub function_id: usize,
    pub captures: Vec<Value>,
}

/// Channel reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelRef {
    pub id: usize,
    pub inner: Arc<ChannelInner>,
}

#[derive(Debug)]
pub struct ChannelInner {
    pub queue: Mutex<Vec<Value>>,
    pub closed: Mutex<bool>,
    pub sender_count: Mutex<usize>,
    pub receiver_count: Mutex<usize>,
    pub not_empty: Condvar,
    pub not_full: Condvar,
    pub capacity: usize,
}

impl ChannelRef {
    pub fn new(capacity: usize) -> Self {
        Self {
            id: 0, // Will be set by VM
            inner: Arc::new(ChannelInner {
                queue: Mutex::new(Vec::with_capacity(capacity)),
                closed: Mutex::new(false),
                sender_count: Mutex::new(0),
                receiver_count: Mutex::new(0),
                not_empty: Condvar::new(),
                not_full: Condvar::new(),
                capacity,
            }),
        }
    }

    pub fn send(&self, value: Value) -> Result<(), VMError> {
        let inner = &self.inner;
        let mut queue = inner.queue.lock().unwrap();
        let closed = inner.closed.lock().unwrap();
        
        if *closed {
            return Err(VMError::ChannelError("Channel closed".to_string()));
        }
        
        while queue.len() >= inner.capacity && inner.capacity > 0 {
            queue = inner.not_full.wait(queue).unwrap();
            let closed = inner.closed.lock().unwrap();
            if *closed {
                return Err(VMError::ChannelError("Channel closed".to_string()));
            }
        }
        
        queue.push(value);
        inner.not_empty.notify_one();
        Ok(())
    }

    pub fn recv(&self) -> Result<Value, VMError> {
        let inner = &self.inner;
        let mut queue = inner.queue.lock().unwrap();
        let closed = inner.closed.lock().unwrap();
        
        while queue.is_empty() && !*closed {
            queue = inner.not_empty.wait(queue).unwrap();
        }
        
        if queue.is_empty() && *closed {
            return Err(VMError::ChannelError("Channel closed and empty".to_string()));
        }
        
        let value = queue.remove(0);
        inner.not_full.notify_one();
        Ok(value)
    }

    pub fn close(&self) {
        let mut closed = self.inner.closed.lock().unwrap();
        *closed = true;
        self.inner.not_empty.notify_all();
        self.inner.not_full.notify_all();
    }
}

/// Mutex reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MutexRef {
    pub id: usize,
    pub inner: Arc<Mutex<()>>,
}

impl MutexRef {
    pub fn new() -> Self {
        Self {
            id: 0, // Will be set by VM
            inner: Arc::new(Mutex::new(())),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, ()> {
        self.inner.lock().unwrap()
    }
}

/// RWLock reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RwLockRef {
    pub id: usize,
    pub inner: Arc<RwLockInner>,
}

#[derive(Debug)]
pub struct RwLockInner {
    pub write_lock: Mutex<()>,
    pub read_count: Mutex<usize>,
    pub write_waiting: Mutex<bool>,
}

impl RwLockRef {
    pub fn new() -> Self {
        Self {
            id: 0, // Will be set by VM
            inner: Arc::new(RwLockInner {
                write_lock: Mutex::new(()),
                read_count: Mutex::new(0),
                write_waiting: Mutex::new(false),
            }),
        }
    }

    pub fn read_lock(&self) -> RwLockReadGuard {
        let mut count = self.inner.read_count.lock().unwrap();
        *count += 1;
        RwLockReadGuard { 
            count,
            inner: self.inner.clone(),
        }
    }

    pub fn write_lock(&self) -> RwLockWriteGuard {
        let _write_waiting = self.inner.write_waiting.lock().unwrap();
        let guard = self.inner.write_lock.lock().unwrap();
        RwLockWriteGuard { _guard: guard }
    }
}

pub struct RwLockReadGuard {
    count: std::sync::MutexGuard<'static, usize>,
    inner: Arc<RwLockInner>,
}

impl Drop for RwLockReadGuard {
    fn drop(&mut self) {
        let mut count = self.inner.read_count.lock().unwrap();
        *count -= 1;
    }
}

pub struct RwLockWriteGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

/// Native function
#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction {
    pub name: String,
    pub func_id: usize,
    #[cfg(not(feature = "serde"))]
    pub func: Option<Box<dyn Fn(&mut VM, Vec<Value>) -> Result<Value, VMError> + Send + Sync>>,
    #[cfg(feature = "serde")]
    pub func: Option<usize>, // Placeholder for serialization
}

#[cfg(feature = "serde")]
impl Serialize for NativeFunction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("NativeFunction", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("func_id", &self.func_id)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NativeFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NativeFunctionData {
            name: String,
            func_id: usize,
        }
        let data = NativeFunctionData::deserialize(deserializer)?;
        Ok(NativeFunction {
            name: data.name,
            func_id: data.func_id,
            func: None, // Function pointer not serialized
        })
    }
}

/// Generator reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratorRef {
    pub id: usize,
}

/// VM Value - represents any runtime value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Object(ObjectRef),
    Closure(ClosureRef),
    Channel(ChannelRef),
    Mutex(MutexRef),
    RwLock(RwLockRef),
    Function(usize), // Function index
    NativeFunction(NativeFunction),
    Generator(GeneratorRef),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Object(_) => "object",
            Value::Closure(_) => "closure",
            Value::Channel(_) => "channel",
            Value::Mutex(_) => "mutex",
            Value::RwLock(_) => "rwlock",
            Value::Function(_) => "function",
            Value::NativeFunction(_) => "native_fn",
            Value::Generator(_) => "generator",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Char(c) => *c != '\0',
            Value::String(s) => !s.is_empty(),
            Value::Bytes(b) => !b.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            Value::Object(_) => true,
            Value::Closure(_) => true,
            Value::Channel(_) => true,
            Value::Mutex(_) => true,
            Value::RwLock(_) => true,
            Value::Function(_) => true,
            Value::NativeFunction(_) => true,
            Value::Generator(_) => true,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Char(c) => Some(*c as i64),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bytes(b) => write!(f, "bytes({})", b.len()),
            Value::Array(a) => write!(f, "[{}]", a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Tuple(t) => write!(f, "({})", t.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Object(_) => write!(f, "object@{}", self.type_name()),
            Value::Closure(_) => write!(f, "closure"),
            Value::Channel(_) => write!(f, "channel"),
            Value::Mutex(_) => write!(f, "mutex"),
            Value::RwLock(_) => write!(f, "rwlock"),
            Value::Function(idx) => write!(f, "fn#{}", idx),
            Value::NativeFunction(nf) => write!(f, "native_fn({})", nf.name),
            Value::Generator(_) => write!(f, "generator"),
        }
    }
}

/// Function bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: TypeInfo,
    pub locals: Vec<LocalInfo>,
    pub instructions: Vec<Instruction>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub is_async: bool,
    pub is_generator: bool,
    pub span: Span,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub ty: TypeInfo,
    pub is_mut: bool,
    pub is_ref: bool,
    pub default: Option<usize>, // Constant index for default value
}

/// Local variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalInfo {
    pub name: String,
    pub ty: TypeInfo,
    pub is_mut: bool,
}

/// Type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeInfo {
    Primitive(PrimitiveType),
    Named(String, Vec<TypeInfo>),
    Tuple(Vec<TypeInfo>),
    Array(Box<TypeInfo>, Option<usize>),
    Slice(Box<TypeInfo>),
    Ref(Box<TypeInfo>, bool),
    RawPtr(Box<TypeInfo>, bool),
    Function(FunctionType),
    Generic(String),
    Never,
}

/// Primitive types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    Char,
    Str,
    String,
    Int,  // Generic integer (i64)
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
    F32, F64,
    Unit,
    Never,
}

/// Function type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionType {
    pub params: Vec<TypeInfo>,
    pub return_type: Box<TypeInfo>,
    pub is_async: bool,
    pub is_unsafe: bool,
}

/// Class/Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<usize>, // Function indices
    pub is_struct: bool,
    pub span: Span,
}

/// Field information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub ty: TypeInfo,
    pub is_pub: bool,
    pub is_mut: bool,
    pub offset: usize,
}

/// Global variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Global {
    pub name: String,
    pub ty: TypeInfo,
    pub is_mut: bool,
    pub is_pub: bool,
    pub init: Option<usize>, // Constant index
    pub span: Span,
}

/// Exception handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionHandler {
    pub try_start: usize,
    pub try_end: usize,
    pub catch_type: Option<TypeInfo>,
    pub catch_start: usize,
    pub finally_start: Option<usize>,
}

/// VM Instructions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Instruction {
    // Stack manipulation
    Nop,
    Pop,
    Dup,
    Swap,
    Rot3,
    
    // Constants
    PushNull,
    PushBool(bool),
    PushInt(i64),
    PushFloat(f64),
    PushConst(usize), // Constant pool index
    
    // Local variables
    LoadLocal(usize),
    StoreLocal(usize),
    LoadArg(usize),
    StoreArg(usize),
    
    // Global variables
    LoadGlobal(usize),
    StoreGlobal(usize),
    
    // Fields
    LoadField(usize),   // Field index
    StoreField(usize),
    LoadFieldStatic(usize, usize), // Class index, field index
    StoreFieldStatic(usize, usize),
    
    // Array/Vector operations
    NewArray(usize),    // Element type index
    NewVector(usize),
    ArrayLen,
    ArrayGet,
    ArraySet,
    VectorPush,
    VectorPop,
    
    // Slice operations
    SliceNew,
    SliceGet,
    SliceSet,
    SliceLen,
    
    // Tuple operations
    NewTuple(usize),    // Arity
    TupleGet(usize),
    TupleSet(usize),
    
    // Object operations
    NewObject(usize),   // Class index
    NewStruct(usize),   // Struct index
    CheckCast(usize),   // Type index
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Pow,
    
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    Jump(usize),
    JumpIf(usize),
    JumpIfNot(usize),
    JumpTable(Vec<usize>),
    
    // Function calls
    Call(usize, usize),        // Function index, arg count
    CallMethod(usize, usize),  // Method index, arg count
    CallVirtual(usize, usize), // VTable index, arg count
    CallInterface(usize, usize), // Interface index, arg count
    TailCall(usize, usize),
    Return,
    ReturnVoid,
    
    // Async
    Await,
    Yield,
    Spawn(usize),      // Function index
    SpawnClosure(usize), // Closure index
    
    // Exception handling
    TryBegin(usize),   // Handler offset
    TryEnd,
    Throw,
    Rethrow,
    
    // Closures
    NewClosure(usize),  // Function index
    ClosureCall(usize), // Closure index, arg count
    LoadCapture(usize), // Capture index
    StoreCapture(usize),
    
    // Async/Generators
    GenNext,
    GenYield,
    GenReturn,
    
    // Channels
    ChanNew(usize),    // Type index
    ChanSend,
    ChanRecv,
    ChanClose,
    
    // Synchronization
    MutexLock,
    MutexUnlock,
    RwLockRead,
    RwLockWrite,
    RwLockUnlock,
    CondWait,
    CondSignal,
    CondBroadcast,
    
    // Type operations
    TypeOf,
    SizeOf,
    AlignOf,
    
    // Memory
    Alloc(usize),      // Size
    AllocArray(usize), // Element type, count
    Free,
    MemCopy,
    MemMove,
    MemSet,
    
    // FFI
    FfiCall(usize),    // External function index
    
    // Debug
    DebugLine(usize),  // Line number
    DebugScopeEnter,
    DebugScopeExit,
    Breakpoint,
}

impl Instruction {
    /// Get the size of this instruction in bytes (for serialization)
    pub fn size(&self) -> usize {
        1 + match self {
            Instruction::PushInt(_) => 8,
            Instruction::PushFloat(_) => 8,
            Instruction::PushConst(_) | Instruction::LoadLocal(_) | Instruction::StoreLocal(_)
            | Instruction::LoadArg(_) | Instruction::StoreArg(_)
            | Instruction::LoadGlobal(_) | Instruction::StoreGlobal(_)
            | Instruction::LoadField(_) | Instruction::StoreField(_)
            | Instruction::NewArray(_) | Instruction::NewVector(_)
            | Instruction::NewTuple(_) | Instruction::TupleGet(_) | Instruction::TupleSet(_)
            | Instruction::NewObject(_) | Instruction::NewStruct(_) | Instruction::CheckCast(_)
            | Instruction::Call(_, _) | Instruction::CallMethod(_, _) | Instruction::CallVirtual(_, _)
            | Instruction::CallInterface(_, _) | Instruction::TailCall(_, _)
            | Instruction::Spawn(_) | Instruction::SpawnClosure(_) | Instruction::LoadCapture(_)
            | Instruction::StoreCapture(_) | Instruction::NewClosure(_) | Instruction::ClosureCall(_)
            | Instruction::ChanNew(_) | Instruction::Alloc(_) | Instruction::AllocArray(_)
            | Instruction::FfiCall(_) | Instruction::DebugLine(_)
            | Instruction::Jump(_) | Instruction::JumpIf(_) | Instruction::JumpIfNot(_) => 4,
            Instruction::LoadFieldStatic(_, _) | Instruction::StoreFieldStatic(_, _)
            | Instruction::CallMethod(_, _) => 8,
            Instruction::JumpTable(v) => 4 * v.len(),
            _ => 0,
        }
    }

    /// Get opcode byte
    pub fn opcode(&self) -> u8 {
        match self {
            Instruction::Nop => 0x00,
            Instruction::Pop => 0x01,
            Instruction::Dup => 0x02,
            Instruction::Swap => 0x03,
            Instruction::Rot3 => 0x04,
            Instruction::PushNull => 0x10,
            Instruction::PushBool(_) => 0x11,
            Instruction::PushInt(_) => 0x12,
            Instruction::PushFloat(_) => 0x13,
            Instruction::PushConst(_) => 0x14,
            Instruction::LoadLocal(_) => 0x20,
            Instruction::StoreLocal(_) => 0x21,
            Instruction::LoadArg(_) => 0x22,
            Instruction::StoreArg(_) => 0x23,
            Instruction::LoadGlobal(_) => 0x24,
            Instruction::StoreGlobal(_) => 0x25,
            Instruction::LoadField(_) => 0x26,
            Instruction::StoreField(_) => 0x27,
            Instruction::LoadFieldStatic(_, _) => 0x28,
            Instruction::StoreFieldStatic(_, _) => 0x29,
            Instruction::NewArray(_) => 0x30,
            Instruction::NewVector(_) => 0x31,
            Instruction::ArrayLen => 0x32,
            Instruction::ArrayGet => 0x33,
            Instruction::ArraySet => 0x34,
            Instruction::VectorPush => 0x35,
            Instruction::VectorPop => 0x36,
            Instruction::NewTuple(_) => 0x37,
            Instruction::TupleGet(_) => 0x38,
            Instruction::TupleSet(_) => 0x39,
            Instruction::SliceNew => 0x3A,
            Instruction::SliceGet => 0x3B,
            Instruction::SliceSet => 0x3C,
            Instruction::SliceLen => 0x3D,
            Instruction::NewObject(_) => 0x40,
            Instruction::NewStruct(_) => 0x41,
            Instruction::CheckCast(_) => 0x42,
            Instruction::Add => 0x50,
            Instruction::Sub => 0x51,
            Instruction::Mul => 0x52,
            Instruction::Div => 0x53,
            Instruction::Mod => 0x54,
            Instruction::Neg => 0x55,
            Instruction::Pow => 0x56,
            Instruction::BitAnd => 0x57,
            Instruction::BitOr => 0x58,
            Instruction::BitXor => 0x59,
            Instruction::BitNot => 0x5A,
            Instruction::Shl => 0x5B,
            Instruction::Shr => 0x5C,
            Instruction::Eq => 0x60,
            Instruction::Ne => 0x61,
            Instruction::Lt => 0x62,
            Instruction::Gt => 0x63,
            Instruction::Le => 0x64,
            Instruction::Ge => 0x65,
            Instruction::And => 0x66,
            Instruction::Or => 0x67,
            Instruction::Not => 0x68,
            Instruction::Jump(_) => 0x70,
            Instruction::JumpIf(_) => 0x71,
            Instruction::JumpIfNot(_) => 0x72,
            Instruction::JumpTable(_) => 0x73,
            Instruction::Call(_, _) => 0x80,
            Instruction::CallMethod(_, _) => 0x81,
            Instruction::CallVirtual(_, _) => 0x82,
            Instruction::CallInterface(_, _) => 0x83,
            Instruction::TailCall(_, _) => 0x84,
            Instruction::Return => 0x85,
            Instruction::ReturnVoid => 0x86,
            Instruction::Await => 0x90,
            Instruction::Yield => 0x91,
            Instruction::Spawn(_) => 0x92,
            Instruction::SpawnClosure(_) => 0x93,
            Instruction::TryBegin(_) => 0xA0,
            Instruction::TryEnd => 0xA1,
            Instruction::Throw => 0xA2,
            Instruction::Rethrow => 0xA3,
            Instruction::NewClosure(_) => 0xB0,
            Instruction::ClosureCall(_) => 0xB1,
            Instruction::LoadCapture(_) => 0xB2,
            Instruction::StoreCapture(_) => 0xB3,
            Instruction::GenNext => 0xC0,
            Instruction::GenYield => 0xC1,
            Instruction::GenReturn => 0xC2,
            Instruction::ChanNew(_) => 0xD0,
            Instruction::ChanSend => 0xD1,
            Instruction::ChanRecv => 0xD2,
            Instruction::ChanClose => 0xD3,
            Instruction::MutexLock => 0xE0,
            Instruction::MutexUnlock => 0xE1,
            Instruction::RwLockRead => 0xE2,
            Instruction::RwLockWrite => 0xE3,
            Instruction::RwLockUnlock => 0xE4,
            Instruction::CondWait => 0xE5,
            Instruction::CondSignal => 0xE6,
            Instruction::CondBroadcast => 0xE7,
            Instruction::TypeOf => 0xF0,
            Instruction::SizeOf => 0xF1,
            Instruction::AlignOf => 0xF2,
            Instruction::Alloc(_) => 0xF3,
            Instruction::AllocArray(_) => 0xF4,
            Instruction::Free => 0xF5,
            Instruction::MemCopy => 0xF6,
            Instruction::MemMove => 0xF7,
            Instruction::MemSet => 0xF8,
            Instruction::FfiCall(_) => 0xF9,
            Instruction::DebugLine(_) => 0xFA,
            Instruction::DebugScopeEnter => 0xFB,
            Instruction::DebugScopeExit => 0xFC,
            Instruction::Breakpoint => 0xFD,
        }
    }
}

/// Serialize bytecode to binary format
pub fn serialize_bytecode(bytecode: &Bytecode) -> Vec<u8> {
    let mut buf = Vec::new();
    
    // Magic: "BHBX"
    buf.extend_from_slice(b"BHBX");
    
    // Version
    buf.extend_from_slice(&bytecode.version.to_le_bytes());
    buf.extend_from_slice(&bytecode.flags.to_le_bytes());
    
    // Constants count
    buf.extend_from_slice(&(bytecode.constants.len() as u32).to_le_bytes());
    for constant in &bytecode.constants {
        serialize_constant(&mut buf, constant);
    }
    
    // Functions count
    buf.extend_from_slice(&(bytecode.functions.len() as u32).to_le_bytes());
    for function in &bytecode.functions {
        serialize_function(&mut buf, function);
    }
    
    // Classes count
    buf.extend_from_slice(&(bytecode.classes.len() as u32).to_le_bytes());
    for class in &bytecode.classes {
        serialize_class(&mut buf, class);
    }
    
    // Globals count
    buf.extend_from_slice(&(bytecode.globals.len() as u32).to_le_bytes());
    for global in &bytecode.globals {
        serialize_global(&mut buf, global);
    }
    
    // Metadata
    buf.extend_from_slice(&(bytecode.metadata.len() as u32).to_le_bytes());
    for (k, v) in &bytecode.metadata {
        let k_bytes = k.as_bytes();
        let v_bytes = v.as_bytes();
        buf.extend_from_slice(&(k_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(k_bytes);
        buf.extend_from_slice(&(v_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(v_bytes);
    }
    
    // Entry point
    if let Some(ep) = bytecode.entry_point {
        buf.push(1);
        buf.extend_from_slice(&(ep as u32).to_le_bytes());
    } else {
        buf.push(0);
    }
    
    buf
}

fn serialize_constant(buf: &mut Vec<u8>, constant: &Constant) {
    match constant {
        Constant::Null => buf.push(0),
        Constant::Bool(b) => { buf.push(1); buf.push(*b as u8); }
        Constant::Int(i) => { buf.push(2); buf.extend_from_slice(&i.to_le_bytes()); }
        Constant::Float(f) => { buf.push(3); buf.extend_from_slice(&f.to_le_bytes()); }
        Constant::String(s) => { 
            buf.push(4); 
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        Constant::Bytes(b) => { 
            buf.push(5); 
            buf.extend_from_slice(&(b.len() as u32).to_le_bytes());
            buf.extend_from_slice(b);
        }
        Constant::Function(idx) => { buf.push(6); buf.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Constant::Type(idx) => { buf.push(7); buf.extend_from_slice(&(*idx as u32).to_le_bytes()); }
    }
}

fn serialize_function(buf: &mut Vec<u8>, function: &Function) {
    let name_bytes = function.name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(name_bytes);
    
    // Params
    buf.extend_from_slice(&(function.params.len() as u32).to_le_bytes());
    for param in &function.params {
        let name_bytes = param.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        // Simplified type serialization - would need full type serialization
    }
    
    // Locals
    buf.extend_from_slice(&(function.locals.len() as u32).to_le_bytes());
    for local in &function.locals {
        let name_bytes = local.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
    }
    
    // Instructions
    buf.extend_from_slice(&(function.instructions.len() as u32).to_le_bytes());
    for instr in &function.instructions {
        buf.push(instr.opcode());
        match instr {
            Instruction::PushInt(i) => buf.extend_from_slice(&i.to_le_bytes()),
            Instruction::PushFloat(f) => buf.extend_from_slice(&f.to_le_bytes()),
            Instruction::PushBool(b) => buf.push(*b as u8),
            Instruction::PushConst(idx) | Instruction::LoadLocal(idx) | Instruction::StoreLocal(idx)
            | Instruction::LoadArg(idx) | Instruction::StoreArg(idx)
            | Instruction::LoadGlobal(idx) | Instruction::StoreGlobal(idx)
            | Instruction::LoadField(idx) | Instruction::StoreField(idx)
            | Instruction::NewArray(idx) | Instruction::NewVector(idx)
            | Instruction::NewTuple(idx) | Instruction::TupleGet(idx) | Instruction::TupleSet(idx)
            | Instruction::NewObject(idx) | Instruction::NewStruct(idx) | Instruction::CheckCast(idx)
            | Instruction::Spawn(idx) | Instruction::SpawnClosure(idx) | Instruction::LoadCapture(idx)
            | Instruction::StoreCapture(idx) | Instruction::NewClosure(idx) | Instruction::ClosureCall(idx)
            | Instruction::ChanNew(idx) | Instruction::Alloc(idx) | Instruction::AllocArray(idx)
            | Instruction::FfiCall(idx) | Instruction::DebugLine(idx) => {
                buf.extend_from_slice(&(*idx as u32).to_le_bytes());
            }
            Instruction::Call(idx, argc) | Instruction::CallMethod(idx, argc)
            | Instruction::CallVirtual(idx, argc) | Instruction::CallInterface(idx, argc)
            | Instruction::TailCall(idx, argc) => {
                buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                buf.extend_from_slice(&(*argc as u32).to_le_bytes());
            }
            Instruction::LoadFieldStatic(class_idx, field_idx) | Instruction::StoreFieldStatic(class_idx, field_idx) => {
                buf.extend_from_slice(&(*class_idx as u32).to_le_bytes());
                buf.extend_from_slice(&(*field_idx as u32).to_le_bytes());
            }
            Instruction::Jump(idx) | Instruction::JumpIf(idx) | Instruction::JumpIfNot(idx) => {
                buf.extend_from_slice(&(*idx as u32).to_le_bytes());
            }
            Instruction::JumpTable(targets) => {
                buf.extend_from_slice(&(targets.len() as u32).to_le_bytes());
                for t in targets {
                    buf.extend_from_slice(&(*t as u32).to_le_bytes());
                }
            }
            _ => {}
        }
    }
    
    // Exception handlers
    buf.extend_from_slice(&(function.exception_handlers.len() as u32).to_le_bytes());
    for handler in &function.exception_handlers {
        buf.extend_from_slice(&(handler.try_start as u32).to_le_bytes());
        buf.extend_from_slice(&(handler.try_end as u32).to_le_bytes());
        buf.extend_from_slice(&(handler.catch_start as u32).to_le_bytes());
        if let Some(finally) = handler.finally_start {
            buf.push(1);
            buf.extend_from_slice(&(finally as u32).to_le_bytes());
        } else {
            buf.push(0);
        }
    }
    
    // Flags
    let mut flags = 0u8;
    if function.is_async { flags |= 1; }
    if function.is_generator { flags |= 2; }
    buf.push(flags);
}

fn serialize_class(buf: &mut Vec<u8>, class: &Class) {
    let name_bytes = class.name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(name_bytes);
    
    buf.extend_from_slice(&(class.fields.len() as u32).to_le_bytes());
    for field in &class.fields {
        let name_bytes = field.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.push(if field.is_pub { 1 } else { 0 });
        buf.push(if field.is_mut { 1 } else { 0 });
        buf.extend_from_slice(&(field.offset as u32).to_le_bytes());
    }
    
    buf.extend_from_slice(&(class.methods.len() as u32).to_le_bytes());
    for method in &class.methods {
        buf.extend_from_slice(&(*method as u32).to_le_bytes());
    }
    
    buf.push(if class.is_struct { 1 } else { 0 });
}

fn serialize_global(buf: &mut Vec<u8>, global: &Global) {
    let name_bytes = global.name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(name_bytes);
    buf.push(if global.is_mut { 1 } else { 0 });
    buf.push(if global.is_pub { 1 } else { 0 });
    if let Some(init) = global.init {
        buf.push(1);
        buf.extend_from_slice(&(init as u32).to_le_bytes());
    } else {
        buf.push(0);
    }
}

/// Deserialize bytecode from binary format
pub fn deserialize_bytecode(data: &[u8]) -> Result<Bytecode, DeserializeError> {
    let mut cursor = Cursor::new(data);
    
    // Check magic
    let magic = cursor.read_bytes(4)?;
    if magic != b"BHBX" {
        return Err(DeserializeError::InvalidMagic);
    }
    
    let version = cursor.read_u16()?;
    let flags = cursor.read_u16()?;
    
    let constants_count = cursor.read_u32()?;
    let mut constants = Vec::with_capacity(constants_count as usize);
    for _ in 0..constants_count {
        constants.push(deserialize_constant(&mut cursor)?);
    }
    
    let functions_count = cursor.read_u32()?;
    let mut functions = Vec::with_capacity(functions_count as usize);
    for _ in 0..functions_count {
        functions.push(deserialize_function(&mut cursor)?);
    }
    
    let classes_count = cursor.read_u32()?;
    let mut classes = Vec::with_capacity(classes_count as usize);
    for _ in 0..classes_count {
        classes.push(deserialize_class(&mut cursor)?);
    }
    
    let globals_count = cursor.read_u32()?;
    let mut globals = Vec::with_capacity(globals_count as usize);
    for _ in 0..globals_count {
        globals.push(deserialize_global(&mut cursor)?);
    }
    
    let metadata_count = cursor.read_u32()?;
    let mut metadata = HashMap::new();
    for _ in 0..metadata_count {
        let k_len = cursor.read_u32()?;
        let k = String::from_utf8(cursor.read_bytes(k_len as usize)?)?;
        let v_len = cursor.read_u32()?;
        let v = String::from_utf8(cursor.read_bytes(v_len as usize)?)?;
        metadata.insert(k, v);
    }
    
    let has_entry = cursor.read_u8()?;
    let entry_point = if has_entry == 1 {
        Some(cursor.read_u32()? as usize)
    } else {
        None
    };
    
    Ok(Bytecode {
        version,
        flags,
        constants,
        functions,
        classes,
        globals,
        metadata,
        entry_point,
    })
}

#[derive(Debug)]
pub enum DeserializeError {
    InvalidMagic,
    UnexpectedEof,
    InvalidUtf8,
    IoError,
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::InvalidMagic => write!(f, "Invalid magic bytes"),
            DeserializeError::UnexpectedEof => write!(f, "Unexpected end of file"),
            DeserializeError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            DeserializeError::IoError => write!(f, "IO error"),
        }
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    
    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], DeserializeError> {
        if self.pos + len > self.data.len() {
            return Err(DeserializeError::UnexpectedEof);
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }
    
    fn read_u8(&mut self) -> Result<u8, DeserializeError> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }
    
    fn read_u16(&mut self) -> Result<u16, DeserializeError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
    
    fn read_u32(&mut self) -> Result<u32, DeserializeError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    
    fn read_u64(&mut self) -> Result<u64, DeserializeError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
    
    fn read_i64(&mut self) -> Result<i64, DeserializeError> {
        Ok(self.read_u64()? as i64)
    }
    
    fn read_f64(&mut self) -> Result<f64, DeserializeError> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

fn deserialize_constant(cursor: &mut Cursor) -> Result<Constant, DeserializeError> {
    let tag = cursor.read_u8()?;
    match tag {
        0 => Ok(Constant::Null),
        1 => Ok(Constant::Bool(cursor.read_u8()? != 0)),
        2 => Ok(Constant::Int(cursor.read_i64()?)),
        3 => Ok(Constant::Float(cursor.read_f64()?)),
        4 => {
            let len = cursor.read_u32()?;
            let bytes = cursor.read_bytes(len as usize)?;
            Ok(Constant::String(String::from_utf8(bytes.to_vec()).map_err(|_| DeserializeError::InvalidUtf8)?))
        }
        5 => {
            let len = cursor.read_u32()?;
            Ok(Constant::Bytes(cursor.read_bytes(len as usize)?.to_vec()))
        }
        6 => Ok(Constant::Function(cursor.read_u32()? as usize)),
        7 => Ok(Constant::Type(cursor.read_u32()? as usize)),
        _ => Err(DeserializeError::InvalidUtf8), // Reusing error type
    }
}

fn deserialize_function(cursor: &mut Cursor) -> Result<Function, DeserializeError> {
    let name_len = cursor.read_u32()?;
    let name = String::from_utf8(cursor.read_bytes(name_len as usize)?.to_vec())
        .map_err(|_| DeserializeError::InvalidUtf8)?;
    
    // Simplified - would need full deserialization
    Ok(Function {
        name,
        params: Vec::new(),
        return_type: TypeInfo::Primitive(PrimitiveType::Unit),
        locals: Vec::new(),
        instructions: Vec::new(),
        exception_handlers: Vec::new(),
        is_async: false,
        is_generator: false,
        span: Span::new(0, 0, 1, 1),
    })
}

fn deserialize_class(cursor: &mut Cursor) -> Result<Class, DeserializeError> {
    let name_len = cursor.read_u32()?;
    let name = String::from_utf8(cursor.read_bytes(name_len as usize)?.to_vec())
        .map_err(|_| DeserializeError::InvalidUtf8)?;
    
    Ok(Class {
        name,
        fields: Vec::new(),
        methods: Vec::new(),
        is_struct: true,
        span: Span::new(0, 0, 1, 1),
    })
}

fn deserialize_global(cursor: &mut Cursor) -> Result<Global, DeserializeError> {
    let name_len = cursor.read_u32()?;
    let name = String::from_utf8(cursor.read_bytes(name_len as usize)?.to_vec())
        .map_err(|_| DeserializeError::InvalidUtf8)?;
    
    Ok(Global {
        name,
        ty: TypeInfo::Primitive(PrimitiveType::Unit),
        is_mut: false,
        is_pub: false,
        init: None,
        span: Span::new(0, 0, 1, 1),
    })
}

use std::collections::HashMap;