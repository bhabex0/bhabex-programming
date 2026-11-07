from typing import Any, List, Dict, Optional
from .tokens import Token, TokenType
from .ast_nodes import (
    Program, Literal, Identifier, Binary, Unary, Grouping, Call, Get, Set,
    This, Super, IndexGet, IndexSet, ListLiteral, DictLiteral, Range,
    Lambda, MatchExpr, MatchCase, TryExpr, ThrowExpr, AwaitExpr, YieldExpr, MacroStmt,
    ExpressionStmt, PrintStmt, VarDecl, Assign, Block, IfStmt,
    WhileStmt, ForStmt, FunctionStmt, ReturnStmt, ClassStmt,
    ImportStmt, BreakStmt, ContinueStmt
)
from .environment import Environment
from .errors import RuntimeError, ReturnException, BreakException, ContinueException, YieldException
from .builtins import BuiltinFunction, BuiltinClass, BuiltinInstance, BoundMethod, create_globals


class Interpreter:
    def __init__(self):
        self.globals = create_globals()
        self.environment = self.globals
        self.locals: Dict[Expr, int] = {}

    def interpret(self, program: Program) -> Any:
        for statement in program.statements:
            if statement:
                self._execute(statement)
        
        # Call main function if it exists
        try:
            main_fn = self.environment.get_at(0, "main")
            if hasattr(main_fn, 'call'):
                try:
                    main_fn.call(self, [])
                except ReturnException as e:
                    return e.value
        except:
            pass  # main not defined or not callable

    def _execute(self, stmt: Stmt) -> Any:
        method_name = f"_visit_{type(stmt).__name__}"
        visitor = getattr(self, method_name, None)
        if visitor:
            return visitor(stmt)
        raise RuntimeError(Token(TokenType.EOF, "", None, 0, 0), f"No visitor for {type(stmt).__name__}")

    def _evaluate(self, expr: Expr) -> Any:
        method_name = f"_visit_{type(expr).__name__}"
        visitor = getattr(self, method_name, None)
        if visitor:
            return visitor(expr)
        raise RuntimeError(Token(TokenType.EOF, "", None, 0, 0), f"No visitor for {type(expr).__name__}")

    def _visit_Program(self, stmt: Program):
        for statement in stmt.statements:
            if statement:
                self._execute(statement)

    def _visit_ExpressionStmt(self, stmt: ExpressionStmt):
        self._evaluate(stmt.expression)

    def _visit_PrintStmt(self, stmt: PrintStmt):
        for i, expr in enumerate(stmt.expressions):
            value = self._evaluate(expr)
            if i > 0:
                print(" ", end="")
            print(self._stringify(value), end="")
        print()

    def _visit_VarDecl(self, stmt: VarDecl):
        value = None
        if stmt.initializer:
            value = self._evaluate(stmt.initializer)
        self.environment.define(stmt.name.lexeme, value)

    def _visit_Assign(self, stmt: Assign):
        value = self._evaluate(stmt.value)
        self.environment.assign(stmt.name, value)

    def _visit_Block(self, stmt: Block):
        self._execute_block(stmt.statements, Environment(self.environment))

    def _execute_block(self, statements: List[Stmt], environment: Environment):
        previous = self.environment
        try:
            self.environment = environment
            for statement in statements:
                if statement:
                    self._execute(statement)
        finally:
            self.environment = previous

    def _visit_IfStmt(self, stmt: IfStmt):
        if self._is_truthy(self._evaluate(stmt.condition)):
            self._execute(stmt.then_branch)
        elif stmt.else_branch:
            self._execute(stmt.else_branch)

    def _visit_WhileStmt(self, stmt: WhileStmt):
        while self._is_truthy(self._evaluate(stmt.condition)):
            try:
                self._execute(stmt.body)
            except BreakException:
                break
            except ContinueException:
                continue

    def _visit_ForStmt(self, stmt: ForStmt):
        iterable = self._evaluate(stmt.iterable)
        if not hasattr(iterable, '__iter__'):
            raise RuntimeError(stmt.var, f"'{type(iterable).__name__}' object is not iterable")
        
        for item in iterable:
            self.environment.define(stmt.var.lexeme, item)
            try:
                self._execute(stmt.body)
            except BreakException:
                break
            except ContinueException:
                continue

    def _visit_FunctionStmt(self, stmt: FunctionStmt):
        function = BhabexFunction(stmt, self.environment, False)
        function.is_async = stmt.is_async
        self.environment.define(stmt.name.lexeme, function)

    def _visit_MacroStmt(self, stmt: MacroStmt):
        # Macros are expanded at compile time, but for simplicity we'll
        # store them as callable objects that can be invoked
        macro = BhabexMacro(stmt, self.environment)
        self.environment.define(stmt.name.lexeme, macro)

    def _visit_ReturnStmt(self, stmt: ReturnStmt):
        value = None
        if stmt.value:
            value = self._evaluate(stmt.value)
        raise ReturnException(value)

    def _visit_ClassStmt(self, stmt: ClassStmt):
        self.environment.define(stmt.name.lexeme, None)
        methods = {}
        for method in stmt.methods:
            function = BhabexFunction(method, self.environment, method.name.lexeme == "init")
            methods[method.name.lexeme] = function
        klass = BhabexClass(stmt.name.lexeme, methods)
        self.environment.assign(stmt.name, klass)

    def _visit_ImportStmt(self, stmt: ImportStmt):
        path = self._evaluate(stmt.path)
        if not isinstance(path, str):
            raise RuntimeError(stmt.path, "Import path must be a string")
        
        # Resolve relative path
        import os
        if not os.path.isabs(path):
            # Try relative to current working directory
            if not os.path.exists(path):
                # Try with .bx extension
                if not path.endswith('.bx'):
                    path = path + '.bx'
        
        if not os.path.exists(path):
            raise RuntimeError(stmt.path, f"Module not found: {path}")
        
        # Read and parse the module
        source = ""
        try:
            with open(path, 'r', encoding='utf-8') as f:
                source = f.read()
        except Exception as e:
            raise RuntimeError(stmt.path, f"Failed to read module: {e}")
        
        # Create a new interpreter for the module (with isolated environment)
        module_env = Environment(self.environment)
        module_interpreter = Interpreter()
        module_interpreter.environment = module_env
        module_interpreter.globals = self.globals
        
        # Parse and execute the module
        tokens = self._tokenize_for_import(source)
        program = self._parse_for_import(tokens)
        module_interpreter.interpret(program)
        
        # Get the module's exports (all non-private definitions)
        exports = {}
        for name, value in module_env.values.items():
            if not name.startswith('_'):
                exports[name] = value
        
        # Create module object
        module_obj = {
            '__name__': path,
            '__exports__': exports
        }
        for name, value in exports.items():
            module_obj[name] = value
        
        alias = stmt.alias.lexeme if stmt.alias else os.path.basename(path).replace('.bx', '')
        self.environment.define(alias, module_obj)
    
    def _tokenize_for_import(self, source: str):
        from .lexer import tokenize
        return tokenize(source)
    
    def _parse_for_import(self, tokens):
        from .parser import parse
        return parse(tokens)

    def _visit_BreakStmt(self, stmt: BreakStmt):
        raise BreakException()

    def _visit_ContinueStmt(self, stmt: ContinueStmt):
        raise ContinueException()

    def _visit_Literal(self, expr: Literal) -> Any:
        return expr.value

    def _visit_Identifier(self, expr: Identifier) -> Any:
        return self.environment.get(expr.token)

    def _visit_Binary(self, expr: Binary) -> Any:
        left = self._evaluate(expr.left)
        right = self._evaluate(expr.right)

        op_type = expr.operator.type

        if op_type == TokenType.PLUS:
            if isinstance(left, (int, float)) and isinstance(right, (int, float)):
                return left + right
            if isinstance(left, str) and isinstance(right, str):
                return left + right
            if isinstance(left, list) and isinstance(right, list):
                return left + right
            raise RuntimeError(expr.operator, "Operands must be two numbers, two strings, or two lists.")

        if op_type == TokenType.MINUS:
            self._check_number_operands(expr.operator, left, right)
            return left - right
        if op_type == TokenType.STAR:
            self._check_number_operands(expr.operator, left, right)
            return left * right
        if op_type == TokenType.SLASH:
            self._check_number_operands(expr.operator, left, right)
            if right == 0:
                raise RuntimeError(expr.operator, "Division by zero.")
            return left / right
        if op_type == TokenType.PERCENT:
            self._check_number_operands(expr.operator, left, right)
            return left % right
        if op_type == TokenType.POWER:
            self._check_number_operands(expr.operator, left, right)
            return left ** right

        if op_type == TokenType.EQ:
            return self._is_equal(left, right)
        if op_type == TokenType.NEQ:
            return not self._is_equal(left, right)
        if op_type == TokenType.GT:
            self._check_number_operands(expr.operator, left, right)
            return left > right
        if op_type == TokenType.GTE:
            self._check_number_operands(expr.operator, left, right)
            return left >= right
        if op_type == TokenType.LT:
            self._check_number_operands(expr.operator, left, right)
            return left < right
        if op_type == TokenType.LTE:
            self._check_number_operands(expr.operator, left, right)
            return left <= right

        return None

    def _visit_Unary(self, expr: Unary) -> Any:
        right = self._evaluate(expr.right)
        if expr.operator.type == TokenType.MINUS:
            self._check_number_operand(expr.operator, right)
            return -right
        if expr.operator.type == TokenType.NOT:
            return not self._is_truthy(right)
        return None

    def _visit_Grouping(self, expr: Grouping) -> Any:
        return self._evaluate(expr.expression)

    def _visit_This(self, expr: This) -> Any:
        return self.environment.get(expr.keyword)

    def _visit_Call(self, expr: Call) -> Any:
        callee = self._evaluate(expr.callee)
        arguments = [self._evaluate(arg) for arg in expr.arguments]

        if isinstance(callee, BuiltinFunction):
            return callee.call(self, arguments)
        if isinstance(callee, BhabexFunction):
            return callee.call(self, arguments)
        if isinstance(callee, BhabexClass):
            return callee.call(self, arguments)
        if isinstance(callee, BhabexMacro):
            return callee.call(self, arguments)
        if isinstance(callee, BoundMethod):
            return callee.call(self, arguments)

        raise RuntimeError(expr.paren, "Can only call functions and classes.")

    def _visit_Get(self, expr: Get) -> Any:
        obj = self._evaluate(expr.object)
        if isinstance(obj, BhabexInstance):
            return obj.get(expr.name)
        if isinstance(obj, BuiltinInstance):
            return obj.get(expr.name.lexeme)
        if isinstance(obj, dict):
            return obj.get(expr.name.lexeme)
        raise RuntimeError(expr.name, "Only instances have properties.")

    def _visit_Set(self, expr: Set) -> Any:
        obj = self._evaluate(expr.object)
        value = self._evaluate(expr.value)
        if isinstance(obj, BhabexInstance):
            obj.set(expr.name, value)
            return value
        if isinstance(obj, BuiltinInstance):
            obj.set(expr.name.lexeme, value)
            return value
        if isinstance(obj, dict):
            obj[expr.name.lexeme] = value
            return value
        raise RuntimeError(expr.name, "Only instances have properties.")

    def _visit_IndexGet(self, expr: IndexGet) -> Any:
        obj = self._evaluate(expr.object)
        index = self._evaluate(expr.index)
        
        if isinstance(obj, list):
            if not isinstance(index, int):
                raise RuntimeError(expr.bracket, "List index must be an integer.")
            if index < 0 or index >= len(obj):
                raise RuntimeError(expr.bracket, "List index out of bounds.")
            return obj[index]
        if isinstance(obj, dict):
            return obj.get(index)
        if isinstance(obj, str):
            if not isinstance(index, int):
                raise RuntimeError(expr.bracket, "String index must be an integer.")
            if index < 0 or index >= len(obj):
                raise RuntimeError(expr.bracket, "String index out of bounds.")
            return obj[index]
        raise RuntimeError(expr.bracket, "Only lists, dicts, and strings support indexing.")

    def _visit_IndexSet(self, expr: IndexSet) -> Any:
        obj = self._evaluate(expr.object)
        index = self._evaluate(expr.index)
        value = self._evaluate(expr.value)
        
        if isinstance(obj, list):
            if not isinstance(index, int):
                raise RuntimeError(expr.bracket, "List index must be an integer.")
            if index < 0 or index >= len(obj):
                raise RuntimeError(expr.bracket, "List index out of bounds.")
            obj[index] = value
            return value
        if isinstance(obj, dict):
            obj[index] = value
            return value
        raise RuntimeError(expr.bracket, "Only lists and dicts support index assignment.")

    def _visit_ListLiteral(self, expr: ListLiteral) -> List[Any]:
        return [self._evaluate(element) for element in expr.elements]

    def _visit_DictLiteral(self, expr: DictLiteral) -> Dict[Any, Any]:
        result = {}
        for k, v in zip(expr.keys, expr.values):
            result[self._evaluate(k)] = self._evaluate(v)
        return result

    def _visit_Range(self, expr: Range) -> range:
        start = self._evaluate(expr.start)
        end = self._evaluate(expr.end)
        self._check_number_operands(expr.token, start, end)
        if expr.inclusive:
            return range(int(start), int(end) + 1)
        return range(int(start), int(end))

    def _visit_Lambda(self, expr: Lambda) -> 'BhabexFunction':
        return BhabexFunction(expr, self.environment, False)

    def _visit_MatchExpr(self, expr: MatchExpr) -> Any:
        subject = self._evaluate(expr.subject)
        for case in expr.cases:
            pattern = self._evaluate(case.pattern)
            if self._match_pattern(subject, pattern):
                previous = self.environment
                try:
                    self.environment = Environment(previous)
                    for stmt in case.body:
                        if stmt:
                            self._execute(stmt)
                finally:
                    self.environment = previous
                return None
        return None

    def _match_pattern(self, subject: Any, pattern: Any) -> bool:
        if pattern == "default" or isinstance(pattern, str) and pattern == "default":
            return True
        return self._is_equal(subject, pattern)

    def _visit_TryExpr(self, expr: TryExpr) -> Any:
        try:
            previous = self.environment
            try:
                self.environment = Environment(previous)
                for stmt in expr.body:
                    if stmt:
                        self._execute(stmt)
            finally:
                self.environment = previous
        except RuntimeError as e:
            if expr.catch_var:
                catch_env = Environment(self.environment)
                catch_env.define(expr.catch_var.lexeme, e.message)
                previous = self.environment
                try:
                    self.environment = catch_env
                    for stmt in expr.catch_body:
                        if stmt:
                            self._execute(stmt)
                finally:
                    self.environment = previous
            else:
                raise

    def _visit_ThrowExpr(self, expr: ThrowExpr) -> Any:
        value = self._evaluate(expr.value)
        raise RuntimeError(expr.token, self._stringify(value))

    def _visit_AwaitExpr(self, expr: AwaitExpr) -> Any:
        value = self._evaluate(expr.value)
        # In a full async implementation, this would yield control
        # For now, we just evaluate the value directly
        return value

    def _visit_YieldExpr(self, expr: YieldExpr) -> Any:
        value = None
        if expr.value:
            value = self._evaluate(expr.value)
        raise YieldException(value)

    def _check_number_operand(self, operator: Token, operand: Any):
        if not isinstance(operand, (int, float)):
            raise RuntimeError(operator, "Operand must be a number.")

    def _check_number_operands(self, operator: Token, left: Any, right: Any):
        if not isinstance(left, (int, float)) or not isinstance(right, (int, float)):
            raise RuntimeError(operator, "Operands must be numbers.")

    def _is_truthy(self, value: Any) -> bool:
        if value is None:
            return False
        if isinstance(value, bool):
            return value
        if isinstance(value, (int, float)):
            return value != 0
        if isinstance(value, str):
            return value != ""
        if isinstance(value, (list, dict)):
            return len(value) > 0
        return True

    def _is_equal(self, left: Any, right: Any) -> bool:
        if left is None and right is None:
            return True
        if left is None or right is None:
            return False
        return left == right

    def _stringify(self, value: Any) -> str:
        if value is None:
            return "nil"
        if isinstance(value, bool):
            return "true" if value else "false"
        if isinstance(value, float):
            text = str(value)
            if text.endswith(".0"):
                text = text[:-2]
            return text
        return str(value)


