//! Symbol table for Bhabex compiler
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use std::collections::HashMap;
use crate::lexer::Span;
use crate::ast::Type;

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Module,
    Parameter,
    Local,
    Field,
    Variant,
    Macro,
    Label,
    Global,
}

/// A symbol in the symbol table
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub ty: Option<Type>,
    pub is_mutable: bool,
    pub is_pub: bool,
    pub is_static: bool,
    pub is_async: bool,
    pub scope_id: usize,
}

impl Symbol {
    pub fn new(name: impl Into<String>, kind: SymbolKind, span: Span, scope_id: usize) -> Self {
        Self {
            name: name.into(),
            kind,
            span,
            ty: None,
            is_mutable: false,
            is_pub: false,
            is_static: false,
            is_async: false,
            scope_id,
        }
    }

    pub fn with_type(mut self, ty: Type) -> Self {
        self.ty = Some(ty);
        self
    }

    pub fn mutable(mut self) -> Self {
        self.is_mutable = true;
        self
    }

    pub fn pub_(mut self) -> Self {
        self.is_pub = true;
        self
    }

    pub fn static_(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn async_(mut self) -> Self {
        self.is_async = true;
        self
    }
}

/// Scope in the symbol table
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    pub parent: Option<usize>,
    pub symbols: HashMap<String, Symbol>,
    pub span: Span,
}

impl Scope {
    pub fn new(id: usize, parent: Option<usize>, span: Span) -> Self {
        Self {
            id,
            parent,
            symbols: HashMap::new(),
            span,
        }
    }

    pub fn insert(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.symbols.insert(symbol.name.clone(), symbol)
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }
}

/// Symbol table with nested scopes
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let global_scope = Scope::new(0, None, Span::new(0, 0, 1, 1));
        Self {
            scopes: vec![global_scope],
            current_scope: 0,
            next_scope_id: 1,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self, span: Span) {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;
        let new_scope = Scope::new(scope_id, Some(self.current_scope), span);
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope = self.scopes.len() - 1;
        }
    }

    /// Get current scope ID
    pub fn current_scope_id(&self) -> usize {
        self.scopes[self.current_scope].id
    }

    /// Define a symbol in current scope
    pub fn define(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.scopes[self.current_scope].insert(symbol)
    }

    /// Lookup a symbol in current scope only
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.scopes[self.current_scope].get(name)
    }

    /// Lookup a symbol in current and parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = self.current_scope;
        loop {
            if let Some(sym) = self.scopes[scope_idx].get(name) {
                return Some(sym);
            }
            if let Some(parent) = self.scopes[scope_idx].parent {
                scope_idx = parent;
            } else {
                break;
            }
        }
        None
    }

    /// Lookup a mutable symbol in current and parent scopes
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        let mut scope_idx = self.current_scope;
        loop {
            if let Some(sym) = self.scopes[scope_idx].get_mut(name) {
                return Some(sym);
            }
            if let Some(parent) = self.scopes[scope_idx].parent {
                scope_idx = parent;
            } else {
                break;
            }
        }
        None
    }

    /// Get all symbols in current scope
    pub fn current_scope_symbols(&self) -> &HashMap<String, Symbol> {
        &self.scopes[self.current_scope].symbols
    }

    /// Get all symbols in all scopes (for debugging)
    pub fn all_symbols(&self) -> Vec<&Symbol> {
        self.scopes.iter().flat_map(|s| s.symbols.values()).collect()
    }

    /// Get scope by ID
    pub fn get_scope(&self, id: usize) -> Option<&Scope> {
        self.scopes.iter().find(|s| s.id == id)
    }

    /// Get mutable scope by ID
    pub fn get_scope_mut(&mut self, id: usize) -> Option<&mut Scope> {
        self.scopes.iter_mut().find(|s| s.id == id)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}