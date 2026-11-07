from typing import Dict, Any, Optional
from .tokens import Token
from .errors import RuntimeError


class Environment:
    def __init__(self, enclosing: Optional['Environment'] = None):
        self.values: Dict[str, Any] = {}
        self.enclosing = enclosing

    def define(self, name: str, value: Any):
        self.values[name] = value

    def get(self, name: Token) -> Any:
        if name.lexeme in self.values:
            return self.values[name.lexeme]
        if self.enclosing:
            return self.enclosing.get(name)
        raise RuntimeError(name, f"Undefined variable '{name.lexeme}'.")

    def assign(self, name: Token, value: Any):
        if name.lexeme in self.values:
            self.values[name.lexeme] = value
            return
        if self.enclosing:
            self.enclosing.assign(name, value)
            return
        raise RuntimeError(name, f"Undefined variable '{name.lexeme}'.")

    def get_at(self, distance: int, name: str) -> Any:
        env = self
        for _ in range(distance):
            env = env.enclosing
        return env.values[name]

    def assign_at(self, distance: int, name: Token, value: Any):
        env = self
        for _ in range(distance):
            env = env.enclosing
        env.values[name.lexeme] = value

    def ancestor(self, distance: int) -> 'Environment':
        env = self
        for _ in range(distance):
            env = env.enclosing
        return env