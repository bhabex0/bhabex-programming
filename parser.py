from typing import List, Optional
from .tokens import Token, TokenType
from .ast_nodes import (
    Program, Literal, Identifier, Binary, Unary, Grouping, Call, Get, Set,
    This, Super, IndexGet, IndexSet, ListLiteral, DictLiteral, Range,
    Lambda, MatchExpr, MatchCase, TryExpr, ThrowExpr, AwaitExpr, YieldExpr, MacroStmt,
    ExpressionStmt, PrintStmt, VarDecl, Assign, Block, IfStmt,
    WhileStmt, ForStmt, FunctionStmt, ReturnStmt, ClassStmt,
    ImportStmt, BreakStmt, ContinueStmt,
    Parameter, TypeAnnotation,
    ast_type_int, ast_type_float, ast_type_bool, ast_type_string,
    ast_type_array, ast_type_dict, ast_type_void, ast_type_any,
    ast_type_optional, ast_type_result, ast_type_custom
)
from .errors import ParseError


class Parser:
    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.current = 0

    def parse(self) -> Program:
        statements = []
        while not self._is_at_end():
            statements.append(self._declaration())
        return Program(statements)

    def _declaration(self):
        try:
            if self._match(TokenType.FN):
                return self._function_declaration()
            if self._match(TokenType.LET):
                return self._var_declaration()
            if self._match(TokenType.CLASS):
                return self._class_declaration()
            if self._match(TokenType.IMPORT):
                return self._import_declaration()
            if self._match(TokenType.MACRO):
                return self._macro_declaration()
            if self._match(TokenType.PRINT):
                return self._print_statement()
            return self._statement()
        except ParseError:
            self._synchronize()
            return None

    def _statement(self):
        if self._match(TokenType.IF):
            return self._if_statement()
        if self._match(TokenType.WHILE):
            return self._while_statement()
        if self._match(TokenType.FOR):
            return self._for_statement()
        if self._match(TokenType.RETURN):
            return self._return_statement()
        if self._match(TokenType.BREAK):
            return BreakStmt(self._previous())
        if self._match(TokenType.CONTINUE):
            return ContinueStmt(self._previous())
        if self._match(TokenType.TRY):
            return self._try_statement()
        if self._match(TokenType.THROW):
            return self._throw_statement()
        if self._match(TokenType.MATCH):
            return self._match_statement()
        if self._match(TokenType.LBRACE):
            return Block(self._block())
        if self._match(TokenType.PRINT):
            return self._print_statement()
        return self._expression_statement()

    def _match_statement(self):
        expr = self._match_expr()
        return ExpressionStmt(expr)

    def _try_statement(self):
        self._consume(TokenType.LBRACE, "Expect '{' after 'try'.")
        body = self._block()
        catch_var = None
        catch_body = []
        
        if self._match(TokenType.CATCH):
            if self._match(TokenType.LPAREN):
                catch_var = self._consume(TokenType.IDENTIFIER, "Expect identifier after 'catch'.")
                self._consume(TokenType.RPAREN, "Expect ')' after catch variable.")
            self._consume(TokenType.LBRACE, "Expect '{' after 'catch'.")
            catch_body = self._block()
        
        return TryExpr(body, catch_var, catch_body, self._previous())

    def _throw_statement(self):
        value = self._expression()
        self._match(TokenType.SEMICOLON)
        return ThrowExpr(value, self._previous())

    def _print_statement(self):
        self._consume(TokenType.LPAREN, "Expect '(' after 'print'.")
        values = []
        if not self._check(TokenType.RPAREN):
            values.append(self._expression())
            while self._match(TokenType.COMMA):
                values.append(self._expression())
        self._consume(TokenType.RPAREN, "Expect ')' after print arguments.")
        self._match(TokenType.SEMICOLON)
        return PrintStmt(values)

    def _expression_statement(self):
        expr = self._expression()
        self._match(TokenType.SEMICOLON)
        return ExpressionStmt(expr)

    def _var_declaration(self):
        name = self._consume(TokenType.IDENTIFIER, "Expect variable name.")
        
        type_annotation = None
        if self._match(TokenType.COLON):
            type_annotation = self._parse_type_annotation()
        
        initializer = None
        if self._match(TokenType.ASSIGN):
            initializer = self._expression()
        self._match(TokenType.SEMICOLON)
        return VarDecl(name, type_annotation, initializer)

    def _function_declaration(self):
        is_async = False
        if self._match(TokenType.ASYNC):
            is_async = True
        
        name = self._consume(TokenType.IDENTIFIER, "Expect function name.")
        self._consume(TokenType.LPAREN, "Expect '(' after function name.")
        params = []
        if not self._check(TokenType.RPAREN):
            params.append(self._parse_parameter())
            while self._match(TokenType.COMMA):
                params.append(self._parse_parameter())
        self._consume(TokenType.RPAREN, "Expect ')' after parameters.")
        
        return_type = None
        if self._match(TokenType.ARROW):
            return_type = self._parse_type_annotation()
        
        self._consume(TokenType.LBRACE, "Expect '{' before function body.")
        body = self._block()
        return FunctionStmt(name, params, body, is_async, return_type)
    
    def _parse_parameter(self):
        name = self._consume(TokenType.IDENTIFIER, "Expect parameter name.")
        type_annotation = None
        if self._match(TokenType.COLON):
            type_annotation = self._parse_type_annotation()
        return Parameter(name, type_annotation)
    
    def _parse_type_annotation(self):
        # Simple type annotation parsing
        if self._match(TokenType.TYPE_INT):
            return ast_type_int()
        if self._match(TokenType.TYPE_FLOAT):
            return ast_type_float()
        if self._match(TokenType.TYPE_BOOL):
            return ast_type_bool()
        if self._match(TokenType.TYPE_STRING):
            return ast_type_string()
        if self._match(TokenType.TYPE_ARRAY):
            self._consume(TokenType.LBRACKET, "Expect '[' after 'array'")
            element_type = self._parse_type_annotation()
            self._consume(TokenType.RBRACKET, "Expect ']' after array element type")
            return ast_type_array(element_type)
        if self._match(TokenType.TYPE_DICT):
            self._consume(TokenType.LBRACKET, "Expect '[' after 'dict'")
            key_type = self._parse_type_annotation()
            self._consume(TokenType.COMMA, "Expect ',' after dict key type")
            value_type = self._parse_type_annotation()
            self._consume(TokenType.RBRACKET, "Expect ']' after dict value type")
            return ast_type_dict(key_type, value_type)
        if self._match(TokenType.TYPE_VOID):
            return ast_type_void()
        if self._match(TokenType.TYPE_ANY):
            return ast_type_any()
        if self._match(TokenType.TYPE_OPTIONAL):
            self._consume(TokenType.LBRACKET, "Expect '[' after 'Optional'")
            inner_type = self._parse_type_annotation()
            self._consume(TokenType.RBRACKET, "Expect ']' after Optional type")
            return ast_type_optional(inner_type)
        if self._match(TokenType.TYPE_RESULT):
            self._consume(TokenType.LBRACKET, "Expect '[' after 'Result'")
            ok_type = self._parse_type_annotation()
            self._consume(TokenType.COMMA, "Expect ',' after Result ok type")
            err_type = self._parse_type_annotation()
            self._consume(TokenType.RBRACKET, "Expect ']' after Result error type")
            return ast_type_result(ok_type, err_type)
        if self._match(TokenType.IDENTIFIER):
            return ast_type_custom(self._previous().lexeme)
        self._error(self._peek(), "Expect type annotation.")
        return None

    def _macro_declaration(self):
        name = self._consume(TokenType.IDENTIFIER, "Expect macro name.")
        self._consume(TokenType.LPAREN, "Expect '(' after macro name.")
        params = []
        if not self._check(TokenType.RPAREN):
            params.append(self._parse_parameter())
            while self._match(TokenType.COMMA):
                params.append(self._parse_parameter())
        self._consume(TokenType.RPAREN, "Expect ')' after parameters.")
        self._consume(TokenType.LBRACE, "Expect '{' before macro body.")
        body = self._block()
        return MacroStmt(name, params, body)

    def _class_declaration(self):
        name = self._consume(TokenType.IDENTIFIER, "Expect class name.")
        self._consume(TokenType.LBRACE, "Expect '{' before class body.")
        methods = []
        while not self._check(TokenType.RBRACE) and not self._is_at_end():
            if self._match(TokenType.FN):
                methods.append(self._function_declaration())
            else:
                self._error(self._peek(), "Expect 'fn' before method name.")
        self._consume(TokenType.RBRACE, "Expect '}' after class body.")
        return ClassStmt(name, methods)

    def _import_declaration(self):
        path = self._expression()
        alias = None
        if self._match(TokenType.AS):
            alias = self._consume(TokenType.IDENTIFIER, "Expect alias name after 'as'.")
        self._match(TokenType.SEMICOLON)
        return ImportStmt(path, alias)

    def _if_statement(self):
        if self._match(TokenType.LPAREN):
            condition = self._expression()
            self._consume(TokenType.RPAREN, "Expect ')' after if condition.")
        else:
            condition = self._expression()
        then_branch = self._statement()
        else_branch = None
        if self._match(TokenType.ELSE):
            else_branch = self._statement()
        return IfStmt(condition, then_branch, else_branch)

    def _while_statement(self):
        if self._match(TokenType.LPAREN):
            condition = self._expression()
            self._consume(TokenType.RPAREN, "Expect ')' after while condition.")
        else:
            condition = self._expression()
        body = self._statement()
        return WhileStmt(condition, body)

    def _for_statement(self):
        var = self._consume(TokenType.IDENTIFIER, "Expect loop variable name.")
        self._consume(TokenType.IN, "Expect 'in' after loop variable.")
        iterable = self._expression()
        body = self._statement()
        return ForStmt(var, iterable, body)

    def _return_statement(self):
        keyword = self._previous()
        value = None
        if not self._check(TokenType.SEMICOLON) and not self._check(TokenType.RBRACE):
            value = self._expression()
        if self._match(TokenType.SEMICOLON):
            pass
        return ReturnStmt(keyword, value)

    def _print_statement(self):
        self._consume(TokenType.LPAREN, "Expect '(' after 'print'.")
        values = []
        if not self._check(TokenType.RPAREN):
            values.append(self._expression())
            while self._match(TokenType.COMMA):
                values.append(self._expression())
        self._consume(TokenType.RPAREN, "Expect ')' after print arguments.")
        self._match(TokenType.SEMICOLON)
        return PrintStmt(values)

    def _expression_statement(self):
        expr = self._expression()
        self._match(TokenType.SEMICOLON)
        return ExpressionStmt(expr)

    def _block(self) -> List:
        statements = []
        while not self._check(TokenType.RBRACE) and not self._is_at_end():
            stmt = self._declaration()
            if stmt:
                statements.append(stmt)
        self._consume(TokenType.RBRACE, "Expect '}' after block.")
        return statements

    def _expression(self):
        return self._assignment()

    def _assignment(self):
        expr = self._or()

        if self._match(TokenType.ASSIGN):
            equals = self._previous()
            value = self._assignment()
            if isinstance(expr, Identifier):
                return Assign(expr.token, value)
            elif isinstance(expr, Get):
                return Set(expr.object, expr.name, value)
            elif isinstance(expr, IndexGet):
                return IndexSet(expr.object, expr.index, value, expr.bracket)
            self._error(equals, "Invalid assignment target.")

        if self._match(TokenType.PLUS_ASSIGN):
            return self._compound_assignment(expr, TokenType.PLUS)
        if self._match(TokenType.MINUS_ASSIGN):
            return self._compound_assignment(expr, TokenType.MINUS)
        if self._match(TokenType.STAR_ASSIGN):
            return self._compound_assignment(expr, TokenType.STAR)
        if self._match(TokenType.SLASH_ASSIGN):
            return self._compound_assignment(expr, TokenType.SLASH)

        return expr

    def _compound_assignment(self, expr, op_type):
        value = self._assignment()
        if isinstance(expr, Identifier):
            return Assign(expr.token, Binary(expr, Token(op_type, ""), value))
        elif isinstance(expr, Get):
            return Set(expr.object, expr.name, Binary(expr, Token(op_type, ""), value))
        elif isinstance(expr, IndexGet):
            return IndexSet(expr.object, expr.index, Binary(expr, Token(op_type, ""), value), expr.bracket)
        self._error(self._previous(), "Invalid assignment target.")

    def _or(self):
        expr = self._and()
        while self._match(TokenType.OR):
            operator = self._previous()
            right = self._and()
            expr = Binary(expr, operator, right)
        return expr

    def _and(self):
        expr = self._equality()
        while self._match(TokenType.AND):
            operator = self._previous()
            right = self._equality()
            expr = Binary(expr, operator, right)
        return expr

    def _equality(self):
        expr = self._comparison()
        while self._match(TokenType.EQ, TokenType.NEQ):
            operator = self._previous()
            right = self._comparison()
            expr = Binary(expr, operator, right)
        return expr

    def _comparison(self):
        expr = self._range()
        while self._match(TokenType.GT, TokenType.GTE, TokenType.LT, TokenType.LTE):
            operator = self._previous()
            right = self._range()
            expr = Binary(expr, operator, right)
        return expr

    def _range(self):
        expr = self._term()
        while self._match(TokenType.RANGE):
            operator = self._previous()
            right = self._term()
            expr = Range(expr, right, False, operator)
        return expr

    def _term(self):
        expr = self._factor()
        while self._match(TokenType.PLUS, TokenType.MINUS):
            operator = self._previous()
            right = self._factor()
            expr = Binary(expr, operator, right)
        return expr

    def _factor(self):
        expr = self._unary()
        while self._match(TokenType.STAR, TokenType.SLASH, TokenType.PERCENT):
            operator = self._previous()
            right = self._unary()
            expr = Binary(expr, operator, right)
        return expr

    def _unary(self):
        if self._match(TokenType.MINUS, TokenType.NOT):
            operator = self._previous()
            right = self._unary()
            return Unary(operator, right)
        return self._call()

    def _call(self):
        expr = self._primary()
        while True:
            if self._match(TokenType.LPAREN):
                expr = self._finish_call(expr)
            elif self._match(TokenType.DOT):
                name = self._consume(TokenType.IDENTIFIER, "Expect property name after '.'.")
                expr = Get(expr, name)
            elif self._match(TokenType.LBRACKET):
                index = self._expression()
                bracket = self._consume(TokenType.RBRACKET, "Expect ']' after index.")
                expr = IndexGet(expr, index, bracket)
            else:
                break
        return expr

    def _finish_call(self, callee):
        arguments = []
        if not self._check(TokenType.RPAREN):
            arguments.append(self._expression())
            while self._match(TokenType.COMMA):
                arguments.append(self._expression())
        paren = self._consume(TokenType.RPAREN, "Expect ')' after arguments.")
        return Call(callee, paren, arguments)

    def _primary(self):
        if self._match(TokenType.FALSE):
            return Literal(False, self._previous())
        if self._match(TokenType.TRUE):
            return Literal(True, self._previous())
        if self._match(TokenType.NIL):
            return Literal(None, self._previous())

        if self._match(TokenType.NUMBER, TokenType.STRING):
            return Literal(self._previous().literal, self._previous())

        if self._match(TokenType.SELF):
            return This(self._previous())

        if self._match(TokenType.THIS):
            return This(self._previous())

        if self._match(TokenType.NEW):
            name = self._consume(TokenType.IDENTIFIER, "Expect class name after 'new'.")
            self._consume(TokenType.LPAREN, "Expect '(' after class name.")
            arguments = []
            if not self._check(TokenType.RPAREN):
                arguments.append(self._expression())
                while self._match(TokenType.COMMA):
                    arguments.append(self._expression())
            paren = self._consume(TokenType.RPAREN, "Expect ')' after arguments.")
            return Call(Identifier(name.lexeme, name), paren, arguments)

        if self._match(TokenType.IDENTIFIER):
            return Identifier(self._previous().lexeme, self._previous())

        if self._match(TokenType.LPAREN):
            expr = self._expression()
            self._consume(TokenType.RPAREN, "Expect ')' after expression.")
            return Grouping(expr)

        if self._match(TokenType.LBRACKET):
            return self._list_literal()

        if self._match(TokenType.LBRACE):
            return self._dict_literal()

        if self._match(TokenType.FN):
            return self._lambda()

        if self._match(TokenType.MATCH):
            return self._match_expr()

        if self._match(TokenType.AWAIT):
            value = self._unary()
            return AwaitExpr(value, self._previous())

        if self._match(TokenType.YIELD):
            value = None
            if not self._check(TokenType.SEMICOLON) and not self._check(TokenType.RBRACE):
                value = self._expression()
            return YieldExpr(value, self._previous())

        self._error(self._peek(), "Expect expression.")
        return None

    def _list_literal(self):
        elements = []
        bracket = self._previous()
        if not self._check(TokenType.RBRACKET):
            elements.append(self._expression())
            while self._match(TokenType.COMMA):
                elements.append(self._expression())
        self._consume(TokenType.RBRACKET, "Expect ']' after list elements.")
        return ListLiteral(elements, bracket)

    def _dict_literal(self):
        keys = []
        values = []
        brace = self._previous()
        if not self._check(TokenType.RBRACE):
            keys.append(self._expression())
            self._consume(TokenType.COLON, "Expect ':' after key.")
            values.append(self._expression())
            while self._match(TokenType.COMMA):
                keys.append(self._expression())
                self._consume(TokenType.COLON, "Expect ':' after key.")
                values.append(self._expression())
        self._consume(TokenType.RBRACE, "Expect '}' after dict elements.")
        return DictLiteral(keys, values, brace)

    def _lambda(self):
        token = self._previous()
        params = []
        self._consume(TokenType.LPAREN, "Expect '(' after 'fn'.")
        if not self._check(TokenType.RPAREN):
            params.append(self._consume(TokenType.IDENTIFIER, "Expect parameter name."))
            while self._match(TokenType.COMMA):
                params.append(self._consume(TokenType.IDENTIFIER, "Expect parameter name."))
        self._consume(TokenType.RPAREN, "Expect ')' after parameters.")
        self._consume(TokenType.LBRACE, "Expect '{' before lambda body.")
        body = self._block()
        return Lambda(params, body, token)

    def _match_expr(self):
        token = self._previous()
        subject = self._expression()
        self._consume(TokenType.LBRACE, "Expect '{' after match expression.")
        cases = []
        while not self._check(TokenType.RBRACE) and not self._is_at_end():
            cases.append(self._match_case())
        self._consume(TokenType.RBRACE, "Expect '}' after match cases.")
        return MatchExpr(subject, cases, token)

    def _match_case(self):
        if self._match(TokenType.DEFAULT):
            pattern = Literal("default", self._previous())
        else:
            pattern = self._expression()
        self._consume(TokenType.ARROW, "Expect '=>' after match pattern.")
        if self._match(TokenType.LBRACE):
            body = self._block()
        else:
            body = [self._statement()]
        return MatchCase(pattern, body)

    def _match(self, *types: TokenType) -> bool:
        for t in types:
            if self._check(t):
                self._advance()
                return True
        return False

    def _check(self, token_type: TokenType) -> bool:
        if self._is_at_end():
            return False
        return self._peek().type == token_type

    def _advance(self) -> Token:
        if not self._is_at_end():
            self.current += 1
        return self._previous()

    def _is_at_end(self) -> bool:
        return self._peek().type == TokenType.EOF

    def _peek(self) -> Token:
        return self.tokens[self.current]

    def _previous(self) -> Token:
        return self.tokens[self.current - 1]

    def _consume(self, token_type: TokenType, message: str) -> Token:
        if self._check(token_type):
            return self._advance()
        raise self._error(self._peek(), message)

    def _error(self, token: Token, message: str) -> ParseError:
        raise ParseError(token, message)

    def _synchronize(self):
        self._advance()
        while not self._is_at_end():
            if self._previous().type == TokenType.SEMICOLON:
                return
            if self._peek().type in (TokenType.FN, TokenType.LET, TokenType.CLASS, 
                                     TokenType.IF, TokenType.WHILE, TokenType.FOR,
                                     TokenType.RETURN, TokenType.IMPORT, TokenType.MACRO):
                return
            self._advance()


def parse(tokens: List[Token]) -> Program:
    parser = Parser(tokens)
    return parser.parse()