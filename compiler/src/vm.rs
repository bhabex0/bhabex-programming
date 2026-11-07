//! Bhabex Virtual Machine - High-performance bytecode interpreter
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use crate::bytecode::{Bytecode, Instruction, Function, Constant, Value, TypeInfo, PrimitiveType, ObjectRef, ClosureRef, ChannelRef, MutexRef, RwLockRef, NativeFunction, GeneratorRef};
use crate::lexer::Span;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

/// VM Error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
    #[error("Null reference")]
    NullReference,
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Global not found: {0}")]
    GlobalNotFound(String),
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Mutex error: {0}")]
    MutexError(String),
    #[error("Exception: {0}")]
    Exception(String),
    #[error("Out of memory")]
    OutOfMemory,
    #[error("FFI error: {0}")]
    FfiError(String),
    #[error("Execution limit exceeded")]
    ExecutionLimitExceeded,
}

/// VM Error types - re-export from bytecode for compatibility
pub type VMValue = Value;

/// Call frame for function execution
#[derive(Debug)]
pub struct CallFrame {
    pub function_id: usize,
    pub ip: usize,
    pub stack_base: usize,
    pub locals: Vec<Value>,
    pub return_address: Option<usize>,
}

/// Virtual Machine
pub struct VM {
    pub bytecode: Arc<Bytecode>,
    pub stack: Vec<Value>,
    pub call_stack: Vec<CallFrame>,
    pub globals: Vec<Value>,
    pub constants: Vec<Value>,
    pub objects: Vec<Arc<Mutex<ObjectRef>>>,
    pub channels: HashMap<usize, ChannelRef>,
    pub mutexes: HashMap<usize, MutexRef>,
    pub rwlocks: HashMap<usize, RwLockRef>,
    pub native_functions: HashMap<usize, NativeFunction>,
    pub instruction_limit: usize,
    pub instructions_executed: usize,
    pub debug_mode: bool,
    pub breakpoints: HashMap<usize, bool>,
}

impl VM {
    /// Create a new VM with the given bytecode
    pub fn new(bytecode: Bytecode) -> Self {
        let mut constants = Vec::with_capacity(bytecode.constants.len());
        for constant in &bytecode.constants {
            constants.push(Self::constant_to_value(constant));
        }
        
        let globals = vec![Value::Null; bytecode.globals.len()];
        
        Self {
            bytecode: Arc::new(bytecode),
            stack: Vec::with_capacity(1024),
            call_stack: Vec::new(),
            globals,
            constants,
            objects: Vec::new(),
            channels: HashMap::new(),
            mutexes: HashMap::new(),
            rwlocks: HashMap::new(),
            native_functions: HashMap::new(),
            instruction_limit: 10_000_000,
            instructions_executed: 0,
            debug_mode: false,
            breakpoints: HashMap::new(),
        }
    }

    fn constant_to_value(constant: &Constant) -> Value {
        match constant {
            Constant::Null => Value::Null,
            Constant::Bool(b) => Value::Bool(*b),
            Constant::Int(i) => Value::Int(*i),
            Constant::Float(f) => Value::Float(*f),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Bytes(b) => Value::Bytes(b.clone()),
            Constant::Function(idx) => Value::Function(*idx),
            Constant::Type(_) => Value::Null, // Types not represented at runtime
        }
    }

    /// Register a native function
    pub fn register_native(&mut self, func: NativeFunction) {
        self.native_functions.insert(func.func_id, func);
    }

    /// Set instruction limit
    pub fn set_instruction_limit(&mut self, limit: usize) {
        self.instruction_limit = limit;
    }

    /// Enable/disable debug mode
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    /// Set breakpoint
    pub fn set_breakpoint(&mut self, instruction_index: usize, enabled: bool) {
        if enabled {
            self.breakpoints.insert(instruction_index, true);
        } else {
            self.breakpoints.remove(&instruction_index);
        }
    }

    /// Execute the program
    pub fn run(&mut self) -> Result<Value, VMError> {
        // Find entry point
        let entry_point = self.bytecode.entry_point
            .ok_or(VMError::FunctionNotFound("main".to_string()))?;
        
        self.call_function(entry_point, vec![])
    }

