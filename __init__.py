"""
Bhabex - A modern, multi-paradigm programming language (.bx)
"""

__version__ = "0.1.0"
__author__ = "bhabex"

from .tokens import Token, TokenType
from .lexer import Lexer, tokenize
from .parser import parse, Parser
from .ast_nodes import Program
from .interpreter import Interpreter
from .repl import REPL
from .errors import BhabexError, LexerError, ParseError, RuntimeError
from .environment import Environment
from .builtins import create_globals, BuiltinFunction, BuiltinClass, BuiltinInstance

__all__ = [
    "Token",
    "TokenType",
    "tokenize",
    "Lexer",
    "parse",
    "Parser",
    "Program",
    "Interpreter",
    "REPL",
    "BhabexError",
    "LexerError",
    "ParseError",
    "RuntimeError",
    "Environment",
    "create_globals",
    "BuiltinFunction",
    "BuiltinClass",
    "BuiltinInstance",
]