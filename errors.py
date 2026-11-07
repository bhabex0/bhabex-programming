from .tokens import Token


class BhabexError(Exception):
    def __init__(self, message: str):
        self.message = message
        super().__init__(message)


class LexerError(BhabexError):
    def __init__(self, line: int, column: int, message: str):
        self.line = line
        self.column = column
        super().__init__(f"[Line {line}:{column}] {message}")


class ParseError(BhabexError):
    def __init__(self, token: Token, message: str):
        self.token = token
        if token.type.name == "EOF":
            super().__init__(f"[Line {token.line}:{token.column}] {message} at end")
        else:
            super().__init__(f"[Line {token.line}:{token.column}] {message} at '{token.lexeme}'")


class RuntimeError(BhabexError):
    def __init__(self, token: Token, message: str):
        self.token = token
        super().__init__(f"[Line {token.line}:{token.column}] {message}")


class ReturnException(Exception):
    def __init__(self, value):
        self.value = value
        super().__init__("return")


class BreakException(Exception):
    pass


class ContinueException(Exception):
    pass


class YieldException(Exception):
    def __init__(self, value):
        self.value = value
        super().__init__("yield")