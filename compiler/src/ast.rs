//! Abstract Syntax Tree for Bhabex
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use std::fmt;
use crate::lexer::Span;
use crate::bytecode::{PrimitiveType, FunctionType as BytecodeFunctionType};
use indexmap::IndexMap;

/// Program - root of the AST
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

impl Program {
    pub fn new(items: Vec<Item>, span: Span) -> Self {
        Self { items, span }
    }
}

/// Top-level items
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    TypeAlias(TypeAlias),
    Const(Const),
    Static(Static),
    Module(Module),
    Import(Import),
    Export(Export),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Function(f) => f.span,
            Item::Struct(s) => s.span,
            Item::Enum(e) => e.span,
            Item::Trait(t) => t.span,
            Item::Impl(i) => i.span,
            Item::TypeAlias(t) => t.span,
            Item::Const(c) => c.span,
            Item::Static(s) => s.span,
            Item::Module(m) => m.span,
            Item::Import(i) => i.span,
            Item::Export(e) => e.span,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Item::Function(f) => Some(&f.name),
            Item::Struct(s) => Some(&s.name),
            Item::Enum(e) => Some(&e.name),
            Item::Trait(t) => Some(&t.name),
            Item::TypeAlias(t) => Some(&t.name),
            Item::Const(c) => Some(&c.name),
            Item::Static(s) => Some(&s.name),
            Item::Module(m) => Some(&m.name),
            _ => None,
        }
    }
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_async: bool,
    pub is_pub: bool,
    pub is_unsafe: bool,
    pub is_extern: bool,
    pub abi: Option<String>,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Generic type parameter
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub is_mut: bool,
    pub is_ref: bool,
    pub is_move: bool,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, span: Span) -> Self {
        Self { stmts, span }
    }
}

/// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    Expr(ExprStmt),
    Return(ReturnStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Loop(LoopStmt),
    Match(MatchStmt),
    Block(Block),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Assign(AssignStmt),
    Try(TryStmt),
    Spawn(SpawnStmt),
    Defer(DeferStmt),
    Label(LabelStmt),
    Goto(GotoStmt),
    Unsafe(UnsafeStmt),
}

/// Let binding
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub ty: Option<Type>,
    pub is_mut: bool,
    pub is_ref: bool,
    pub init: Option<Expr>,
    pub span: Span,
}

/// Expression statement
#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Return statement
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// If statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iterator: Expr,
    pub body: Box<Stmt>,
    pub span: Span,
}

/// Infinite loop
#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    pub body: Box<Stmt>,
    pub span: Span,
}

/// Match expression/statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub expr: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

/// Break statement
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStmt {
    pub label: Option<String>,
    pub value: Option<Expr>,
    pub span: Span,
}

/// Continue statement
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStmt {
    pub label: Option<String>,
    pub span: Span,
}

/// Assignment
#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    pub target: Expr,
    pub op: Option<AssignOp>,
    pub value: Expr,
    pub span: Span,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Add, Sub, Mul, Div, Mod, Pow,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

/// Try-catch statement
#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally: Option<Block>,
    pub span: Span,
}

/// Catch clause
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub pattern: Pattern,
    pub body: Block,
    pub span: Span,
}

/// Spawn statement (async task)
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Defer statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeferStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Label statement
#[derive(Debug, Clone, PartialEq)]
pub struct LabelStmt {
    pub name: String,
    pub stmt: Box<Stmt>,
    pub span: Span,
}

/// Goto statement
#[derive(Debug, Clone, PartialEq)]
pub struct GotoStmt {
    pub label: String,
    pub span: Span,
}

/// Unsafe block
#[derive(Debug, Clone, PartialEq)]
pub struct UnsafeStmt {
    pub body: Block,
    pub span: Span,
}

/// Patterns for destructuring
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Span),
    Ident(String, Span),
    Literal(Literal, Span),
    Struct(String, Vec<(String, Pattern)>, Span),
    Tuple(Vec<Pattern>, Span),
    Slice(Vec<Pattern>, Option<Box<Pattern>>, Span),
    Range(Box<Pattern>, Box<Pattern>, bool, Span),
    Or(Vec<Pattern>, Span),
    Binding(String, Box<Pattern>, Span),
    Ref(Box<Pattern>, Span),
    Deref(Box<Pattern>, Span),
}

/// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    Field(FieldExpr),
    Index(IndexExpr),
    Range(RangeExpr),
    Cast(CastExpr),
    If(IfExpr),
    Match(MatchExpr),
    Block(Block),
    Closure(ClosureExpr),
    Async(AsyncExpr),
    Await(AwaitExpr),
    Try(TryExpr),
    Throw(ThrowExpr),
    Spawn(SpawnExpr),
    Struct(StructExpr),
    Array(ArrayExpr),
    Tuple(Vec<Expr>, Span),
    Slice(SliceExpr),
    Macro(MacroExpr),
    Parenthesized(Box<Expr>, Span),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    Unit,
    Null,
}

impl Literal {
    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::Int(_) => "int",
            Literal::Float(_) => "float",
            Literal::Bool(_) => "bool",
            Literal::Char(_) => "char",
            Literal::String(_) => "string",
            Literal::Bytes(_) => "bytes",
            Literal::Unit => "()",
            Literal::Null => "null",
        }
    }
}

/// Binary expression
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
}

/// Unary expression
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, Not, BitNot, Deref, Ref, RefMut, Move,
}

