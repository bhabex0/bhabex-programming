"""
Bhabex Code Formatter

Formats .bx source code with consistent style.
"""

from .lexer import tokenize
from .parser import parse
from .ast_nodes import Program, Stmt, Expr


class Formatter:
    def __init__(self, indent_size: int = 4):
        self.indent_size = indent_size
        self.indent_level = 0
        self.output = []

    def format(self, program) -> str:
        self.indent_level = 0
        self.output = []
        for stmt in program.statements:
            if stmt:
                self._visit_stmt(stmt)
        return "\n".join(self.output)

    def _visit_stmt(self, stmt):
        from .ast_nodes import (
            ExpressionStmt, PrintStmt, VarDecl, Assign, Block, IfStmt,
            WhileStmt, ForStmt, FunctionStmt, ReturnStmt, ClassStmt,
            ImportStmt, BreakStmt, ContinueStmt,
        )
        
        if isinstance(stmt, ExpressionStmt):
            self._visit_expression_stmt(stmt)
        elif isinstance(stmt, PrintStmt):
            self._visit_print_stmt(stmt)
        elif isinstance(stmt, VarDecl):
            self._visit_var_decl(stmt)
        elif isinstance(stmt, Assign):
            self._visit_assign(stmt)
        elif isinstance(stmt, Block):
            self._visit_block(stmt)
        elif isinstance(stmt, IfStmt):
            self._visit_if(stmt)
        elif isinstance(stmt, WhileStmt):
            self._visit_while(stmt)
        elif isinstance(stmt, ForStmt):
            self._visit_for(stmt)
        elif isinstance(stmt, FunctionStmt):
            self._visit_function(stmt)
        elif isinstance(stmt, ReturnStmt):
            self._visit_return(stmt)
        elif isinstance(stmt, ClassStmt):
            self._visit_class(stmt)
        elif isinstance(stmt, ImportStmt):
            self._visit_import(stmt)
        elif isinstance(stmt, BreakStmt):
            self.output.append(self._indent() + "break;")
        elif isinstance(stmt, ContinueStmt):
            self.output.append(self._indent() + "continue;")
        else:
            pass

    def _visit_expression_stmt(self, stmt):
        self.output.append(self._indent() + self._expr_to_str(stmt.expression) + ";")

    def _visit_print_stmt(self, stmt):
        parts = [self._expr_to_str(expr) for expr in stmt.expressions]
        self.output.append(self._indent() + "print(" + ", ".join(parts) + ");")

    def _visit_var_decl(self, stmt):
        line = self._indent() + "let " + stmt.name.lexeme
        if stmt.initializer:
            line += " = " + self._expr_to_str(stmt.initializer)
        line += ";"
        self.output.append(line)

    def _visit_assign(self, stmt):
        self.output.append(self._expr_to_str(stmt.target))
        if stmt.op:
            self.output.append(" " + stmt.op + " ")
        else:
            self.output.append(" = ")
        self.output.append(self._expr_to_str(stmt.value))
        self.output.append(";")

    def _visit_block(self, block):
        self.output.append(self._indent() + "{")
        self.indent_level += 1
        statements = block.statements if hasattr(block, 'statements') else block
        for stmt in statements:
            if stmt:
                self._visit_stmt(stmt)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")

    def _visit_if(self, stmt):
        self.output.append(self._indent() + "if " + self._expr_to_str(stmt.condition) + " {")
        self.indent_level += 1
        if stmt.then_branch:
            # then_branch is already a Block, visit its statements directly
            from .ast_nodes import Block
            if isinstance(stmt.then_branch, Block):
                for s in stmt.then_branch.statements:
                    if s:
                        self._visit_stmt(s)
            else:
                self._visit_stmt(stmt.then_branch)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")
        if stmt.else_branch:
            self.output.append(self._indent() + "else {")
            self.indent_level += 1
            from .ast_nodes import Block
            if isinstance(stmt.else_branch, Block):
                for s in stmt.else_branch.statements:
                    if s:
                        self._visit_stmt(s)
            else:
                self._visit_stmt(stmt.else_branch)
            self.indent_level -= 1
            self.output.append(self._indent() + "}")

    def _visit_while(self, stmt):
        self.output.append(self._indent() + "while " + self._expr_to_str(stmt.condition) + " {")
        self.indent_level += 1
        if stmt.body:
            from .ast_nodes import Block
            if isinstance(stmt.body, Block):
                for s in stmt.body.statements:
                    if s:
                        self._visit_stmt(s)
            else:
                self._visit_stmt(stmt.body)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")

    def _visit_for(self, stmt):
        self.output.append(self._indent() + f"for {stmt.var.lexeme} in " + self._expr_to_str(stmt.iterable) + " {")
        self.indent_level += 1
        if stmt.body:
            from .ast_nodes import Block
            if isinstance(stmt.body, Block):
                for s in stmt.body.statements:
                    if s:
                        self._visit_stmt(s)
            else:
                self._visit_stmt(stmt.body)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")

    def _visit_function(self, stmt):
        params = ", ".join(p.name.lexeme for p in stmt.params)
        self.output.append(self._indent() + f"fn {stmt.name.lexeme}({params}) {{")
        self.indent_level += 1
        for s in stmt.body:
            if s:
                self._visit_stmt(s)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")

    def _visit_return(self, stmt):
        self.output.append(self._indent() + "return")
        if stmt.value:
            self.output.append(" ")
            self.output.append(self._expr_to_str(stmt.value))
        self.output.append(";")

    def _visit_class(self, stmt):
        self.output.append(self._indent() + "class " + stmt.name.lexeme + " {\n")
        self.indent_level += 1
        for method in stmt.methods:
            self._visit_function(method)
        self.indent_level -= 1
        self.output.append(self._indent() + "}")

    def _visit_import(self, stmt):
        self.output.append(self._indent() + "import " + stmt.path)
        if stmt.alias:
            self.output.append(" as " + stmt.alias.lexeme)
        self.output.append(";")

    def _expr_to_str(self, expr):
        from .ast_nodes import (
            Literal, Identifier, Binary, Unary, Grouping, Call, Get, Set,
            This, Super, IndexGet, IndexSet, ListLiteral, DictLiteral, Range,
            Lambda, MatchExpr, TryExpr, ThrowExpr, Assign
        )
        if isinstance(expr, Literal):
            if expr.value is None:
                return "nil"
            elif isinstance(expr.value, bool):
                return "true" if expr.value else "false"
            elif isinstance(expr.value, str):
                return '"' + expr.value + '"'
            elif isinstance(expr.value, (int, float)):
                return str(expr.value)
            else:
                return str(expr.value)
        elif isinstance(expr, Identifier):
            return expr.name
        elif isinstance(expr, Binary):
            return self._expr_to_str(expr.left) + " " + expr.operator.lexeme + " " + self._expr_to_str(expr.right)
        elif isinstance(expr, Unary):
            return expr.operator.lexeme + self._expr_to_str(expr.right)
        elif isinstance(expr, Grouping):
            return "(" + self._expr_to_str(expr.expression) + ")"
        elif isinstance(expr, Call):
            result = self._expr_to_str(expr.callee) + "("
            for i, arg in enumerate(expr.arguments):
                if i > 0:
                    result += ", "
                result += self._expr_to_str(arg)
            return result + ")"
        elif isinstance(expr, Get):
            return self._expr_to_str(expr.object) + "." + expr.name.lexeme
        elif isinstance(expr, IndexGet):
            return self._expr_to_str(expr.object) + "[" + self._expr_to_str(expr.index) + "]"
        elif isinstance(expr, ListLiteral):
            result = "["
            for i, elem in enumerate(expr.elements):
                if i > 0:
                    result += ", "
                result += self._expr_to_str(elem)
            return result + "]"
        elif isinstance(expr, DictLiteral):
            result = "{"
            for i, (k, v) in enumerate(zip(expr.keys, expr.values)):
                if i > 0:
                    result += ", "
                result += self._expr_to_str(k) + ": " + self._expr_to_str(v)
            return result + "}"
        elif isinstance(expr, Range):
            result = self._expr_to_str(expr.start) + ".." + self._expr_to_str(expr.end)
            if expr.inclusive:
                result += "="
            return result
        elif isinstance(expr, Assign):
            return expr.name.lexeme + " = " + self._expr_to_str(expr.value)
        elif isinstance(expr, Lambda):
            result = "fn("
            for i, param in enumerate(expr.params):
                if i > 0:
                    result += ", "
                result += param.lexeme
            result += ") "
            # For simplicity, just show the first statement
            if expr.body:
                result += " { ... }"
            else:
                result += "{ }"
            return result
        else:
            return "<expr>"

    def _indent(self):
        return " " * (self.indent_level * self.indent_size)

    def append(self, text):
        self.output.append(text)


def format_file(file_path: str, indent_size: int = 4) -> str:
    """Format a .bx file and return formatted source."""
    with open(file_path, 'r', encoding='utf-8') as f:
        source = f.read()
    return format_source(source, indent_size)


def format_source(source: str, indent_size: int = 4) -> str:
    """Format source code and return formatted string."""
    tokens = tokenize(source)
    from bhabex.parser import parse
    program = parse(tokens)
    return _format_program(program, indent_size)


def _format_program(program, indent_size: int) -> str:
    formatter = Formatter(indent_size=indent_size)
    return formatter.format(program)


def main():
    import sys
    if len(sys.argv) < 2:
        print("Usage: python -m bhabex.fmt <file.bx> [--indent N]")
        sys.exit(1)

    file_path = sys.argv[1]
    indent_size = 4
    if "--indent" in sys.argv:
        idx = sys.argv.index("--indent")
        if idx + 1 < len(sys.argv):
            indent_size = int(sys.argv[idx + 1])

    formatted = format_file(sys.argv[1], indent_size)
    print(formatted)


if __name__ == "__main__":
    main()