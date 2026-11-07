//! Bhabex Parser - Recursive descent parser with error recovery
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use crate::lexer::{Token, TokenKind, Span};
use crate::ast::*;
use crate::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticBag};
use crate::symbols::{SymbolTable, Symbol, SymbolKind};
use std::collections::VecDeque;

/// Parser for Bhabex language
pub struct Parser<'a> {
    tokens: VecDeque<Token>,
    source: &'a str,
    diagnostics: DiagnosticBag,
    symbols: SymbolTable,
    current_scope: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>, source: &'a str) -> Self {
        let mut symbols = SymbolTable::new();
        let global_scope = symbols.current_scope_id();
        
        Self {
            tokens: tokens.into(),
            source,
            diagnostics: DiagnosticBag::new(),
            symbols,
            current_scope: global_scope,
        }
    }

    /// Parse the entire program
    pub fn parse(mut self) -> (Program, DiagnosticBag, SymbolTable) {
        let mut items = Vec::new();
        let start_span = self.current_span();
        
        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(_) => {
                    self.synchronize();
                }
            }
        }
        
        let end_span = self.previous_span();
        let program = Program::new(items, Span::new(start_span.start, end_span.end, start_span.line, start_span.column));
        
        (program, self.diagnostics, self.symbols)
    }

    // ===== Item parsing =====

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let attrs = self.parse_attributes()?;
        
        if self.check(&TokenKind::Fn) {
            self.parse_function(attrs).map(Item::Function)
        } else if self.check(&TokenKind::Struct) {
            self.parse_struct(attrs).map(Item::Struct)
        } else if self.check(&TokenKind::Enum) {
            self.parse_enum(attrs).map(Item::Enum)
        } else if self.check(&TokenKind::Trait) {
            self.parse_trait(attrs).map(Item::Trait)
        } else if self.check(&TokenKind::Impl) {
            self.parse_impl(attrs).map(Item::Impl)
        } else if self.check(&TokenKind::Type) {
            self.parse_type_alias(attrs).map(Item::TypeAlias)
        } else if self.check(&TokenKind::Const) {
            self.parse_const(attrs).map(Item::Const)
        } else if self.check(&TokenKind::Static) {
            self.parse_static(attrs).map(Item::Static)
        } else if self.check(&TokenKind::Mod) {
            self.parse_module(attrs).map(Item::Module)
        } else if self.check(&TokenKind::Import) {
            self.parse_import().map(Item::Import)
        } else if self.check(&TokenKind::Export) {
            self.parse_export().map(Item::Export)
        } else {
            Err(self.error("Expected item definition"))
        }
    }

    fn parse_function(&mut self, attrs: Vec<Attribute>) -> Result<Function, ParseError> {
        let start = self.current_span();
        let is_async = self.consume_if(&TokenKind::Async).is_some();
        let is_unsafe = self.consume_if(&TokenKind::Unsafe).is_some();
        let is_extern = self.consume_if(&TokenKind::Extern).is_some();
        
        let abi = if is_extern {
            if self.check(&TokenKind::String) {
                Some(self.consume().unwrap().text.trim_matches('"').to_string())
            } else {
                None
            }
        } else {
            None
        };

        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();

        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.consume_if(&TokenKind::Arrow).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Function {
            name,
            generics,
            params,
            return_type,
            body,
            is_async,
            is_pub,
            is_unsafe,
            is_extern,
            abi,
            attrs,
            span,
        })
    }

    fn parse_struct(&mut self, attrs: Vec<Attribute>) -> Result<Struct, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Struct)?;
        let name = self.expect_ident()?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let generics = self.parse_generics()?;

        let fields = if self.check(&TokenKind::LBrace) {
            self.parse_struct_fields()?
        } else {
            Vec::new()
        };

        let methods = Vec::new(); // TODO: parse impl blocks
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Struct { name, generics, fields, methods, is_pub, attrs, span })
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<StructField>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let attrs = self.parse_attributes()?;
            let is_pub = self.consume_if(&TokenKind::Pub).is_some();
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type()?;
            fields.push(StructField { name, ty, is_pub, is_mut, attrs, span: self.current_span() });
            
            if !self.consume_if(&TokenKind::Comma).is_some() {
                break;
            }
        }
        
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_enum(&mut self, attrs: Vec<Attribute>) -> Result<Enum, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Enum)?;
        let name = self.expect_ident()?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let generics = self.parse_generics()?;

        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let attrs = self.parse_attributes()?;
            let name = self.expect_ident()?;
            
            let fields = if self.check(&TokenKind::LParen) {
                self.consume();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    let ty = self.parse_type()?;
                    fields.push(EnumField::Tuple(ty));
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RParen)?;
                fields
            } else if self.check(&TokenKind::LBrace) {
                self.consume();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let name = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let ty = self.parse_type()?;
                    fields.push(EnumField::Named(name, ty));
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RBrace)?;
                fields
            } else {
                Vec::new()
            };

            let discriminant = if self.consume_if(&TokenKind::Assign).is_some() {
                Some(self.parse_expr()?)
            } else {
                None
            };

            variants.push(EnumVariant { name, fields, discriminant, attrs, span: self.current_span() });
            
            if !self.consume_if(&TokenKind::Comma).is_some() {
                break;
            }
        }
        
        self.expect(&TokenKind::RBrace)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Enum { name, generics, variants, is_pub, attrs, span })
    }

    fn parse_trait(&mut self, attrs: Vec<Attribute>) -> Result<Trait, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Trait)?;
        let name = self.expect_ident()?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let generics = self.parse_generics()?;

        let bounds = if self.consume_if(&TokenKind::Colon).is_some() {
            self.parse_type_bounds()?
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let attrs = self.parse_attributes()?;
            if self.check(&TokenKind::Fn) {
                items.push(TraitItem::Function(self.parse_function(attrs)?));
            } else if self.check(&TokenKind::Type) {
                items.push(TraitItem::Type(self.parse_type_alias(attrs)?));
            } else if self.check(&TokenKind::Const) {
                items.push(TraitItem::Const(self.parse_const(attrs)?));
            } else {
                return Err(self.error("Expected trait item"));
            }
        }
        
        self.expect(&TokenKind::RBrace)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Trait { name, generics, bounds, items, is_pub, attrs, span })
    }

    fn parse_impl(&mut self, attrs: Vec<Attribute>) -> Result<Impl, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Impl)?;
        let generics = self.parse_generics()?;

        let (trait_ref, target_type) = if self.check(&TokenKind::For) {
            self.consume();
            let target_type = self.parse_type()?;
            self.expect(&TokenKind::Colon)?;
            let trait_ref = self.parse_trait_ref()?;
            (Some(trait_ref), target_type)
        } else {
            let target_type = self.parse_type()?;
            (None, target_type)
        };

        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let attrs = self.parse_attributes()?;
            if self.check(&TokenKind::Fn) {
                items.push(ImplItem::Function(self.parse_function(attrs)?));
            } else if self.check(&TokenKind::Type) {
                items.push(ImplItem::Type(self.parse_type_alias(attrs)?));
            } else if self.check(&TokenKind::Const) {
                items.push(ImplItem::Const(self.parse_const(attrs)?));
            } else {
                return Err(self.error("Expected impl item"));
            }
        }
        
        self.expect(&TokenKind::RBrace)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Impl { generics, trait_ref, target_type, items, span })
    }

    fn parse_type_alias(&mut self, attrs: Vec<Attribute>) -> Result<TypeAlias, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Type)?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let name = self.expect_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::Assign)?;
        let ty = self.parse_type()?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(TypeAlias { name, generics, ty, is_pub, attrs, span })
    }

    fn parse_const(&mut self, attrs: Vec<Attribute>) -> Result<Const, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Const)?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Assign)?;
        let value = self.parse_expr()?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Const { name, ty, value, is_pub, attrs, span })
    }

    fn parse_static(&mut self, attrs: Vec<Attribute>) -> Result<Static, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Static)?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let is_mut = self.consume_if(&TokenKind::Mut).is_some();
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let value = if self.consume_if(&TokenKind::Assign).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Static { name, ty, value, is_mut, is_pub, attrs, span })
    }

    fn parse_module(&mut self, attrs: Vec<Attribute>) -> Result<Module, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Mod)?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        let name = self.expect_ident()?;

        let items = if self.check(&TokenKind::LBrace) {
            self.consume();
            let mut items = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                items.push(self.parse_item()?);
            }
            self.expect(&TokenKind::RBrace)?;
            items
        } else {
            self.expect(&TokenKind::Semi)?;
            Vec::new()
        };

        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Module { name, items, is_pub, span })
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Import)?;
        let is_pub = self.consume_if(&TokenKind::Pub).is_some();
        
        let path = self.expect_ident()?;
        let mut items = Vec::new();
        
        if self.consume_if(&TokenKind::DoubleColon).is_some() {
            if self.check(&TokenKind::LBrace) {
                self.consume();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let name = self.expect_ident()?;
                    let alias = if self.consume_if(&TokenKind::As).is_some() {
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    items.push(ImportItem { name, alias, span: self.current_span() });
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RBrace)?;
            } else {
                let name = self.expect_ident()?;
                let alias = if self.consume_if(&TokenKind::As).is_some() {
                    Some(self.expect_ident()?)
                } else {
                    None
                };
                items.push(ImportItem { name, alias, span: self.current_span() });
            }
        }

        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Import { path, items, is_pub, span })
    }

    fn parse_export(&mut self) -> Result<Export, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Export)?;
        
        let mut items = Vec::new();
        if self.check(&TokenKind::LBrace) {
            self.consume();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                let name = self.expect_ident()?;
                let alias = if self.consume_if(&TokenKind::As).is_some() {
                    Some(self.expect_ident()?)
                } else {
                    None
                };
                items.push(ExportItem { name, alias, span: self.current_span() });
                if !self.consume_if(&TokenKind::Comma).is_some() { break; }
            }
            self.expect(&TokenKind::RBrace)?;
        }

        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Export { items, span })
    }

    // ===== Helper parsing =====

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        let mut attrs = Vec::new();
        while self.check(&TokenKind::At) || self.check(&TokenKind::Hash) {
            self.consume(); // consume @ or #
            if self.check(&TokenKind::LBracket) {
                self.consume();
                let name = self.expect_ident()?;
                let args = if self.check(&TokenKind::LParen) {
                    self.consume();
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expr()?);
                        if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    args
                } else {
                    Vec::new()
                };
                self.expect(&TokenKind::RBracket)?;
                attrs.push(Attribute::new(name, self.current_span()).with_args(args));
            } else {
                let name = self.expect_ident()?;
                attrs.push(Attribute::new(name, self.current_span()));
            }
        }
        Ok(attrs)
    }

    fn parse_generics(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        if !self.check(&TokenKind::Lt) {
            return Ok(Vec::new());
        }
        
        self.consume(); // consume <
        let mut generics = Vec::new();
        
        while !self.check(&TokenKind::Gt) && !self.is_at_end() {
            let name = self.expect_ident()?;
            let bounds = if self.consume_if(&TokenKind::Colon).is_some() {
                self.parse_type_bounds()?
            } else {
                Vec::new()
            };
            let default = if self.consume_if(&TokenKind::Assign).is_some() {
                Some(self.parse_type()?)
            } else {
                None
            };
            
            generics.push(GenericParam {
                name,
                bounds,
                default,
                span: self.current_span(),
            });
            
            if !self.consume_if(&TokenKind::Comma).is_some() {
                break;
            }
        }
        
        self.expect(&TokenKind::Gt)?;
        Ok(generics)
    }

    fn parse_type_bounds(&mut self) -> Result<Vec<Type>, ParseError> {
        let mut bounds = Vec::new();
        bounds.push(self.parse_type()?);
        while self.consume_if(&TokenKind::Plus).is_some() {
            bounds.push(self.parse_type()?);
        }
        Ok(bounds)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let is_ref = self.consume_if(&TokenKind::And).is_some();
            let is_move = self.consume_if(&TokenKind::Move).is_some();
            
            let name = self.expect_ident()?;
            let ty = if self.consume_if(&TokenKind::Colon).is_some() {
                Some(self.parse_type()?)
            } else {
                None
            };
            let default = if self.consume_if(&TokenKind::Assign).is_some() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            
            params.push(Param {
                name,
                ty,
                is_mut,
                is_ref,
                is_move,
                default,
                span: self.current_span(),
            });
            
            if !self.consume_if(&TokenKind::Comma).is_some() {
                break;
            }
        }
        
        Ok(params)
    }

    fn parse_trait_ref(&mut self) -> Result<TraitRef, ParseError> {
        let start = self.current_span();
        let path = self.expect_ident()?;
        let args = if self.check(&TokenKind::Lt) {
            self.parse_type_args()?
        } else {
            Vec::new()
        };
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(TraitRef { path, args, span })
    }

    fn parse_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.expect(&TokenKind::Lt)?;
        let mut args = Vec::new();
        
        while !self.check(&TokenKind::Gt) && !self.is_at_end() {
            args.push(self.parse_type()?);
            if !self.consume_if(&TokenKind::Comma).is_some() {
                break;
            }
        }
        
        self.expect(&TokenKind::Gt)?;
        Ok(args)
    }

    // ===== Type parsing =====

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        self.parse_type_or()
    }

    fn parse_type_or(&mut self) -> Result<Type, ParseError> {
        let mut left = self.parse_type_and()?;
        
        while self.consume_if(&TokenKind::Or).is_some() {
            let right = self.parse_type_and()?;
            left = Type::DynTrait(vec![left, right]);
        }
        
        Ok(left)
    }

    fn parse_type_and(&mut self) -> Result<Type, ParseError> {
        let mut left = self.parse_type_primary()?;
        
        while self.consume_if(&TokenKind::And).is_some() {
            let right = self.parse_type_primary()?;
            left = Type::DynTrait(vec![left, right]);
        }
        
        Ok(left)
    }

    fn parse_type_primary(&mut self) -> Result<Type, ParseError> {
        if self.consume_if(&TokenKind::And).is_some() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let ty = self.parse_type_primary()?;
            return Ok(Type::Ref(Box::new(ty), is_mut));
        }
        
        if self.consume_if(&TokenKind::Star).is_some() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let ty = self.parse_type_primary()?;
            return Ok(Type::RawPtr(Box::new(ty), is_mut));
        }
        
        if self.check(&TokenKind::LParen) {
            self.consume();
            let ty = self.parse_type()?;
            self.expect(&TokenKind::RParen)?;
            return Ok(Type::Paren(Box::new(ty)));
        }
        
        if self.check(&TokenKind::LBracket) {
            self.consume();
            let ty = self.parse_type()?;
            let size = if self.consume_if(&TokenKind::Semi).is_some() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&TokenKind::RBracket)?;
            return Ok(Type::Array(Box::new(ty), size));
        }
        
        if self.check(&TokenKind::Fn) {
            return self.parse_function_type();
        }
        
        if self.check(&TokenKind::Impl) {
            self.consume();
            let bounds = self.parse_type_bounds()?;
            return Ok(Type::ImplTrait(bounds));
        }
        
        if self.check(&TokenKind::Dyn) {
            self.consume();
            let bounds = self.parse_type_bounds()?;
            return Ok(Type::DynTrait(bounds));
        }
        
        // Named type or primitive
        let ident = self.expect_ident()?;
        let args = if self.check(&TokenKind::Lt) {
            self.parse_type_args()?
        } else {
            Vec::new()
        };
        
        // Check for primitive types
        let ty = match ident.as_str() {
            "bool" => Type::Primitive(PrimitiveType::Bool),
            "char" => Type::Primitive(PrimitiveType::Char),
            "str" => Type::Primitive(PrimitiveType::Str),
            "String" => Type::Primitive(PrimitiveType::String),
            "i8" => Type::Primitive(PrimitiveType::I8),
            "i16" => Type::Primitive(PrimitiveType::I16),
            "i32" => Type::Primitive(PrimitiveType::I32),
            "i64" => Type::Primitive(PrimitiveType::I64),
            "i128" => Type::Primitive(PrimitiveType::I128),
            "isize" => Type::Primitive(PrimitiveType::Isize),
            "u8" => Type::Primitive(PrimitiveType::U8),
            "u16" => Type::Primitive(PrimitiveType::U16),
            "u32" => Type::Primitive(PrimitiveType::U32),
            "u64" => Type::Primitive(PrimitiveType::U64),
            "u128" => Type::Primitive(PrimitiveType::U128),
            "usize" => Type::Primitive(PrimitiveType::Usize),
            "f32" => Type::Primitive(PrimitiveType::F32),
            "f64" => Type::Primitive(PrimitiveType::F64),
            "()" => Type::Primitive(PrimitiveType::Unit),
            "never" => Type::Primitive(PrimitiveType::Never),
            _ => Type::Named(ident, args),
        };
        
        // Check for reference type after named type
        if self.consume_if(&TokenKind::And).is_some() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            return Ok(Type::Ref(Box::new(ty), is_mut));
        }
        
        Ok(ty)
    }

    fn parse_function_type(&mut self) -> Result<Type, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Fn)?;
        let is_async = self.consume_if(&TokenKind::Async).is_some();
        let is_unsafe = self.consume_if(&TokenKind::Unsafe).is_some();
        
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            params.push(self.parse_type()?);
            if !self.consume_if(&TokenKind::Comma).is_some() { break; }
        }
        self.expect(&TokenKind::RParen)?;
        
        let return_type = if self.consume_if(&TokenKind::Arrow).is_some() {
            Box::new(self.parse_type()?)
        } else {
            Box::new(Type::Primitive(PrimitiveType::Unit))
        };
        
        let abi = if self.check(&TokenKind::Extern) {
            self.consume();
            if self.check(&TokenKind::String) {
                Some(self.consume().unwrap().text.trim_matches('"').to_string())
            } else {
                None
            }
        } else {
            None
        };
        
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Type::Function(FunctionType {
            params,
            return_type,
            is_async,
            is_unsafe,
            abi,
        }))
    }

    // ===== Statement parsing =====

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace)?;
        self.symbols.enter_scope(start);
        
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        
        self.symbols.exit_scope();
        self.expect(&TokenKind::RBrace)?;
        
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(Block::new(stmts, span))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let attrs = self.parse_attributes()?;
        
        if self.check(&TokenKind::Let) {
            self.parse_let_stmt().map(Stmt::Let)
        } else if self.check(&TokenKind::Return) {
            self.parse_return_stmt().map(Stmt::Return)
        } else if self.check(&TokenKind::If) {
            self.parse_if_stmt().map(Stmt::If)
        } else if self.check(&TokenKind::While) {
            self.parse_while_stmt().map(Stmt::While)
        } else if self.check(&TokenKind::For) {
            self.parse_for_stmt().map(Stmt::For)
        } else if self.check(&TokenKind::Loop) {
            self.parse_loop_stmt().map(Stmt::Loop)
        } else if self.check(&TokenKind::Match) {
            self.parse_match_stmt().map(Stmt::Match)
        } else if self.check(&TokenKind::Break) {
            self.parse_break_stmt().map(Stmt::Break)
        } else if self.check(&TokenKind::Continue) {
            self.parse_continue_stmt().map(Stmt::Continue)
        } else if self.check(&TokenKind::LBrace) {
            self.parse_block().map(Stmt::Block)
        } else if self.check(&TokenKind::Try) {
            self.parse_try_stmt().map(Stmt::Try)
        } else if self.check(&TokenKind::Spawn) {
            self.parse_spawn_stmt().map(Stmt::Spawn)
        } else if self.check(&TokenKind::Defer) {
            self.parse_defer_stmt().map(Stmt::Defer)
        } else if self.check(&TokenKind::Unsafe) {
            self.parse_unsafe_stmt().map(Stmt::Unsafe)
        } else if self.check(&TokenKind::Label) {
            self.parse_label_stmt().map(Stmt::Label)
        } else if self.check(&TokenKind::Goto) {
            self.parse_goto_stmt().map(Stmt::Goto)
        } else {
            self.parse_expr_stmt().map(Stmt::Expr)
        }
    }

    fn parse_let_stmt(&mut self) -> Result<LetStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Let)?;
        let is_mut = self.consume_if(&TokenKind::Mut).is_some();
        let is_ref = self.consume_if(&TokenKind::And).is_some();
        let name = self.expect_ident()?;
        let ty = if self.consume_if(&TokenKind::Colon).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.consume_if(&TokenKind::Assign).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(LetStmt { name, ty, is_mut, is_ref, init, span })
    }

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Return)?;
        let value = if !self.check(&TokenKind::Semi) && !self.check(&TokenKind::RBrace) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(ReturnStmt { value, span })
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_branch = Box::new(self.parse_stmt()?);
        let else_branch = if self.consume_if(&TokenKind::Else).is_some() {
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(IfStmt { condition, then_branch, else_branch, span })
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::While)?;
        let condition = self.parse_expr()?;
        let body = Box::new(self.parse_stmt()?);
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(WhileStmt { condition, body, span })
    }

    fn parse_for_stmt(&mut self) -> Result<ForStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::For)?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::In)?;
        let iterator = self.parse_expr()?;
        let body = Box::new(self.parse_stmt()?);
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(ForStmt { pattern, iterator, body, span })
    }

    fn parse_loop_stmt(&mut self) -> Result<LoopStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Loop)?;
        let body = Box::new(self.parse_stmt()?);
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(LoopStmt { body, span })
    }

    fn parse_match_stmt(&mut self) -> Result<MatchStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Match)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::LBrace)?;
        
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            let guard = if self.consume_if(&TokenKind::If).is_some() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&TokenKind::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(MatchArm { pattern, guard, body, span: self.current_span() });
            if !self.consume_if(&TokenKind::Comma).is_some() { break; }
        }
        
        self.expect(&TokenKind::RBrace)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(MatchStmt { expr, arms, span })
    }

    fn parse_break_stmt(&mut self) -> Result<BreakStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Break)?;
        let label = if self.check(&TokenKind::Ident) {
            Some(self.consume().unwrap().text)
        } else {
            None
        };
        let value = if !self.check(&TokenKind::Semi) && !self.check(&TokenKind::RBrace) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(BreakStmt { label, value, span })
    }

    fn parse_continue_stmt(&mut self) -> Result<ContinueStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Continue)?;
        let label = if self.check(&TokenKind::Ident) {
            Some(self.consume().unwrap().text)
        } else {
            None
        };
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(ContinueStmt { label, span })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt, ParseError> {
        let start = self.current_span();
        let expr = self.parse_expr()?;
        if !self.check(&TokenKind::RBrace) {
            self.expect(&TokenKind::Semi)?;
        }
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(ExprStmt { expr, span })
    }

    fn parse_try_stmt(&mut self) -> Result<TryStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Try)?;
        let body = self.parse_block()?;
        
        let mut catch_clauses = Vec::new();
        while self.consume_if(&TokenKind::Catch).is_some() {
            let pattern = self.parse_pattern()?;
            let body = self.parse_block()?;
            catch_clauses.push(CatchClause { pattern, body, span: self.current_span() });
        }
        
        let finally = if self.consume_if(&TokenKind::Finally).is_some() {
            Some(self.parse_block()?)
        } else {
            None
        };
        
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(TryStmt { body, catch_clauses, finally, span })
    }

    fn parse_spawn_stmt(&mut self) -> Result<SpawnStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Spawn)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(SpawnStmt { expr, span })
    }

    fn parse_defer_stmt(&mut self) -> Result<DeferStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Defer)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(DeferStmt { expr, span })
    }

    fn parse_unsafe_stmt(&mut self) -> Result<UnsafeStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Unsafe)?;
        let body = self.parse_block()?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(UnsafeStmt { body, span })
    }

    fn parse_label_stmt(&mut self) -> Result<LabelStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Label)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let stmt = Box::new(self.parse_stmt()?);
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(LabelStmt { name, stmt, span })
    }

    fn parse_goto_stmt(&mut self) -> Result<GotoStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Goto)?;
        let label = self.expect_ident()?;
        self.expect(&TokenKind::Semi)?;
        let span = Span::new(start.start, self.previous_span().end, start.line, start.column);
        Ok(GotoStmt { label, span })
    }

    // ===== Expression parsing (Pratt parser) =====

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_prec(0)
    }

    fn parse_expr_prec(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;
        
        while let Some(op) = self.current_token() {
            let prec = Self::precedence(&op.kind);
            if prec < min_prec {
                break;
            }
            
            self.consume();
            let right = self.parse_expr_prec(prec + 1)?;
            
            let op_kind = match op.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                TokenKind::Pow => BinaryOp::Pow,
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::Ne => BinaryOp::Ne,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Ge => BinaryOp::Ge,
                TokenKind::And => BinaryOp::And,
                TokenKind::Or => BinaryOp::Or,
                TokenKind::Assign => BinaryOp::Assign,
                TokenKind::PlusEq => BinaryOp::AddAssign,
                TokenKind::MinusEq => BinaryOp::SubAssign,
                TokenKind::StarEq => BinaryOp::MulAssign,
                TokenKind::SlashEq => BinaryOp::DivAssign,
                TokenKind::PercentEq => BinaryOp::ModAssign,
                _ => break,
            };
            
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: op_kind,
                right: Box::new(right),
                span: Span::new(left.span().start, self.previous_span().end, left.span().line, left.span().column),
            });
        }
        
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        if self.consume_if(&TokenKind::Minus).is_some() {
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        if self.consume_if(&TokenKind::Not).is_some() {
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        if self.consume_if(&TokenKind::BitNot).is_some() {
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::BitNot,
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        if self.consume_if(&TokenKind::And).is_some() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: if is_mut { UnaryOp::RefMut } else { UnaryOp::Ref },
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        if self.consume_if(&TokenKind::Star).is_some() {
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Deref,
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        if self.consume_if(&TokenKind::Move).is_some() {
            let expr = self.parse_prefix()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Move,
                expr: Box::new(expr),
                span: self.current_span(),
            }));
        }
        
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.consume().ok_or_else(|| self.error("Unexpected end of input"))?;
        
        match token.kind {
            TokenKind::Int => Ok(Expr::Literal(Literal::Int(token.text.parse().unwrap()), token.span)),
            TokenKind::Float => Ok(Expr::Literal(Literal::Float(token.text.parse().unwrap()), token.span)),
            TokenKind::True => Ok(Expr::Literal(Literal::Bool(true), token.span)),
            TokenKind::False => Ok(Expr::Literal(Literal::Bool(false), token.span)),
            TokenKind::Char => Ok(Expr::Literal(Literal::Char(token.text.chars().nth(1).unwrap()), token.span)),
            TokenKind::String => Ok(Expr::Literal(Literal::String(token.text.trim_matches('"').to_string()), token.span)),
            TokenKind::RawString => Ok(Expr::Literal(Literal::String(token.text.trim_start_matches('r').trim_matches('#').trim_matches('"').to_string()), token.span)),
            TokenKind::Null => Ok(Expr::Literal(Literal::Null, token.span)),
            TokenKind::Ident => {
                let name = token.text;
                
                // Check for macro invocation
                if self.check(&TokenKind::Bang) {
                    self.consume(); // consume !
                    self.expect(&TokenKind::LParen)?;
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expr()?);
                        if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr::Macro(MacroExpr { name, args, span: token.span }));
                }
                
                // Check for struct literal
                if self.check(&TokenKind::LBrace) {
                    return self.parse_struct_literal(name, token.span);
                }
                
                // Check for function call or method call
                if self.check(&TokenKind::LParen) {
                    self.consume(); // consume (
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expr()?);
                        if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr::Call(CallExpr {
                        callee: Box::new(Expr::Ident(name, token.span)),
                        args,
                        span: token.span,
                    }));
                }
                
                Ok(Expr::Ident(name, token.span))
            }
            TokenKind::LParen => {
                // Parenthesized expression or tuple
                if self.check(&TokenKind::RParen) {
                    self.consume();
                    return Ok(Expr::Tuple(Vec::new(), token.span));
                }
                
                let expr = self.parse_expr()?;
                
                if self.consume_if(&TokenKind::Comma).is_some() {
                    // Tuple
                    let mut elements = vec![expr];
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        elements.push(self.parse_expr()?);
                        if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr::Tuple(elements, token.span));
                }
                
                self.expect(&TokenKind::RParen)?;
                Ok(Expr::Parenthesized(Box::new(expr), token.span))
            }
            TokenKind::LBrace => {
                // Block expression
                let block = self.parse_block()?;
                Ok(Expr::Block(block))
            }
            TokenKind::LBracket => {
                // Array or slice
                let mut elements = Vec::new();
                while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                    elements.push(self.parse_expr()?);
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::Array(ArrayExpr { elements, span: token.span }))
            }
            TokenKind::If => {
                let condition = self.parse_expr()?;
                let then_branch = Box::new(self.parse_expr()?);
                let else_branch = if self.consume_if(&TokenKind::Else).is_some() {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr::If(IfExpr { condition, then_branch, else_branch, span: token.span }))
            }
            TokenKind::Match => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let pattern = self.parse_pattern()?;
                    let guard = if self.consume_if(&TokenKind::If).is_some() {
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::FatArrow)?;
                    let body = self.parse_expr()?;
                    arms.push(MatchArm { pattern, guard, body, span: self.current_span() });
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RBrace)?;
                Ok(Expr::Match(MatchExpr { expr: Box::new(expr), arms, span: token.span }))
            }
            TokenKind::Async => {
                let body = Box::new(self.parse_expr()?);
                Ok(Expr::Async(AsyncExpr { body, span: token.span }))
            }
            TokenKind::Await => {
                let expr = Box::new(self.parse_prefix()?);
                Ok(Expr::Await(AwaitExpr { expr, span: token.span }))
            }
            TokenKind::Try => {
                let expr = Box::new(self.parse_prefix()?);
                Ok(Expr::Try(TryExpr { expr, span: token.span }))
            }
            TokenKind::Throw => {
                let expr = Box::new(self.parse_expr()?);
                Ok(Expr::Throw(ThrowExpr { expr, span: token.span }))
            }
            TokenKind::Spawn => {
                let expr = Box::new(self.parse_expr()?);
                Ok(Expr::Spawn(SpawnExpr { expr, span: token.span }))
            }
            TokenKind::Closure => {
                self.parse_closure(token.span)
            }
            _ => Err(self.error(&format!("Unexpected token: {:?}", token.kind))),
        }
    }

    fn parse_struct_literal(&mut self, name: String, span: Span) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let field_name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push((field_name, value));
            if !self.consume_if(&TokenKind::Comma).is_some() { break; }
        }
        
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Struct(StructExpr { name, fields, span }))
    }

    fn parse_closure(&mut self, span: Span) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;
        
        let return_type = if self.consume_if(&TokenKind::Arrow).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = Box::new(self.parse_expr()?);
        
        Ok(Expr::Closure(ClosureExpr {
            params,
            return_type,
            body,
            is_async: false,
            is_move: false,
            captures: Vec::new(),
            span,
        }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if self.consume_if(&TokenKind::Underscore).is_some() {
            return Ok(Pattern::Wildcard(self.current_span()));
        }
        
        if self.check(&TokenKind::Ident) {
            let name = self.consume().unwrap().text;
            
            if self.check(&TokenKind::At) {
                self.consume();
                let pattern = self.parse_pattern()?;
                return Ok(Pattern::Binding(name, Box::new(pattern), self.current_span()));
            }
            
            if self.check(&TokenKind::LParen) {
                self.consume();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    fields.push(self.parse_pattern()?);
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RParen)?;
                return Ok(Pattern::Tuple(fields, self.current_span()));
            }
            
            if self.check(&TokenKind::LBrace) {
                self.consume();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let field_name = self.expect_ident()?;
                    let pattern = if self.consume_if(&TokenKind::Colon).is_some() {
                        self.parse_pattern()?
                    } else {
                        Pattern::Ident(field_name.clone(), self.current_span())
                    };
                    fields.push((field_name, pattern));
                    if !self.consume_if(&TokenKind::Comma).is_some() { break; }
                }
                self.expect(&TokenKind::RBrace)?;
                return Ok(Pattern::Struct(name, fields, self.current_span()));
            }
            
            return Ok(Pattern::Ident(name, self.current_span()));
        }
        
        if self.check(&TokenKind::Int) || self.check(&TokenKind::Float) 
            || self.check(&TokenKind::String) || self.check(&TokenKind::Char)
            || self.check(&TokenKind::True) || self.check(&TokenKind::False) {
            let token = self.consume().unwrap();
            let literal = match token.kind {
                TokenKind::Int => Literal::Int(token.text.parse().unwrap()),
                TokenKind::Float => Literal::Float(token.text.parse().unwrap()),
                TokenKind::String => Literal::String(token.text.trim_matches('"').to_string()),
                TokenKind::Char => Literal::Char(token.text.chars().nth(1).unwrap()),
                TokenKind::True => Literal::Bool(true),
                TokenKind::False => Literal::Bool(false),
                _ => Literal::Unit,
            };
            return Ok(Pattern::Literal(literal, token.span));
        }
        
        if self.consume_if(&TokenKind::And).is_some() {
            let is_mut = self.consume_if(&TokenKind::Mut).is_some();
            let pattern = self.parse_pattern()?;
            return Ok(Pattern::Ref(Box::new(pattern), self.current_span()));
        }
        
        if self.consume_if(&TokenKind::Star).is_some() {
            let pattern = self.parse_pattern()?;
            return Ok(Pattern::Deref(Box::new(pattern), self.current_span()));
        }
        
        if self.check(&TokenKind::LBracket) {
            self.consume();
            let mut patterns = Vec::new();
            let mut rest = None;
            while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                if self.check(&TokenKind::Spread) {
                    self.consume();
                    rest = Some(Box::new(self.parse_pattern()?));
                    break;
                }
                patterns.push(self.parse_pattern()?);
                if !self.consume_if(&TokenKind::Comma).is_some() { break; }
            }
            self.expect(&TokenKind::RBracket)?;
            return Ok(Pattern::Slice(patterns, rest, self.current_span()));
        }
        
        if self.consume_if(&TokenKind::Or).is_some() {
            let mut patterns = vec![self.parse_pattern()?];
            while self.consume_if(&TokenKind::Or).is_some() {
                patterns.push(self.parse_pattern()?);
            }
            return Ok(Pattern::Or(patterns, self.current_span()));
        }
        
        Err(self.error("Expected pattern"))
    }

    // ===== Helper methods =====

    fn current_token(&mut self) -> Option<&Token> {
        self.tokens.front()
    }

    fn consume(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn consume_if(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.current_token().map(|t| &t.kind) == Some(kind) {
            self.consume()
        } else {
            None
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.tokens.front().map(|t| &t.kind) == Some(kind)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.consume().unwrap())
        } else {
            Err(self.error(&format!("Expected {:?}", kind)))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let token = self.expect(&TokenKind::Ident)?;
        Ok(token.text)
    }

    fn current_span(&self) -> Span {
        self.tokens.front().map(|t| t.span).unwrap_or_else(|| Span::new(0, 0, 1, 1))
    }

    fn previous_span(&self) -> Span {
        // Would need to track previous token - simplified for now
        Span::new(0, 0, 1, 1)
    }

    fn is_at_end(&self) -> bool {
        self.tokens.is_empty() || self.tokens.front().map(|t| matches!(t.kind, TokenKind::Error) && t.text.is_empty()).unwrap_or(false)
    }

    fn error(&self, msg: &str) -> ParseError {
        let span = self.current_span();
        self.diagnostics.add(Diagnostic::error(msg, span));
        ParseError { span, message: msg.to_string() }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if let Some(token) = self.consume() {
                if matches!(token.kind, TokenKind::Semi | TokenKind::RBrace | TokenKind::Fn | TokenKind::Struct | TokenKind::Enum) {
                    break;
                }
            }
        }
    }

    fn precedence(kind: &TokenKind) -> u8 {
        match kind {
            TokenKind::Or => 1,
            TokenKind::And => 2,
            TokenKind::EqEq | TokenKind::Ne => 3,
            TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 6,
            TokenKind::Pow => 7,
            _ => 0,
        }
    }
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error @ {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Convenience function to parse source
pub fn parse(source: &str) -> (Program, DiagnosticBag, SymbolTable) {
    let (tokens, lexer_diagnostics) = crate::lexer::tokenize(source);
    let mut parser = Parser::new(tokens, source);
    parser.diagnostics.extend(lexer_diagnostics);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = "fn main() { let x = 42; }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
        assert_eq!(program.items.len(), 1);
        
        if let Item::Function(f) = &program.items[0] {
            assert_eq!(f.name, "main");
            assert_eq!(f.params.len(), 0);
            assert_eq!(f.body.stmts.len(), 1);
        }
    }

    #[test]
    fn test_parse_struct() {
        let source = "struct User { name: String, age: i32 }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
        assert_eq!(program.items.len(), 1);
        
        if let Item::Struct(s) = &program.items[0] {
            assert_eq!(s.name, "User");
            assert_eq!(s.fields.len(), 2);
        }
    }

    #[test]
    fn test_parse_enum() {
        let source = "enum Option { Some(i32), None }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
        assert_eq!(program.items.len(), 1);
        
        if let Item::Enum(e) = &program.items[0] {
            assert_eq!(e.name, "Option");
            assert_eq!(e.variants.len(), 2);
        }
    }

    #[test]
    fn test_parse_expressions() {
        let source = "fn main() { let x = 1 + 2 * 3; }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn test_parse_if() {
        let source = "fn main() { if true { 1 } else { 2 } }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn test_parse_match() {
        let source = "fn main() { match x { 1 => \"one\", _ => \"other\" } }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn test_parse_closure() {
        let source = "fn main() { let f = |x| x + 1; }";
        let (program, diagnostics, _) = parse(source);
        
        assert!(!diagnostics.has_errors());
    }
}