/// Function call
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Method call
#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub receiver: Box<Expr>,
    pub method: String,
    pub type_args: Vec<Type>,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Field access
#[derive(Debug, Clone, PartialEq)]
pub struct FieldExpr {
    pub base: Box<Expr>,
    pub field: String,
    pub span: Span,
}

/// Index access
#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

/// Range expression
#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
    pub span: Span,
}

/// Type cast
#[derive(Debug, Clone, PartialEq)]
pub struct CastExpr {
    pub expr: Box<Expr>,
    pub ty: Type,
    pub span: Span,
}

/// If expression
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Option<Box<Expr>>,
    pub span: Span,
}

/// Match expression
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub expr: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Closure expression
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureExpr {
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Expr>,
    pub is_async: bool,
    pub is_move: bool,
    pub captures: Vec<String>,
    pub span: Span,
}

/// Async expression
#[derive(Debug, Clone, PartialEq)]
pub struct AsyncExpr {
    pub body: Box<Expr>,
    pub span: Span,
}

/// Await expression
#[derive(Debug, Clone, PartialEq)]
pub struct AwaitExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Try expression
#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Throw expression
#[derive(Debug, Clone, PartialEq)]
pub struct ThrowExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Spawn expression
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Struct literal
#[derive(Debug, Clone, PartialEq)]
pub struct StructExpr {
    pub name: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

/// Array literal
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

/// Slice expression
#[derive(Debug, Clone, PartialEq)]
pub struct SliceExpr {
    pub base: Box<Expr>,
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub span: Span,
}

/// Macro invocation
#[derive(Debug, Clone, PartialEq)]
pub struct MacroExpr {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Named(String, Vec<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>, Option<Box<Expr>>),
    Slice(Box<Type>),
    Ref(Box<Type>, bool),
    RawPtr(Box<Type>, bool),
    Function(FunctionType),
    ImplTrait(Vec<Type>),
    DynTrait(Vec<Type>),
    Generic(String),
    Infer,
    Never,
    Paren(Box<Type>),
}

/// Function type
pub type FunctionType = crate::bytecode::FunctionType;

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub methods: Vec<Function>,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
    pub is_mut: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<EnumField>,
    pub discriminant: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Enum field (tuple or struct style)
#[derive(Debug, Clone, PartialEq)]
pub enum EnumField {
    Tuple(Type),
    Named(String, Type),
}

/// Trait definition
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub bounds: Vec<Type>,
    pub items: Vec<TraitItem>,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Trait item
#[derive(Debug, Clone, PartialEq)]
pub enum TraitItem {
    Function(Function),
    Type(TypeAlias),
    Const(Const),
}

/// Impl block
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub generics: Vec<GenericParam>,
    pub trait_ref: Option<TraitRef>,
    pub target_type: Type,
    pub items: Vec<ImplItem>,
    pub span: Span,
}

/// Impl item
#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Function(Function),
    Type(TypeAlias),
    Const(Const),
}

/// Trait reference
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub path: String,
    pub args: Vec<Type>,
    pub span: Span,
}

/// Type alias
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Constant
#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Static variable
#[derive(Debug, Clone, PartialEq)]
pub struct Static {
    pub name: String,
    pub ty: Type,
    pub value: Option<Expr>,
    pub is_mut: bool,
    pub is_pub: bool,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

/// Module
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub is_pub: bool,
    pub span: Span,
}

/// Import
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub items: Vec<ImportItem>,
    pub is_pub: bool,
    pub span: Span,
}

/// Import item
#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub span: Span,
}

/// Export
#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    pub items: Vec<ExportItem>,
    pub span: Span,
}

/// Export item
#[derive(Debug, Clone, PartialEq)]
pub struct ExportItem {
    pub name: String,
    pub alias: Option<String>,
    pub span: Span,
}

/// Attribute
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

impl Attribute {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
            span,
        }
    }

    pub fn with_args(mut self, args: Vec<Expr>) -> Self {
        self.args = args;
        self
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{:?}", p),
            Type::Named(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}<{}>", name, args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))
                }
            }
            Type::Tuple(types) => write!(f, "({})", types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")),
            Type::Array(ty, size) => {
                if let Some(size) = size {
                    write!(f, "[{}; {}]", ty, size)
                } else {
                    write!(f, "[{}]", ty)
                }
            }
            Type::Slice(ty) => write!(f, "[{}]", ty),
            Type::Ref(ty, is_mut) => write!(f, "&{}{}", if *is_mut { "mut " } else { "" }, ty),
            Type::RawPtr(ty, is_mut) => write!(f, "*{}{}", if *is_mut { "mut " } else { "const " }, ty),
            Type::Function(ft) => write!(f, "fn({}) -> {}", ft.params.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "), ft.return_type),
            Type::ImplTrait(bounds) => write!(f, "impl {}", bounds.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" + ")),
            Type::DynTrait(bounds) => write!(f, "dyn {}", bounds.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" + ")),
            Type::Generic(name) => write!(f, "{}", name),
            Type::Infer => write!(f, "_"),
            Type::Never => write!(f, "!"),
            Type::Paren(ty) => write!(f, "({})", ty),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::Str => write!(f, "&str"),
            PrimitiveType::String => write!(f, "String"),
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::I128 => write!(f, "i128"),
            PrimitiveType::Isize => write!(f, "isize"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::U128 => write!(f, "u128"),
            PrimitiveType::Usize => write!(f, "usize"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Unit => write!(f, "()"),
            PrimitiveType::Never => write!(f, "!"),
        }
    }
}