class BhabexFunction:
    def __init__(self, declaration, closure: Environment, is_initializer: bool):
        self.declaration = declaration
        self.closure = closure
        self.is_initializer = is_initializer
        self.is_async = False

    def call(self, interpreter: Interpreter, arguments: List[Any], environment: Optional[Environment] = None) -> Any:
        if environment is None:
            environment = Environment(self.closure)
        params = self.declaration.params if hasattr(self.declaration, 'params') else self.declaration.params
        for i, param in enumerate(params):
            if hasattr(param, 'name'):
                # Parameter object
                environment.define(param.name.lexeme, arguments[i])
            else:
                # Token (for backward compatibility)
                environment.define(param.lexeme, arguments[i])
        
        # Check if function contains yield expressions (is a generator)
        is_generator = self._contains_yield(self.declaration.body)
        
        if is_generator:
            return BhabexGenerator(self, interpreter, arguments, environment)
        
        try:
            if hasattr(self.declaration, 'body'):
                interpreter._execute_block(self.declaration.body, environment)
            else:
                interpreter._execute_block(self.declaration.body, environment)
        except ReturnException as return_value:
            if self.is_initializer:
                return environment.get_at(0, "self")
            return return_value.value
        except YieldException as yield_value:
            return yield_value.value
        
        if self.is_initializer:
            return environment.get_at(0, "self")
        return None
    
    def _contains_yield(self, body) -> bool:
        statements = body.statements if hasattr(body, 'statements') else body
        if isinstance(statements, list):
            for stmt in statements:
                if stmt and self._check_yield_in_stmt(stmt):
                    return True
        return False
    
    def _check_yield_in_stmt(self, stmt) -> bool:
        # Handle Block objects (which contain a list of statements)
        if hasattr(stmt, 'statements'):
            for s in stmt.statements:
                if s and self._check_yield_in_stmt(s):
                    return True
            return False
        
        if hasattr(stmt, 'expression') and stmt.expression:
            if self._check_yield_in_expr(stmt.expression):
                return True
        if hasattr(stmt, 'expressions'):
            for expr in stmt.expressions:
                if self._check_yield_in_expr(expr):
                    return True
        if hasattr(stmt, 'then_branch') and stmt.then_branch:
            if self._check_yield_in_stmt(stmt.then_branch):
                return True
        if hasattr(stmt, 'else_branch') and stmt.else_branch:
            if self._check_yield_in_stmt(stmt.else_branch):
                return True
        if hasattr(stmt, 'body') and stmt.body:
            if self._check_yield_in_stmt(stmt.body):
                return True
        if hasattr(stmt, 'catch_body') and stmt.catch_body:
            if self._check_yield_in_stmt(stmt.catch_body):
                return True
        return False
    
    def _check_yield_in_expr(self, expr) -> bool:
        if hasattr(expr, '__class__') and expr.__class__.__name__ == 'YieldExpr':
            return True
        if hasattr(expr, 'left') and expr.left:
            if self._check_yield_in_expr(expr.left):
                return True
        if hasattr(expr, 'right') and expr.right:
            if self._check_yield_in_expr(expr.right):
                return True
        if hasattr(expr, 'operand') and expr.operand:
            if self._check_yield_in_expr(expr.operand):
                return True
        if hasattr(expr, 'expression') and expr.expression:
            if self._check_yield_in_expr(expr.expression):
                return True
        if hasattr(expr, 'callee') and expr.callee:
            if self._check_yield_in_expr(expr.callee):
                return True
        if hasattr(expr, 'arguments'):
            for arg in expr.arguments:
                if self._check_yield_in_expr(arg):
                    return True
        if hasattr(expr, 'object') and expr.object:
            if self._check_yield_in_expr(expr.object):
                return True
        if hasattr(expr, 'value') and expr.value:
            if self._check_yield_in_expr(expr.value):
                return True
        if hasattr(expr, 'elements'):
            for elem in expr.elements:
                if self._check_yield_in_expr(elem):
                    return True
        if hasattr(expr, 'keys'):
            for key in expr.keys:
                if self._check_yield_in_expr(key):
                    return True
        if hasattr(expr, 'values'):
            for val in expr.values:
                if self._check_yield_in_expr(val):
                    return True
        if hasattr(expr, 'start') and expr.start:
            if self._check_yield_in_expr(expr.start):
                return True
        if hasattr(expr, 'end') and expr.end:
            if self._check_yield_in_expr(expr.end):
                return True
        if hasattr(expr, 'subject') and expr.subject:
            if self._check_yield_in_expr(expr.subject):
                return True
        if hasattr(expr, 'cases'):
            for case in expr.cases:
                if case.pattern and self._check_yield_in_expr(case.pattern):
                    return True
                if case.body:
                    if self._check_yield_in_stmt(case.body):
                        return True
        if hasattr(expr, 'body') and expr.body:
            if self._check_yield_in_stmt(expr.body):
                return True
        if hasattr(expr, 'catch_body') and expr.catch_body:
            if self._check_yield_in_stmt(expr.catch_body):
                return True
        if hasattr(expr, 'catch_var') and expr.catch_var:
            pass
        return False

    def bind(self, instance: 'BhabexInstance') -> 'BoundMethod':
        environment = Environment(self.closure)
        environment.define("self", instance)
        environment.define("this", instance)
        return BoundMethod(instance, self, environment)

    def __str__(self):
        prefix = "async " if self.is_async else ""
        if hasattr(self.declaration, 'name'):
            return f"<{prefix}fn {self.declaration.name.lexeme}>"
        return f"<{prefix}fn lambda>"