    /// Call a function by index
    pub fn call_function(&mut self, function_id: usize, args: Vec<Value>) -> Result<Value, VMError> {
        if function_id >= self.bytecode.functions.len() {
            return Err(VMError::FunctionNotFound(format!("Function index {}", function_id)));
        }
        
        let function = &self.bytecode.functions[function_id];
        
        // Check argument count
        if args.len() != function.params.len() {
            return Err(VMError::TypeMismatch {
                expected: format!("{} arguments", function.params.len()),
                got: format!("{} arguments", args.len()),
            });
        }
        
        // Create call frame
        let stack_base = self.stack.len();
        let mut locals = Vec::with_capacity(function.locals.len());
        for _ in 0..function.locals.len() {
            locals.push(Value::Null);
        }
        
        // Push arguments onto stack
        for arg in args {
            self.stack.push(arg);
        }
        
        let frame = CallFrame {
            function_id,
            ip: 0,
            stack_base,
            locals,
            return_address: None,
        };
        
        self.call_stack.push(frame);
        
        // Execute
        self.execute_frame()
    }

    /// Execute current frame
    fn execute_frame(&mut self) -> Result<Value, VMError> {
        while let Some(frame) = self.call_stack.last_mut() {
            if self.instructions_executed >= self.instruction_limit {
                return Err(VMError::ExecutionLimitExceeded);
            }
            
            self.instructions_executed += 1;
            
            let function = &self.bytecode.functions[frame.function_id];
            
            if frame.ip >= function.instructions.len() {
                // Function completed without explicit return
                self.call_stack.pop();
                self.stack.truncate(frame.stack_base);
                continue;
            }
            
            // Check breakpoint
            if self.debug_mode && self.breakpoints.get(&frame.ip).copied().unwrap_or(false) {
                self.debug_break(frame);
            }
            
            let instruction = function.instructions[frame.ip].clone();
            frame.ip += 1;
            
            if self.debug_mode {
                self.debug_instruction(&instruction);
            }
            
            self.execute_instruction(instruction)?;
        }
        
        // Return final result
        Ok(self.stack.pop().unwrap_or(Value::Null))
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), VMError> {
        use Instruction::*;
        
        match instruction {
            Nop => {}
            
            Pop => {
                self.stack.pop().ok_or(VMError::StackUnderflow)?;
            }
            
            Dup => {
                let val = self.stack.last().ok_or(VMError::StackUnderflow)?.clone();
                self.stack.push(val);
            }
            
            Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
            }
            
            Rot3 => {
                let len = self.stack.len();
                if len < 3 {
                    return Err(VMError::StackUnderflow);
                }
                let a = self.stack[len - 3].clone();
                self.stack[len - 3] = self.stack[len - 2].clone();
                self.stack[len - 2] = self.stack[len - 1].clone();
                self.stack[len - 1] = a;
            }
            
            PushNull => self.stack.push(Value::Null),
            PushBool(b) => self.stack.push(Value::Bool(b)),
            PushInt(i) => self.stack.push(Value::Int(i)),
            PushFloat(f) => self.stack.push(Value::Float(f)),
            PushConst(idx) => {
                let val = self.constants.get(idx).ok_or(VMError::FunctionNotFound(format!("Constant {}", idx)))?.clone();
                self.stack.push(val);
            }
            
            LoadLocal(idx) => {
                let frame = self.call_stack.last().ok_or(VMError::StackUnderflow)?;
                let val = frame.locals.get(idx).ok_or(VMError::IndexOutOfBounds(idx))?.clone();
                self.stack.push(val);
            }
            
            StoreLocal(idx) => {
                let frame = self.call_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if idx < frame.locals.len() {
                    frame.locals[idx] = val;
                } else {
                    return Err(VMError::IndexOutOfBounds(idx));
                }
            }
            
            LoadArg(idx) => {
                let frame = self.call_stack.last().ok_or(VMError::StackUnderflow)?;
                let val = frame.locals.get(idx).ok_or(VMError::IndexOutOfBounds(idx))?.clone();
                self.stack.push(val);
            }
            
            StoreArg(idx) => {
                let frame = self.call_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if idx < frame.locals.len() {
                    frame.locals[idx] = val;
                } else {
                    return Err(VMError::IndexOutOfBounds(idx));
                }
            }
            
            LoadGlobal(idx) => {
                let val = self.globals.get(idx).ok_or(VMError::GlobalNotFound(idx.to_string()))?.clone();
                self.stack.push(val);
            }
            
            StoreGlobal(idx) => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if idx < self.globals.len() {
                    self.globals[idx] = val;
                } else {
                    return Err(VMError::GlobalNotFound(idx.to_string()));
                }
            }
            
