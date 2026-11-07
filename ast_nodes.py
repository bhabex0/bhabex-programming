from dataclasses import dataclass, field
from typing import List, Optional, Any
from .tokens import Token


@dataclass(unsafe_hash=True)
class Parameter:
    name: Token
    type_annotation: Optional['TypeAnnotation'] = None


@dataclass(unsafe_hash=True)
class TypeAnnotation:
    kind: str  # "int", "float", "bool", "string", "array", "dict", "void", "any", "optional", "result", "custom"
    element_type: Optional['TypeAnnotation'] = None  # for array, optional
    key_type: Optional['TypeAnnotation'] = None  # for dict
    value_type: Optional['TypeAnnotation'] = None  # for dict, result
    ok_type: Optional['TypeAnnotation'] = None  # for result
    err_type: Optional['TypeAnnotation'] = None  # for result
    custom_name: Optional[str] = None  # for custom types
    params: List['TypeAnnotation'] = field(default_factory=list)  # for function types
    return_type: Optional['TypeAnnotation'] = None  # for function types


@dataclass(unsafe_hash=True)
class Expr:
    pass


@dataclass(unsafe_hash=True)
class Stmt:
    pass


@dataclass(unsafe_hash=True)
class Program(Stmt):
    statements: List[Stmt] = field(default_factory=list)


@dataclass(unsafe_hash=True)
class Literal(Expr):
    value: Any
    token: Token


@dataclass(unsafe_hash=True)
class Identifier(Expr):
    name: str
    token: Token


@dataclass(unsafe_hash=True)
class Binary(Expr):
    left: Expr
    operator: Token
    right: Expr


@dataclass(unsafe_hash=True)
class Unary(Expr):
    operator: Token
    right: Expr


@dataclass(unsafe_hash=True)
class Grouping(Expr):
    expression: Expr


@dataclass(unsafe_hash=True)
class Call(Expr):
    callee: Expr
    paren: Token
    arguments: List[Expr]


@dataclass(unsafe_hash=True)
class Get(Expr):
    object: Expr
    name: Token


@dataclass(unsafe_hash=True)
class Set(Expr):
    object: Expr
    name: Token
    value: Expr


@dataclass(unsafe_hash=True)
class This(Expr):
    keyword: Token


@dataclass(unsafe_hash=True)
class Super(Expr):
    keyword: Token
    method: Token


@dataclass(unsafe_hash=True)
class IndexGet(Expr):
    object: Expr
    index: Expr
    bracket: Token


@dataclass(unsafe_hash=True)
class IndexSet(Expr):
    object: Expr
    index: Expr
    value: Expr
    bracket: Token


@dataclass(unsafe_hash=True)
class ListLiteral(Expr):
    elements: List[Expr]
    bracket: Token


@dataclass(unsafe_hash=True)
class DictLiteral(Expr):
    keys: List[Expr]
    values: List[Expr]
    brace: Token


@dataclass(unsafe_hash=True)
class Range(Expr):
    start: Expr
    end: Expr
    inclusive: bool
    token: Token


@dataclass(unsafe_hash=True)
class Lambda(Expr):
    params: List[Token]
    body: List[Stmt]
    token: Token


@dataclass(unsafe_hash=True)
class MatchExpr(Expr):
    subject: Expr
    cases: List['MatchCase']
    token: Token


@dataclass(unsafe_hash=True)
class MatchCase:
    pattern: Expr
    body: List[Stmt]


@dataclass(unsafe_hash=True)
class TryExpr(Expr):
    body: List[Stmt]
    catch_var: Optional[Token]
    catch_body: List[Stmt]
    token: Token


@dataclass(unsafe_hash=True)
class ThrowExpr(Expr):
    value: Expr
    token: Token


@dataclass(unsafe_hash=True)
class AwaitExpr(Expr):
    value: Expr
    token: Token


@dataclass(unsafe_hash=True)
class YieldExpr(Expr):
    value: Optional[Expr]
    token: Token


@dataclass(unsafe_hash=True)
class ExpressionStmt(Stmt):
    expression: Expr


@dataclass(unsafe_hash=True)
class PrintStmt(Stmt):
    expressions: List[Expr]


@dataclass(unsafe_hash=True)
class VarDecl(Stmt):
    name: Token
    type_annotation: Optional[TypeAnnotation]
    initializer: Optional[Expr]


@dataclass(unsafe_hash=True)
class Assign(Stmt):
    name: Token
    value: Expr


@dataclass(unsafe_hash=True)
class Block(Stmt):
    statements: List[Stmt]


@dataclass(unsafe_hash=True)
class IfStmt(Stmt):
    condition: Expr
    then_branch: Stmt
    else_branch: Optional[Stmt]


@dataclass(unsafe_hash=True)
class WhileStmt(Stmt):
    condition: Expr
    body: Stmt


@dataclass(unsafe_hash=True)
class ForStmt(Stmt):
    var: Token
    iterable: Expr
    body: Stmt


@dataclass(unsafe_hash=True)
class FunctionStmt(Stmt):
    name: Token
    params: List[Parameter]
    body: List[Stmt]
    is_async: bool = False
    return_type: Optional[TypeAnnotation] = None


@dataclass(unsafe_hash=True)
class ReturnStmt(Stmt):
    keyword: Token
    value: Optional[Expr]


@dataclass(unsafe_hash=True)
class ClassStmt(Stmt):
    name: Token
    methods: List[FunctionStmt]


@dataclass(unsafe_hash=True)
class ImportStmt(Stmt):
    path: Expr
    alias: Optional[Token]


@dataclass(unsafe_hash=True)
class BreakStmt(Stmt):
    keyword: Token


@dataclass(unsafe_hash=True)
class ContinueStmt(Stmt):
    keyword: Token


@dataclass(unsafe_hash=True)
class MacroStmt(Stmt):
    name: Token
    params: List[Parameter]
    body: List[Stmt]


# Type annotation helper functions
def ast_type_int() -> TypeAnnotation:
    return TypeAnnotation(kind="int")

def ast_type_float() -> TypeAnnotation:
    return TypeAnnotation(kind="float")

def ast_type_bool() -> TypeAnnotation:
    return TypeAnnotation(kind="bool")

def ast_type_string() -> TypeAnnotation:
    return TypeAnnotation(kind="string")

def ast_type_array(element_type: TypeAnnotation) -> TypeAnnotation:
    return TypeAnnotation(kind="array", element_type=element_type)

def ast_type_dict(key_type: TypeAnnotation, value_type: TypeAnnotation) -> TypeAnnotation:
    return TypeAnnotation(kind="dict", key_type=key_type, value_type=value_type)

def ast_type_void() -> TypeAnnotation:
    return TypeAnnotation(kind="void")

def ast_type_any() -> TypeAnnotation:
    return TypeAnnotation(kind="any")

def ast_type_optional(inner: TypeAnnotation) -> TypeAnnotation:
    return TypeAnnotation(kind="optional", element_type=inner)

def ast_type_result(ok: TypeAnnotation, err: TypeAnnotation) -> TypeAnnotation:
    return TypeAnnotation(kind="result", ok_type=ok, err_type=err)

def ast_type_void() -> TypeAnnotation:
    return TypeAnnotation(kind="void")

def ast_type_any() -> TypeAnnotation:
    return TypeAnnotation(kind="any")

def ast_type_custom(name: str) -> TypeAnnotation:
    return TypeAnnotation(kind="custom", custom_name=name)