class BhabexGenerator:
    def __init__(self, function: BhabexFunction, interpreter: Interpreter, arguments: List[Any], environment: Environment):
        self.function = function
        self.interpreter = interpreter
        self.environment = environment
        self.arguments = arguments
        # Pre-collect all yielded values
        self.values = self._collect_values()
        self.index = 0
    
    def _collect_values(self) -> List[Any]:
        values = []
        # Create a temporary interpreter to execute the generator function
        temp_env = Environment(self.function.closure)
        params = self.function.declaration.params
        for i, param in enumerate(params):
            if i < len(self.arguments):
                temp_env.define(param.lexeme, self.arguments[i])
            else:
                temp_env.define(param.lexeme, None)
        
        # Execute the function body and collect yields
        def yield_handler(value):
            values.append(value)
        
        # Temporarily replace the yield handler
        old_visit_yield = self.interpreter._visit_YieldExpr
        self.interpreter._visit_YieldExpr = lambda expr: values.append(self.interpreter._evaluate(expr.value)) or None
        
        try:
            self.interpreter._execute_block(self.function.declaration.body, temp_env)
        except ReturnException:
            pass
        finally:
            self.interpreter._visit_YieldExpr = old_visit_yield
        
        return values
    
    def __iter__(self):
        return self
    
    def __next__(self):
        if self.index >= len(self.values):
            raise StopIteration
        value = self.values[self.index]
        self.index += 1
        return value


