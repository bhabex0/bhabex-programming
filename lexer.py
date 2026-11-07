from typing import List, Optional
from .tokens import Token, TokenType, KEYWORDS


class Lexer:
    def __init__(self, source: str):
        self.source = source
        self.tokens: List[Token] = []
        self.start = 0
        self.current = 0
        self.line = 1
        self.column = 1

    def scan_tokens(self) -> List[Token]:
        while not self._is_at_end():
            self.start = self.current
            self._scan_token()

        self.tokens.append(Token(TokenType.EOF, "", None, self.line, self.column))
        return self.tokens

    def _is_at_end(self) -> bool:
        return self.current >= len(self.source)

    def _scan_token(self):
        c = self._advance()

        if c == '(':
            self._add_token(TokenType.LPAREN)
        elif c == ')':
            self._add_token(TokenType.RPAREN)
        elif c == '{':
            self._add_token(TokenType.LBRACE)
        elif c == '}':
            self._add_token(TokenType.RBRACE)
        elif c == '[':
            self._add_token(TokenType.LBRACKET)
        elif c == ']':
            self._add_token(TokenType.RBRACKET)
        elif c == ',':
            self._add_token(TokenType.COMMA)
        elif c == ':':
            self._add_token(TokenType.COLON)
        elif c == ';':
            self._add_token(TokenType.SEMICOLON)
        elif c == '+':
            if self._match('='):
                self._add_token(TokenType.PLUS_ASSIGN)
            else:
                self._add_token(TokenType.PLUS)
        elif c == '-':
            if self._match('='):
                self._add_token(TokenType.MINUS_ASSIGN)
            elif self._match('>'):
                self._add_token(TokenType.ARROW)
            else:
                self._add_token(TokenType.MINUS)
        elif c == '*':
            if self._match('='):
                self._add_token(TokenType.STAR_ASSIGN)
            elif self._match('*'):
                self._add_token(TokenType.POWER)
            else:
                self._add_token(TokenType.STAR)
        elif c == '/':
            if self._match('='):
                self._add_token(TokenType.SLASH_ASSIGN)
            elif self._match('/'):
                self._skip_line_comment()
            elif self._match('*'):
                self._skip_block_comment()
            else:
                self._add_token(TokenType.SLASH)
        elif c == '%':
            self._add_token(TokenType.PERCENT)
        elif c == '=':
            if self._match('='):
                self._add_token(TokenType.EQ)
            elif self._match('>'):
                self._add_token(TokenType.ARROW)
            else:
                self._add_token(TokenType.ASSIGN)
        elif c == '!':
            if self._match('='):
                self._add_token(TokenType.NEQ)
            else:
                self._add_token(TokenType.NOT)
        elif c == '<':
            if self._match('='):
                self._add_token(TokenType.LTE)
            else:
                self._add_token(TokenType.LT)
        elif c == '>':
            if self._match('='):
                self._add_token(TokenType.GTE)
            else:
                self._add_token(TokenType.GT)
        elif c == '.':
            if self._match('.'):
                self._add_token(TokenType.RANGE)
            else:
                self._add_token(TokenType.DOT)
        elif c == '#':
            self._skip_line_comment()
        elif c in ' \t\r':
            pass
        elif c == '\n':
            self.line += 1
            self.column = 1
        elif c == 'r' and self._peek() == '"':
            self._advance()  # consume 'r'
            self._raw_string()
        elif c == '"':
            self._string()
        elif c.isdigit():
            self._number()
        elif c.isalpha() or c == '_':
            self._identifier()
        else:
            self._error(f"Unexpected character: {c}")

    def _advance(self) -> str:
        c = self.source[self.current]
        self.current += 1
        self.column += 1
        return c

    def _peek(self) -> Optional[str]:
        if self._is_at_end():
            return None
        return self.source[self.current]

    def _peek_next(self) -> Optional[str]:
        if self.current + 1 >= len(self.source):
            return None
        return self.source[self.current + 1]

    def _match(self, expected: str) -> bool:
        if self._is_at_end():
            return False
        if self.source[self.current] != expected:
            return False
        self.current += 1
        self.column += 1
        return True

    def _add_token(self, token_type: TokenType, literal: Optional[object] = None):
        text = self.source[self.start:self.current]
        self.tokens.append(Token(token_type, text, literal, self.line, self.column - len(text)))

    def _string(self):
        while self._peek() != '"' and not self._is_at_end():
            if self._peek() == '\n':
                self.line += 1
                self.column = 1
            self._advance()

        if self._is_at_end():
            self._error("Unterminated string")
            return

        self._advance()
        value = self.source[self.start + 1:self.current - 1]
        self._add_token(TokenType.STRING, value)

    def _raw_string(self):
        while self._peek() != '"' and not self._is_at_end():
            if self._peek() == '\n':
                self.line += 1
                self.column = 1
            self._advance()

        if self._is_at_end():
            self._error("Unterminated raw string")
            return

        self._advance()
        value = self.source[self.start + 2:self.current - 1]
        self._add_token(TokenType.STRING, value)

    def _number(self):
        while self._peek() is not None and self._peek().isdigit():
            self._advance()

        if self._peek() == '.' and self._peek_next() is not None and self._peek_next().isdigit():
            self._advance()
            while self._peek() is not None and self._peek().isdigit():
                self._advance()

            value = float(self.source[self.start:self.current])
            self._add_token(TokenType.NUMBER, value)
        else:
            value = int(self.source[self.start:self.current])
            self._add_token(TokenType.NUMBER, value)

    def _identifier(self):
        while self._peek() is not None and (self._peek().isalnum() or self._peek() == '_'):
            self._advance()

        text = self.source[self.start:self.current]
        token_type = KEYWORDS.get(text, TokenType.IDENTIFIER)
        self._add_token(token_type)

    def _skip_line_comment(self):
        while self._peek() is not None and self._peek() != '\n':
            self._advance()

    def _skip_block_comment(self):
        while not self._is_at_end():
            if self._peek() == '*' and self._peek_next() == '/':
                self._advance()
                self._advance()
                return
            if self._peek() == '\n':
                self.line += 1
                self.column = 1
            self._advance()
        self._error("Unterminated block comment")

    def _error(self, message: str):
        raise SyntaxError(f"[Line {self.line}:{self.column}] {message}")


def tokenize(source: str) -> List[Token]:
    lexer = Lexer(source)
    return lexer.scan_tokens()