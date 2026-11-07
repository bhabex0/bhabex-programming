from lexer import tokenize
from parser import parse
from interpreter import Interpreter

source = """
fn main() {
    print("Hello from Bhabex!")
    let x = 42
    let y = 10
    print(x + y)
}
"""

tokens = tokenize(source)
program = parse(tokens)
interpreter = Interpreter()
interpreter.interpret(program)