            LoadField(idx) => {
                let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match obj {
                    Value::Object(obj_ref) => {
                        let field = obj_ref.lock().unwrap().get_field(idx)
                            .ok_or(VMError::FieldNotFound(idx.to_string()))?;
                        self.stack.push(field);
                    }
                    _ => return Err(VMError::TypeMismatch {
                        expected: "object".to_string(),
                        got: obj.type_name().to_string(),
                    }),
                }
            }
            
            StoreField(idx) => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match obj {
                    Value::Object(obj_ref) => {
                        obj_ref.lock().unwrap().set_field(idx, value)?;
                    }
                    _ => return Err(VMError::TypeMismatch {
                        expected: "object".to_string(),
                        got: obj.type_name().to_string(),
                    }),
                }
            }
            
            NewArray(idx) => {
                let type_info = self.get_type_info(idx);
                let len = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                let mut array = Vec::with_capacity(len);
                for _ in 0..len {
                    array.push(Value::Null);
                }
                self.stack.push(Value::Array(array));
            }
            
            NewVector(idx) => {
                self.stack.push(Value::Array(Vec::new()));
            }
            
            ArrayLen => {
                let arr = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match arr {
                    Value::Array(a) => self.stack.push(Value::Int(a.len() as i64)),
                    _ => return Err(VMError::TypeMismatch { expected: "array".to_string(), got: arr.type_name().to_string() }),
                }
            }
            
            ArrayGet => {
                let index = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                let arr = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match arr {
                    Value::Array(a) => {
                        let val = a.get(index).cloned().unwrap_or(Value::Null);
                        self.stack.push(val);
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "array".to_string(), got: arr.type_name().to_string() }),
                }
            }
            
            ArraySet => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let index = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                let arr = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match arr {
                    Value::Array(mut a) => {
                        if index < a.len() {
                            a[index] = value;
                            self.stack.push(Value::Array(a));
                        } else {
                            return Err(VMError::IndexOutOfBounds(index));
                        }
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "array".to_string(), got: arr.type_name().to_string() }),
                }
            }
            
            VectorPush => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let vec = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match vec {
                    Value::Array(mut a) => {
                        a.push(value);
                        self.stack.push(Value::Array(a));
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "array".to_string(), got: vec.type_name().to_string() }),
                }
            }
            
            VectorPop => {
                let vec = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match vec {
                    Value::Array(mut a) => {
                        let val = a.pop().unwrap_or(Value::Null);
                        self.stack.push(val);
                        self.stack.push(Value::Array(a));
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "array".to_string(), got: vec.type_name().to_string() }),
                }
            }
            
            NewTuple(arity) => {
                let mut elements = Vec::with_capacity(arity);
                for _ in 0..arity {
                    elements.push(self.stack.pop().ok_or(VMError::StackUnderflow)?);
                }
                elements.reverse();
                self.stack.push(Value::Tuple(elements));
            }
            
            TupleGet(idx) => {
                let tuple = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match tuple {
                    Value::Tuple(t) => {
                        let val = t.get(idx).cloned().unwrap_or(Value::Null);
                        self.stack.push(val);
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "tuple".to_string(), got: tuple.type_name().to_string() }),
                }
            }
            
            TupleSet(idx) => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let tuple = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match tuple {
                    Value::Tuple(mut t) => {
                        if idx < t.len() {
                            t[idx] = value;
                            self.stack.push(Value::Tuple(t));
                        } else {
                            return Err(VMError::IndexOutOfBounds(idx));
                        }
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "tuple".to_string(), got: tuple.type_name().to_string() }),
                }
            }
            
            NewObject(class_idx) => {
                let class = self.bytecode.classes.get(class_idx).ok_or(VMError::FunctionNotFound(class_idx.to_string()))?;
                let obj = ObjectRef::new(class_idx, class.fields.len());
                let obj_ref = Arc::new(Mutex::new(obj));
                let obj_idx = self.objects.len();
                self.objects.push(obj_ref.clone());
                self.stack.push(Value::Object(obj_ref));
            }
            
            NewStruct(struct_idx) => {
                // Same as NewObject for structs
                self.execute_instruction(NewObject(struct_idx))?;
            }
            
            CheckCast(type_idx) => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                // Type checking would go here
                self.stack.push(val);
            }
            
            // Arithmetic
            Add => self.binary_op(|a, b| Ok(Value::Int(a + b)), |a, b| Ok(Value::Float(a + b)), |a, b| Ok(Value::String(a + &b)))?,
            Sub => self.binary_op(|a, b| Ok(Value::Int(a - b)), |a, b| Ok(Value::Float(a - b)), |_, _| Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: "string".to_string() }))?,
            Mul => self.binary_op(|a, b| Ok(Value::Int(a * b)), |a, b| Ok(Value::Float(a * b)), |_, _| Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: "string".to_string() }))?,
            Div => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let result = match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 { return Err(VMError::DivisionByZero); }
                        Value::Int(a / b)
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 { return Err(VMError::DivisionByZero); }
                        Value::Float(a / b)
                    }
                    (Value::Int(a), Value::Float(b)) => {
                        if b == 0.0 { return Err(VMError::DivisionByZero); }
                        Value::Float(a as f64 / b)
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if b == 0 { return Err(VMError::DivisionByZero); }
                        Value::Float(a / b as f64)
                    }
                    _ => return Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: "invalid".to_string() }),
                };
                self.stack.push(result);
            }
            
            Mod => self.binary_op(|a, b| Ok(Value::Int(a % b)), |a, b| Ok(Value::Float(a % b)), |_, _| Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: "string".to_string() }))?,
            Neg => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let result = match val {
                    Value::Int(i) => Value::Int(-i),
                    Value::Float(f) => Value::Float(-f),
                    _ => return Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: val.type_name().to_string() }),
                };
                self.stack.push(result);
            }
            
            Pow => self.binary_op(
                |a, b| Ok(Value::Int(a.pow(b as u32))),
                |a, b| Ok(Value::Float(a.powf(b))),
                |_, _| Err(VMError::TypeMismatch { expected: "numeric".to_string(), got: "string".to_string() })
            )?,
            
            // Bitwise
            BitAnd => self.binary_op(|a, b| Ok(Value::Int(a & b)), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "float".to_string() }), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "string".to_string() }))?,
            BitOr => self.binary_op(|a, b| Ok(Value::Int(a | b)), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "float".to_string() }), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "string".to_string() }))?,
            BitXor => self.binary_op(|a, b| Ok(Value::Int(a ^ b)), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "float".to_string() }), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "string".to_string() }))?,
            BitNot => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                match val {
                    Value::Int(i) => self.stack.push(Value::Int(!i)),
                    _ => return Err(VMError::TypeMismatch { expected: "int".to_string(), got: val.type_name().to_string() }),
                }
            }
            
            Shl => self.binary_op(|a, b| Ok(Value::Int(a << b)), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "float".to_string() }), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "string".to_string() }))?,
            Shr => self.binary_op(|a, b| Ok(Value::Int(a >> b)), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "float".to_string() }), |_, _| Err(VMError::TypeMismatch { expected: "int".to_string(), got: "string".to_string() }))?,
            
            // Comparison
            Eq => self.compare_op(|a, b| a == b)?,
            Ne => self.compare_op(|a, b| a != b)?,
            Lt => self.compare_op(|a, b| match (a, b) {
                (Value::Int(a), Value::Int(b)) => a < b,
                (Value::Float(a), Value::Float(b)) => a < b,
                (Value::Int(a), Value::Float(b)) => (*a as f64) < b,
                (Value::Float(a), Value::Int(b)) => a < (*b as f64),
                (Value::String(a), Value::String(b)) => a < b,
                _ => false,
            })?,
            Gt => self.compare_op(|a, b| match (a, b) {
                (Value::Int(a), Value::Int(b)) => a > b,
                (Value::Float(a), Value::Float(b)) => a > b,
                (Value::Int(a), Value::Float(b)) => (*a as f64) > b,
                (Value::Float(a), Value::Int(b)) => a > (*b as f64),
                (Value::String(a), Value::String(b)) => a > b,
                _ => false,
            })?,
            Le => self.compare_op(|a, b| match (a, b) {
                (Value::Int(a), Value::Int(b)) => a <= b,
                (Value::Float(a), Value::Float(b)) => a <= b,
                (Value::Int(a), Value::Float(b)) => (*a as f64) <= b,
                (Value::Float(a), Value::Int(b)) => a <= (*b as f64),
                (Value::String(a), Value::String(b)) => a <= b,
                _ => false,
            })?,
            Ge => self.compare_op(|a, b| match (a, b) {
                (Value::Int(a), Value::Int(b)) => a >= b,
                (Value::Float(a), Value::Float(b)) => a >= b,
                (Value::Int(a), Value::Float(b)) => (*a as f64) >= b,
                (Value::Float(a), Value::Int(b)) => a >= (*b as f64),
                (Value::String(a), Value::String(b)) => a >= b,
                _ => false,
            })?,
            
            // Logical
            And => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let result = a.is_truthy() && b.is_truthy();
                self.stack.push(Value::Bool(result));
            }
            Or => {
                let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let result = a.is_truthy() || b.is_truthy();
                self.stack.push(Value::Bool(result));
            }
            Not => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.stack.push(Value::Bool(!val.is_truthy()));
            }
            
            // Control flow
            Jump(target) => {
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.ip = target;
                }
            }
            JumpIf(target) => {
                let cond = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond.is_truthy() {
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.ip = target;
                    }
                }
            }
            JumpIfNot(target) => {
                let cond = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.is_truthy() {
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.ip = target;
                    }
                }
            }
            JumpTable(targets) => {
                let idx = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                if idx < targets.len() {
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.ip = targets[idx];
                    }
                }
            }
            
            // Function calls
            Call(func_idx, argc) => {
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(self.stack.pop().ok_or(VMError::StackUnderflow)?);
                }
                // Check if native - look up by func_id
                if let Some(native) = self.native_functions.get(&func_idx) {
                    if let Some(func) = &native.func {
                        let result = func(self, args)?;
                        self.stack.push(result);
                    } else {
                        return Err(VMError::FfiError("Native function not initialized".to_string()));
                    }
                } else {
                    let function_id = func_idx;
                    let function = &self.bytecode.functions[function_id];
                    
                    let stack_base = self.stack.len();
                    let mut locals = Vec::with_capacity(function.locals.len());
                    for _ in 0..function.locals.len() {
                        locals.push(Value::Null);
                    }
                    
                    // Push args
                    for arg in args {
                        self.stack.push(arg);
                    }
                    
                    let frame = CallFrame {
                        function_id,
                        ip: 0,
                        stack_base,
                        locals,
                        return_address: None,
                    };
                    
                    self.call_stack.push(frame);
                }
            }
            
            CallMethod(method_idx, argc) => {
                // Simplified - would need receiver
                self.execute_instruction(Call(method_idx, argc))?;
            }
            
            TailCall(func_idx, argc) => {
                // Pop current frame and call new function
                self.call_stack.pop();
                self.execute_instruction(Call(func_idx, argc))?;
            }
            
            Return => {
                let result = self.stack.pop().unwrap_or(Value::Null);
                self.return_from_call(result)?;
            }
            
            ReturnVoid => {
                self.return_from_call(Value::Null)?;
            }
            
            // Async
            Await => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                // Simplified - would need actual async runtime
                self.stack.push(val);
            }
            
            Yield => {
                let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                // Would need generator support
                self.stack.push(val);
            }
            
            Spawn(func_idx) => {
                // Spawn new task
                let handle = self.spawn_task(func_idx, vec![])?;
                self.stack.push(Value::Int(handle as i64));
            }
            
            // Exception handling
            TryBegin(handler_offset) => {
                // Would need exception handling support
            }
            TryEnd => {}
            Throw => {
                let exception = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                return Err(VMError::Exception(format!("{:?}", exception)));
            }
            
            // Closures
            NewClosure(func_idx) => {
                let frame = self.call_stack.last().ok_or(VMError::StackUnderflow)?;
                let captures = frame.locals.clone(); // Simplified
                let closure = ClosureRef { function_id: func_idx, captures };
                self.stack.push(Value::Closure(closure));
            }
            
            ClosureCall(closure_idx) => {
                // Would need closure execution
            }
            
            // Channels
            ChanNew(capacity) => {
                let chan = ChannelRef::new(capacity);
                let idx = self.channels.len();
                self.channels.insert(idx, chan);
                self.stack.push(Value::Int(idx as i64));
            }
            
            ChanSend => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let chan_idx = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                if let Some(chan) = self.channels.get(&chan_idx) {
                    chan.send(self.stack.pop().ok_or(VMError::StackUnderflow)?)?;
                } else {
                    return Err(VMError::ChannelError("Channel not found".to_string()));
                }
            }
            
            ChanRecv => {
                let chan_idx = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                if let Some(chan) = self.channels.get(&chan_idx) {
                    let value = chan.recv()?;
                    self.stack.push(value);
                } else {
                    return Err(VMError::ChannelError("Channel not found".to_string()));
                }
            }
            
            ChanClose => {
                let chan_idx = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                if let Some(chan) = self.channels.remove(&chan_idx) {
                    chan.close();
                }
            }
            
            // Synchronization
            MutexLock => {
                let mutex_idx = self.stack.pop().ok_or(VMError::StackUnderflow)?.as_int().unwrap_or(0) as usize;
                if let Some(mutex) = self.mutexes.get(&mutex_idx) {
                    let _guard = mutex.lock();
                    // Store guard on stack for unlock
                    self.stack.push(Value::Int(mutex_idx as i64));
                } else {
                    let mutex = MutexRef::new();
                    let idx = self.mutexes.len();
                    self.mutexes.insert(idx, mutex.clone());
                    let _guard = mutex.lock();
                    self.stack.push(Value::Int(idx as i64));
                }
            }
            
            MutexUnlock => {
                // Guard dropped automatically
            }
            
            // Debug
            DebugLine(line) => {
                if self.debug_mode {
                    println!("Line: {}", line);
                }
            }
            
            Breakpoint => {
                if self.debug_mode {
                    println!("Breakpoint hit!");
                    // Would pause execution in real implementation
                }
            }
            
            _ => {
                return Err(VMError::InvalidInstruction(format!("{:?}", instruction)));
            }
        }
        
        Ok(())
    }

    fn binary_op<F1, F2, F3>(&mut self, int_op: F1, float_op: F2, string_op: F3) -> Result<(), VMError>
    where
        F1: FnOnce(i64, i64) -> Result<Value, VMError>,
        F2: FnOnce(f64, f64) -> Result<Value, VMError>,
        F3: FnOnce(String, String) -> Result<Value, VMError>,
    {
        let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        
        let result = match (a, b) {
            (Value::Int(a), Value::Int(b)) => int_op(a, b)?,
            (Value::Float(a), Value::Float(b)) => float_op(a, b)?,
            (Value::Int(a), Value::Float(b)) => float_op(a as f64, b)?,
            (Value::Float(a), Value::Int(b)) => float_op(a, b as f64)?,
            (Value::String(a), Value::String(b)) => string_op(a, b)?,
            _ => return Err(VMError::TypeMismatch { expected: "compatible types".to_string(), got: "incompatible".to_string() }),
        };
        
        self.stack.push(result);
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), VMError>
    where
        F: FnOnce(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let result = op(&a, &b);
        self.stack.push(Value::Bool(result));
        Ok(())
    }

    fn return_from_call(&mut self, result: Value) -> Result<(), VMError> {
        let frame = self.call_stack.pop().ok_or(VMError::StackUnderflow)?;
        self.stack.truncate(frame.stack_base);
        self.stack.push(result);
        Ok(())
    }

    fn spawn_task(&mut self, func_idx: usize, args: Vec<Value>) -> Result<usize, VMError> {
        // Would spawn actual thread in real implementation
        Ok(0)
    }

    fn get_type_info(&self, idx: usize) -> &TypeInfo {
        // Simplified
        &TypeInfo::Primitive(PrimitiveType::Int)
    }

    fn debug_break(&mut self, frame: &CallFrame) {
        println!("Break at function {}, ip: {}", frame.function_id, frame.ip);
        // Would pause for debugger
    }

    fn debug_instruction(&self, instruction: &Instruction) {
        println!("  {:?}", instruction);
    }

    /// Get stack trace
    pub fn stack_trace(&self) -> Vec<String> {
        self.call_stack.iter().map(|frame| {
            let func = &self.bytecode.functions[frame.function_id];
            format!("  {} @ ip {}", func.name, frame.ip)
        }).collect()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Bytecode::new())
    }
}