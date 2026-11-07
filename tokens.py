from enum import Enum, auto
from dataclasses import dataclass
from typing import Optional, Any


class TokenType(Enum):
    EOF = auto()

    # Literals
    IDENTIFIER = auto()
    NUMBER = auto()
    STRING = auto()

    # Keywords
    FN = auto()
    LET = auto()
    IF = auto()
    ELSE = auto()
    FOR = auto()
    IN = auto()
    WHILE = auto()
    RETURN = auto()
    CLASS = auto()
    NEW = auto()
    SELF = auto()
    THIS = auto()
    IMPORT = auto()
    AS = auto()
    MATCH = auto()
    DEFAULT = auto()
    TRY = auto()
    CATCH = auto()
    THROW = auto()
    TRUE = auto()
    FALSE = auto()
    NIL = auto()
    AND = auto()
    OR = auto()
    NOT = auto()
    BREAK = auto()
    CONTINUE = auto()
    PRINT = auto()
    ASYNC = auto()
    AWAIT = auto()
    YIELD = auto()

    # Type annotations
    TYPE_INT = auto()
    TYPE_FLOAT = auto()
    TYPE_BOOL = auto()
    TYPE_STRING = auto()

    # Macro
    MACRO = auto()
    TYPE_ARRAY = auto()
    TYPE_DICT = auto()
    TYPE_VOID = auto()
    TYPE_ANY = auto()
    TYPE_OPTIONAL = auto()
    TYPE_RESULT = auto()
    TYPE_FN = auto()

    # Operators
    PLUS = auto()
    MINUS = auto()
    STAR = auto()
    SLASH = auto()
    PERCENT = auto()
    POWER = auto()

    # Comparison
    EQ = auto()
    NEQ = auto()
    LT = auto()
    GT = auto()
    LTE = auto()
    GTE = auto()

    # Assignment
    ASSIGN = auto()
    PLUS_ASSIGN = auto()
    MINUS_ASSIGN = auto()
    STAR_ASSIGN = auto()
    SLASH_ASSIGN = auto()

    # Delimiters
    LPAREN = auto()
    RPAREN = auto()
    LBRACE = auto()
    RBRACE = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    COMMA = auto()
    DOT = auto()
    COLON = auto()
    SEMICOLON = auto()
    ARROW = auto()
    RANGE = auto()

    # Comments
    COMMENT = auto()


KEYWORDS = {
    "fn": TokenType.FN,
    "let": TokenType.LET,
    "if": TokenType.IF,
    "else": TokenType.ELSE,
    "for": TokenType.FOR,
    "in": TokenType.IN,
    "while": TokenType.WHILE,
    "return": TokenType.RETURN,
    "class": TokenType.CLASS,
    "new": TokenType.NEW,
    "self": TokenType.SELF,
    "this": TokenType.THIS,
    "import": TokenType.IMPORT,
    "as": TokenType.AS,
    "match": TokenType.MATCH,
    "default": TokenType.DEFAULT,
    "try": TokenType.TRY,
    "catch": TokenType.CATCH,
    "throw": TokenType.THROW,
    "true": TokenType.TRUE,
    "false": TokenType.FALSE,
    "nil": TokenType.NIL,
    "and": TokenType.AND,
    "or": TokenType.OR,
    "not": TokenType.NOT,
    "break": TokenType.BREAK,
    "continue": TokenType.CONTINUE,
    "print": TokenType.PRINT,
    "async": TokenType.ASYNC,
    "await": TokenType.AWAIT,
    "yield": TokenType.YIELD,
    "int": TokenType.TYPE_INT,
    "float": TokenType.TYPE_FLOAT,
    "bool": TokenType.TYPE_BOOL,
    "string": TokenType.TYPE_STRING,
    "array": TokenType.TYPE_ARRAY,
    "dict": TokenType.TYPE_DICT,
    "void": TokenType.TYPE_VOID,
    "any": TokenType.TYPE_ANY,
    "Optional": TokenType.TYPE_OPTIONAL,
    "Result": TokenType.TYPE_RESULT,
    "macro": TokenType.MACRO,
}


@dataclass(unsafe_hash=True)
class Token:
    type: TokenType
    lexeme: str
    literal: Optional[Any] = None
    line: int = 1
    column: int = 1

    def __str__(self) -> str:
        return f"{self.type.name} '{self.lexeme}' ({self.line}:{self.column})"