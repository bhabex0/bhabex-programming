import sys
from .lexer import tokenize
from .parser import parse
from .interpreter import Interpreter
from .errors import BhabexError


class REPL:
    def __init__(self):
        self.interpreter = Interpreter()
        self.buffer = ""

    def run(self):
        print("Bhabex REPL v0.1.0")
        print("Type 'exit' or 'quit' to exit, 'help' for help.")
        print()
        
        while True:
            try:
                if self.buffer:
                    prompt = "... "
                else:
                    prompt = ">>> "
                
                line = input(prompt)
                
                if line.strip().lower() in ('exit', 'quit'):
                    print("Goodbye!")
                    break
                
                if line.strip().lower() == 'help':
                    self._print_help()
                    continue
                
                if line.strip().lower() == 'clear':
                    self.buffer = ""
                    print("Buffer cleared.")
                    continue
                
                self.buffer += line + "\n"
                
                if not self._is_complete(self.buffer):
                    continue
                
                self._execute_buffer()
                self.buffer = ""
                
            except KeyboardInterrupt:
                print("\nKeyboardInterrupt")
                self.buffer = ""
            except EOFError:
                print("\nGoodbye!")
                break
            except Exception as e:
                print(f"Error: {e}")
                self.buffer = ""

    def _is_complete(self, code: str) -> bool:
        """Check if the code is complete (balanced braces, brackets, parens)."""
        parens = 0
        braces = 0
        brackets = 0
        in_string = False
        string_char = None
        escape = False
        
        for char in code:
            if escape:
                escape = False
                continue
            
            if char == '\\':
                escape = True
                continue
            
            if in_string:
                if char == string_char:
                    in_string = False
                    string_char = None
                continue
            
            if char in ('"', "'"):
                in_string = True
                string_char = char
                continue
            
            if char == '(':
                parens += 1
            elif char == ')':
                parens -= 1
            elif char == '{':
                braces += 1
            elif char == '}':
                braces -= 1
            elif char == '[':
                brackets += 1
            elif char == ']':
                brackets -= 1
        
        return parens == 0 and braces == 0 and brackets == 0

    def _execute_buffer(self):
        try:
            tokens = tokenize(self.buffer)
            program = parse(tokens)
            self.interpreter.interpret(program)
        except BhabexError as e:
            print(f"Error: {e}")
        except Exception as e:
            print(f"Error: {e}")

    def _print_help(self):
        print("""
Bhabex REPL Commands:
  exit, quit    - Exit the REPL
  help          - Show this help
  clear         - Clear the input buffer

Bhabex Language Quick Reference:
  let x = 10        - Variable declaration
  fn add(a, b) {    - Function definition
      return a + b
  }
  if x > 0 {        - Conditional
      print(x)
  } else {
      print(-x)
  }
  for i in 0..10 {  - For loop
      print(i)
  }
  class Person {    - Class definition
      fn init(name) { self.name = name }
      fn greet() { print("Hi, " + self.name) }
  }
  let p = new Person("Alice")  - Object creation
  p.greet()
  
  try {             - Error handling
      throw "error"
  } catch (e) {
      print("Caught: " + e)
  }

Built-in functions: print, len, str, int, float, range, type, input, list, dict, assert
        """)


def main():
    repl = REPL()
    repl.run()


if __name__ == "__main__":
    main()