class BoundMethod:
    def __init__(self, instance: 'BhabexInstance', method: BhabexFunction, environment: Environment):
        self.instance = instance
        self.method = method
        self.environment = environment

    def call(self, interpreter: Interpreter, arguments: List[Any]) -> Any:
        return self.method.call(interpreter, arguments, self.environment)

    def __str__(self):
        return f"<bound method {self.method}>"


class BhabexClass:
    def __init__(self, name: str, methods: Dict[str, BhabexFunction]):
        self.name = name
        self.methods = methods

    def call(self, interpreter: Interpreter, arguments: List[Any]) -> 'BhabexInstance':
        instance = BhabexInstance(self)
        initializer = self.methods.get("init")
        if initializer:
            initializer.bind(instance).call(interpreter, arguments)
        return instance

    def find_method(self, name: str) -> Optional[BhabexFunction]:
        return self.methods.get(name)

    def __str__(self):
        return self.name


class BhabexInstance:
    def __init__(self, klass: BhabexClass):
        self.klass = klass
        self.fields = {}

    def get(self, name: Token) -> Any:
        if name.lexeme in self.fields:
            return self.fields[name.lexeme]
        method = self.klass.find_method(name.lexeme)
        if method:
            return method.bind(self)
        raise RuntimeError(name, f"Undefined property '{name.lexeme}'.")

    def set(self, name: Token, value: Any):
        self.fields[name.lexeme] = value

    def __str__(self):
        return f"{self.klass.name} instance"


class BhabexMacro:
    def __init__(self, declaration: MacroStmt, closure: Environment):
        self.declaration = declaration
        self.closure = closure

    def call(self, interpreter: Interpreter, arguments: List[Any]) -> Any:
        # Create a new environment for the macro expansion
        env = Environment(self.closure)
        params = self.declaration.params
        for i, param in enumerate(params):
            if i < len(arguments):
                env.define(param.name.lexeme, arguments[i])
            else:
                env.define(param.name.lexeme, None)
        
        # Execute the macro body in the new environment
        # The macro body is executed at "compile time" (when the macro is invoked)
        # and its result is spliced into the calling code
        previous_env = interpreter.environment
        try:
            interpreter.environment = env
            for stmt in self.declaration.body:
                if stmt:
                    interpreter._execute(stmt)
            # The last expression in the macro body becomes the expansion result
            # For simplicity, we return None (macros typically don't return values)
            return None
        finally:
            interpreter.environment = previous_env

    def __str__(self):
        if hasattr(self.declaration, 'name'):
            return f"<macro {self.declaration.name.lexeme}>"
        return "<macro lambda>"


# Alias for type hints
Stmt = object
Expr = object