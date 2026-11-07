"""
Bhabex Language Server Protocol (LSP) Implementation

Provides basic LSP support for .bx files.
"""

import json
import sys
import threading
from typing import Dict, List, Optional, Any
from .lexer import tokenize
from .parser import parse
from .ast_nodes import Program


class LSPServer:
    def __init__(self):
        self.documents: Dict[str, str] = {}
        self.request_id = 0
        self.lock = threading.Lock()

    def handle_request(self, request: Dict) -> Optional[Dict]:
        method = request.get("method")
        params = request.get("params", {})
        request_id = request.get("id")

        try:
            if method == "initialize":
                return self._handle_initialize(request_id, params)
            elif method == "initialized":
                self._handle_initialized()
                return None
            elif method == "textDocument/didOpen":
                self._handle_did_open(params)
                return None
            elif method == "textDocument/didChange":
                self._handle_did_change(params)
                return None
            elif method == "textDocument/didClose":
                self._handle_did_close(params)
                return None
            elif method == "textDocument/completion":
                return self._handle_completion(request_id, params)
            elif method == "textDocument/hover":
                return self._handle_hover(request_id, params)
            elif method == "textDocument/definition":
                return self._handle_definition(request_id, params)
            elif method == "textDocument/formatting":
                return self._handle_formatting(request_id, params)
            elif method == "textDocument/diagnostic":
                return self._handle_diagnostic(request_id, params)
            elif method == "shutdown":
                return {"id": request_id, "result": None}
            elif method == "exit":
                sys.exit(0)
            else:
                return {
                    "id": request_id,
                    "error": {"code": -32601, "message": f"Method not found: {method}"}
                }
        except Exception as e:
            return {
                "id": request_id,
                "error": {"code": -32603, "message": f"Internal error: {str(e)}"}
            }

    def _handle_initialize(self, request_id: int, params: Dict) -> Dict:
        return {
            "id": request_id,
            "result": {
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {"triggerCharacters": [".", "("]},
                    "hoverProvider": True,
                    "definitionProvider": True,
                    "documentFormattingProvider": True,
                    "diagnosticProvider": {"interFileDependencies": False, "workspaceDiagnostics": False}
                },
                "serverInfo": {"name": "bhabex-lsp", "version": "0.1.0"}
            }
        }

    def _handle_initialized(self):
        pass

    def _handle_did_open(self, params: Dict):
        doc = params.get("textDocument", {})
        uri = doc.get("uri")
        text = doc.get("text", "")
        if uri:
            self.documents[uri] = text

    def _handle_did_change(self, params: Dict):
        doc = params.get("textDocument", {})
        uri = doc.get("uri")
        changes = params.get("contentChanges", [])
        if uri and uri in self.documents:
            for change in changes:
                if "text" in change:
                    self.documents[uri] = change["text"]

    def _handle_did_close(self, params: Dict):
        doc = params.get("textDocument", {})
        uri = doc.get("uri")
        if uri and uri in self.documents:
            del self.documents[uri]

    def _handle_completion(self, request_id: int, params: Dict) -> Dict:
        items = []

        # Keywords
        keywords = [
            "fn", "let", "if", "else", "for", "in", "while", "return",
            "class", "new", "self", "import", "as", "match", "default",
            "try", "catch", "throw", "true", "false", "nil", "and", "or", "not",
            "break", "continue", "print", "async", "await", "yield",
            "import", "export", "macro", "type", "struct", "enum", "trait", "impl"
        ]
        for kw in keywords:
            items.append({"label": kw, "kind": 14})

        # Built-in functions
        builtins = [
            "print", "len", "str", "int", "float", "range", "type",
            "input", "list", "dict", "assert", "read_file", "write_file",
            "append_file", "file_exists", "is_file", "is_dir", "list_dir",
            "mkdir", "remove", "rename", "file_size", "json_parse",
            "json_stringify", "http_get", "http_post", "http_put",
            "http_delete", "http_patch", "http_head", "regex_match",
            "regex_findall", "regex_replace", "regex_split", "sin", "cos",
            "tan", "sqrt", "pow", "log", "exp", "floor", "ceil", "random",
            "randint", "now", "parse_datetime", "timestamp", "sleep",
            "env", "args", "exit", "cwd", "chdir", "platform",
        ]
        for fn in builtins:
            items.append({"label": fn, "kind": 3})

        return {"id": request_id, "result": {"isIncomplete": False, "items": items}}

    def _handle_hover(self, request_id: int, params: Dict) -> Dict:
        uri = params.get("textDocument", {}).get("uri")
        position = params.get("position", {})
        line = position.get("line", 0)
        character = position.get("character", 0)

        content = "No documentation available."
        if uri and uri in self.documents:
            text = self.documents[uri]
            lines = text.split("\n")
            if 0 <= line < len(lines):
                word = self._get_word_at_position(lines[line], character)
                if word:
                    content = self._get_hover_content(word)

        return {
            "id": request_id,
            "result": {"contents": {"kind": "markdown", "value": content}}
        }

    def _handle_definition(self, request_id: int, params: Dict) -> Dict:
        return {"id": request_id, "result": []}

    def _handle_formatting(self, request_id: int, params: Dict) -> Dict:
        from .formatter import format_source

        uri = params.get("textDocument", {}).get("uri")
        if uri and uri in self.documents:
            text = self.documents[uri]
            try:
                formatted = format_source(text)
                return {
                    "id": request_id,
                    "result": [{
                        "range": {
                            "start": {"line": 0, "character": 0},
                            "end": {"line": len(text.split("\n")), "character": 0}
                        },
                        "newText": formatted
                    }]
                }
            except Exception as e:
                return {"id": request_id, "error": {"code": -32603, "message": str(e)}}
        return {"id": request_id, "result": []}

    def _handle_diagnostic(self, request_id: int, params: Dict) -> Dict:
        uri = params.get("textDocument", {}).get("uri")
        diagnostics = []

        if uri and uri in self.documents:
            text = self.documents[uri]
            try:
                tokens = tokenize(text)
                program = parse(tokens)
                diagnostics = self._collect_diagnostics(program, text)
            except Exception as e:
                diagnostics.append({
                    "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}},
                    "severity": 1,
                    "message": str(e),
                    "source": "bhabex"
                })

        return {"id": request_id, "result": {"kind": "full", "items": diagnostics}}

    def _collect_diagnostics(self, program: Program, text: str) -> List[Dict]:
        diagnostics = []
        lines = text.split("\n")
        
        # Check for common issues
        for i, line in enumerate(lines):
            # Trailing whitespace
            if line.rstrip() != line:
                diagnostics.append({
                    "range": {
                        "start": {"line": i, "character": len(line.rstrip())},
                        "end": {"line": i, "character": len(line)}
                    },
                    "severity": 3,  # Information
                    "message": "Trailing whitespace",
                    "source": "bhabex"
                })
            
            # TODO comments
            if "TODO" in line:
                idx = line.index("TODO")
                diagnostics.append({
                    "range": {
                        "start": {"line": i, "character": idx},
                        "end": {"line": i, "character": idx + 4}
                    },
                    "severity": 2,  # Warning
                    "message": "TODO comment found",
                    "source": "bhabex"
                })
            
            # FIXME comments
            if "FIXME" in line:
                idx = line.index("FIXME")
                diagnostics.append({
                    "range": {
                        "start": {"line": i, "character": idx},
                        "end": {"line": i, "character": idx + 5}
                    },
                    "severity": 2,  # Warning
                    "message": "FIXME comment found",
                    "source": "bhabex"
                })
        
        return diagnostics

    def _get_word_at_position(self, line: str, character: int) -> Optional[str]:
        if character >= len(line):
            return None
        start = character
        while start > 0 and (line[start - 1].isalnum() or line[start - 1] == "_"):
            start -= 1
        end = character
        while end < len(line) and (line[end].isalnum() or line[end] == "_"):
            end += 1
        if start < end:
            return line[start:end]
        return None

    def _get_hover_content(self, word: str) -> str:
        docs = {
            "print": "**print(...values)**\n\nPrints values to stdout.",
            "len": "**len(value)**\n\nReturns length of string, list, or dict.",
            "str": "**str(value)**\n\nConverts value to string.",
            "int": "**int(value)**\n\nConverts value to integer.",
            "float": "**float(value)**\n\nConverts value to float.",
            "range": "**range(stop) / range(start, stop) / range(start, stop, step)**\n\nGenerates a list of integers.",
            "if": "**if condition { ... } else { ... }**\n\nConditional statement.",
            "for": "**for item in iterable { ... }**\n\nLoop over iterable.",
            "while": "**while condition { ... }**\n\nLoop while condition is true.",
            "fn": "**fn name(params) { ... }**\n\nFunction declaration.",
            "class": "**class Name { ... }**\n\nClass declaration.",
            "match": "**match expr { pattern => body ... }**\n\nPattern matching.",
            "try": "**try { ... } catch (e) { ... }**\n\nException handling.",
        }
        return docs.get(word, f"No documentation for `{word}`")


def run_stdio():
    server = LSPServer()

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        if line.startswith("Content-Length:"):
            continue
        if line == "":
            continue

        try:
            request = json.loads(line)
            response = server.handle_request(request)
            if response:
                print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            pass


if __name__ == "__main__":
    run_stdio()