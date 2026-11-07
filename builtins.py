from typing import Any, List, Callable
import json
import os
import sys
import re
import math
import random
import time
import datetime
import urllib.request
import urllib.error
from .tokens import Token, TokenType
from .environment import Environment
from .errors import RuntimeError


class BuiltinFunction:
    def __init__(self, name: str, arity: int, func: Callable):
        self.name = name
        self.arity = arity
        self.func = func

    def call(self, interpreter, arguments: List[Any]) -> Any:
        if len(arguments) != self.arity and self.arity >= 0:
            raise RuntimeError(Token(TokenType.IDENTIFIER, self.name, None, 0, 0),
                             f"Expected {self.arity} arguments but got {len(arguments)}.")
        return self.func(arguments)

    def __str__(self):
        return f"<builtin fn {self.name}>"


class BuiltinClass:
    def __init__(self, name: str, methods: dict):
        self.name = name
        self.methods = methods

    def instantiate(self, arguments: List[Any]) -> 'BuiltinInstance':
        return BuiltinInstance(self, arguments)

    def __str__(self):
        return f"<builtin class {self.name}>"


class BuiltinInstance:
    def __init__(self, klass: BuiltinClass, init_args: List[Any]):
        self.klass = klass
        self.fields = {}
        if "init" in klass.methods:
            klass.methods["init"].bind(self).call(None, init_args)

    def get(self, name: str) -> Any:
        if name in self.fields:
            return self.fields[name]
        if name in self.klass.methods:
            return self.klass.methods[name].bind(self)
        return None

    def set(self, name: str, value: Any):
        self.fields[name] = value

    def __str__(self):
        return f"<{self.klass.name} instance>"


class BoundMethod:
    def __init__(self, instance: BuiltinInstance, method: BuiltinFunction):
        self.instance = instance
        self.method = method

    def call(self, interpreter, arguments: List[Any]) -> Any:
        return self.method.call(interpreter, [self.instance] + arguments)

    def bind(self, instance: BuiltinInstance) -> 'BoundMethod':
        return BoundMethod(instance, self.method)

    def __str__(self):
        return f"<bound method {self.method.name}>"


def make_print_fn():
    def print_fn(args: List[Any]) -> None:
        for i, arg in enumerate(args):
            if i > 0:
                print(" ", end="")
            print(stringify(arg), end="")
        print()
        return None
    return BuiltinFunction("print", -1, print_fn)


def make_len_fn():
    def len_fn(args: List[Any]) -> int:
        val = args[0]
        if isinstance(val, (str, list, dict)):
            return len(val)
        raise RuntimeError(Token(TokenType.IDENTIFIER, "len", None, 0, 0),
                          f"Cannot get length of {type(val).__name__}")
    return BuiltinFunction("len", 1, len_fn)


def make_str_fn():
    def str_fn(args: List[Any]) -> str:
        return stringify(args[0])
    return BuiltinFunction("str", 1, str_fn)


def make_int_fn():
    def int_fn(args: List[Any]) -> int:
        val = args[0]
        if isinstance(val, (int, float)):
            return int(val)
        if isinstance(val, str):
            return int(val)
        raise RuntimeError(Token(TokenType.IDENTIFIER, "int", None, 0, 0),
                          f"Cannot convert {type(val).__name__} to int")
    return BuiltinFunction("int", 1, int_fn)


def make_float_fn():
    def float_fn(args: List[Any]) -> float:
        val = args[0]
        if isinstance(val, (int, float)):
            return float(val)
        if isinstance(val, str):
            return float(val)
        raise RuntimeError(Token(TokenType.IDENTIFIER, "float", None, 0, 0),
                          f"Cannot convert {type(val).__name__} to float")
    return BuiltinFunction("float", 1, float_fn)


def make_range_fn():
    def range_fn(args: List[Any]) -> List[int]:
        if len(args) == 1:
            return list(range(int(args[0])))
        elif len(args) == 2:
            return list(range(int(args[0]), int(args[1])))
        elif len(args) == 3:
            return list(range(int(args[0]), int(args[1]), int(args[2])))
        raise RuntimeError(Token(TokenType.IDENTIFIER, "range", None, 0, 0),
                          "range() takes 1-3 arguments")
    return BuiltinFunction("range", -1, range_fn)


def make_type_fn():
    def type_fn(args: List[Any]) -> str:
        return type(args[0]).__name__
    return BuiltinFunction("type", 1, type_fn)


def make_input_fn():
    def input_fn(args: List[Any]) -> str:
        prompt = stringify(args[0]) if args else ""
        return input(prompt)
    return BuiltinFunction("input", -1, input_fn)


def make_list_fn():
    def list_fn(args: List[Any]) -> List[Any]:
        return list(args)
    return BuiltinFunction("list", -1, list_fn)


def make_dict_fn():
    def dict_fn(args: List[Any]) -> dict:
        if len(args) % 2 != 0:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "dict", None, 0, 0),
                              "dict() requires even number of arguments")
        result = {}
        for i in range(0, len(args), 2):
            result[args[i]] = args[i + 1]
        return result
    return BuiltinFunction("dict", -1, dict_fn)


def make_assert_fn():
    def assert_fn(args: List[Any]) -> None:
        if not args[0]:
            msg = stringify(args[1]) if len(args) > 1 else "Assertion failed"
            raise RuntimeError(Token(TokenType.IDENTIFIER, "assert", None, 0, 0), msg)
        return None
    return BuiltinFunction("assert", -1, assert_fn)


# ===== File I/O =====
def make_read_file_fn():
    def read_file_fn(args: List[Any]) -> str:
        path = stringify(args[0])
        encoding = stringify(args[1]) if len(args) > 1 else "utf-8"
        try:
            with open(path, "r", encoding=encoding) as f:
                return f.read()
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "read_file", None, 0, 0), str(e))
    return BuiltinFunction("read_file", -1, read_file_fn)


def make_write_file_fn():
    def write_file_fn(args: List[Any]) -> None:
        path = stringify(args[0])
        content = stringify(args[1]) if len(args) > 1 else ""
        encoding = stringify(args[2]) if len(args) > 2 else "utf-8"
        try:
            with open(path, "w", encoding=encoding) as f:
                f.write(content)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "write_file", None, 0, 0), str(e))
        return None
    return BuiltinFunction("write_file", -1, write_file_fn)


def make_append_file_fn():
    def append_file_fn(args: List[Any]) -> None:
        path = stringify(args[0])
        content = stringify(args[1]) if len(args) > 1 else ""
        encoding = stringify(args[2]) if len(args) > 2 else "utf-8"
        try:
            with open(path, "a", encoding=encoding) as f:
                f.write(content)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "append_file", None, 0, 0), str(e))
        return None
    return BuiltinFunction("append_file", -1, append_file_fn)


def make_file_exists_fn():
    def file_exists_fn(args: List[Any]) -> bool:
        path = stringify(args[0])
        return os.path.exists(path)
    return BuiltinFunction("file_exists", 1, file_exists_fn)


def make_is_file_fn():
    def is_file_fn(args: List[Any]) -> bool:
        path = stringify(args[0])
        return os.path.isfile(path)
    return BuiltinFunction("is_file", 1, is_file_fn)


def make_is_dir_fn():
    def is_dir_fn(args: List[Any]) -> bool:
        path = stringify(args[0])
        return os.path.isdir(path)
    return BuiltinFunction("is_dir", 1, is_dir_fn)


def make_list_dir_fn():
    def list_dir_fn(args: List[Any]) -> List[str]:
        path = stringify(args[0]) if args else "."
        try:
            return os.listdir(path)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "list_dir", None, 0, 0), str(e))
    return BuiltinFunction("list_dir", -1, list_dir_fn)


def make_mkdir_fn():
    def mkdir_fn(args: List[Any]) -> None:
        path = stringify(args[0])
        try:
            os.makedirs(path, exist_ok=True)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "mkdir", None, 0, 0), str(e))
        return None
    return BuiltinFunction("mkdir", 1, mkdir_fn)


def make_remove_fn():
    def remove_fn(args: List[Any]) -> None:
        path = stringify(args[0])
        try:
            if os.path.isdir(path):
                os.rmdir(path)
            else:
                os.remove(path)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "remove", None, 0, 0), str(e))
        return None
    return BuiltinFunction("remove", 1, remove_fn)


def make_rename_fn():
    def rename_fn(args: List[Any]) -> None:
        src = stringify(args[0])
        dst = stringify(args[1])
        try:
            os.rename(src, dst)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "rename", None, 0, 0), str(e))
        return None
    return BuiltinFunction("rename", 2, rename_fn)


def make_file_size_fn():
    def file_size_fn(args: List[Any]) -> int:
        path = stringify(args[0])
        try:
            return os.path.getsize(path)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "file_size", None, 0, 0), str(e))
    return BuiltinFunction("file_size", 1, file_size_fn)


# ===== JSON =====
def make_json_parse_fn():
    def json_parse_fn(args: List[Any]) -> Any:
        text = stringify(args[0])
        try:
            return json.loads(text)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "json_parse", None, 0, 0), str(e))
    return BuiltinFunction("json_parse", 1, json_parse_fn)


def make_json_stringify_fn():
    def json_stringify_fn(args: List[Any]) -> str:
        value = args[0]
        indent = int(args[1]) if len(args) > 1 and args[1] is not None else None
        try:
            return json.dumps(value, indent=indent)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "json_stringify", None, 0, 0), str(e))
    return BuiltinFunction("json_stringify", -1, json_stringify_fn)


# ===== HTTP =====
def make_http_get_fn():
    def http_get_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        headers = args[1] if len(args) > 1 and isinstance(args[1], dict) else {}
        timeout = float(args[2]) if len(args) > 2 else 30.0
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": response.read().decode("utf-8")
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_get", None, 0, 0), str(e))
    return BuiltinFunction("http_get", -1, http_get_fn)


def make_http_post_fn():
    def http_post_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        data = args[1] if len(args) > 1 else None
        headers = args[2] if len(args) > 2 and isinstance(args[2], dict) else {}
        timeout = float(args[3]) if len(args) > 3 else 30.0
        try:
            if data is not None:
                if isinstance(data, (dict, list)):
                    data = json.dumps(data).encode("utf-8")
                    if "Content-Type" not in headers:
                        headers["Content-Type"] = "application/json"
                else:
                    data = stringify(data).encode("utf-8")
            req = urllib.request.Request(url, data=data, headers=headers, method="POST")
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": response.read().decode("utf-8")
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_post", None, 0, 0), str(e))
    return BuiltinFunction("http_post", -1, http_post_fn)


# ===== Regex =====
def make_regex_match_fn():
    def regex_match_fn(args: List[Any]) -> List[str] | None:
        pattern = stringify(args[0])
        text = stringify(args[1])
        try:
            match = re.search(pattern, text)
            if match:
                return list(match.groups()) if match.groups() else [match.group(0)]
            return None
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "regex_match", None, 0, 0), str(e))
    return BuiltinFunction("regex_match", 2, regex_match_fn)


def make_regex_findall_fn():
    def regex_findall_fn(args: List[Any]) -> List[str]:
        pattern = stringify(args[0])
        text = stringify(args[1])
        try:
            return re.findall(pattern, text)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "regex_findall", None, 0, 0), str(e))
    return BuiltinFunction("regex_findall", 2, regex_findall_fn)


def make_regex_replace_fn():
    def regex_replace_fn(args: List[Any]) -> str:
        pattern = stringify(args[0])
        replacement = stringify(args[1])
        text = stringify(args[2])
        count = int(args[3]) if len(args) > 3 else 0
        try:
            return re.sub(pattern, replacement, text, count=count)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "regex_replace", None, 0, 0), str(e))
    return BuiltinFunction("regex_replace", -1, regex_replace_fn)


def make_regex_split_fn():
    def regex_split_fn(args: List[Any]) -> List[str]:
        pattern = stringify(args[0])
        text = stringify(args[1])
        maxsplit = int(args[2]) if len(args) > 2 else 0
        try:
            return re.split(pattern, text, maxsplit=maxsplit)
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "regex_split", None, 0, 0), str(e))
    return BuiltinFunction("regex_split", -1, regex_split_fn)


# ===== Math =====
def make_math_sin_fn():
    def math_sin_fn(args: List[Any]) -> float:
        return math.sin(float(args[0]))
    return BuiltinFunction("sin", 1, math_sin_fn)


def make_math_cos_fn():
    def math_cos_fn(args: List[Any]) -> float:
        return math.cos(float(args[0]))
    return BuiltinFunction("cos", 1, math_cos_fn)


def make_math_tan_fn():
    def math_tan_fn(args: List[Any]) -> float:
        return math.tan(float(args[0]))
    return BuiltinFunction("tan", 1, math_tan_fn)


def make_math_sqrt_fn():
    def math_sqrt_fn(args: List[Any]) -> float:
        return math.sqrt(float(args[0]))
    return BuiltinFunction("sqrt", 1, math_sqrt_fn)


def make_math_pow_fn():
    def math_pow_fn(args: List[Any]) -> float:
        return math.pow(float(args[0]), float(args[1]))
    return BuiltinFunction("pow", 2, math_pow_fn)


def make_math_log_fn():
    def math_log_fn(args: List[Any]) -> float:
        x = float(args[0])
        base = float(args[1]) if len(args) > 1 else math.e
        return math.log(x, base)
    return BuiltinFunction("log", -1, math_log_fn)


def make_math_exp_fn():
    def math_exp_fn(args: List[Any]) -> float:
        return math.exp(float(args[0]))
    return BuiltinFunction("exp", 1, math_exp_fn)


def make_math_floor_fn():
    def math_floor_fn(args: List[Any]) -> int:
        return math.floor(float(args[0]))
    return BuiltinFunction("floor", 1, math_floor_fn)


def make_math_ceil_fn():
    def math_ceil_fn(args: List[Any]) -> int:
        return math.ceil(float(args[0]))
    return BuiltinFunction("ceil", 1, math_ceil_fn)


def make_math_random_fn():
    def math_random_fn(args: List[Any]) -> float:
        if len(args) == 0:
            return random.random()
        elif len(args) == 1:
            return random.uniform(0, float(args[0]))
        else:
            return random.uniform(float(args[0]), float(args[1]))
    return BuiltinFunction("random", -1, math_random_fn)


def make_math_randint_fn():
    def math_randint_fn(args: List[Any]) -> int:
        a = int(args[0])
        b = int(args[1])
        return random.randint(a, b)
    return BuiltinFunction("randint", 2, math_randint_fn)


def make_math_pi_fn():
    return BuiltinFunction("pi", 0, lambda _: math.pi)


def make_math_e_fn():
    return BuiltinFunction("e", 0, lambda _: math.e)


# ===== Datetime =====
def make_datetime_now_fn():
    def datetime_now_fn(args: List[Any]) -> str:
        fmt = stringify(args[0]) if args else "%Y-%m-%d %H:%M:%S"
        return datetime.datetime.now().strftime(fmt)
    return BuiltinFunction("now", -1, datetime_now_fn)


def make_datetime_parse_fn():
    def datetime_parse_fn(args: List[Any]) -> str:
        text = stringify(args[0])
        fmt = stringify(args[1]) if len(args) > 1 else "%Y-%m-%d %H:%M:%S"
        try:
            dt = datetime.datetime.strptime(text, fmt)
            return dt.strftime("%Y-%m-%d %H:%M:%S")
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "parse_datetime", None, 0, 0), str(e))
    return BuiltinFunction("parse_datetime", -1, datetime_parse_fn)


def make_datetime_timestamp_fn():
    def datetime_timestamp_fn(args: List[Any]) -> float:
        return time.time()
    return BuiltinFunction("timestamp", 0, datetime_timestamp_fn)


def make_datetime_sleep_fn():
    def datetime_sleep_fn(args: List[Any]) -> None:
        seconds = float(args[0])
        time.sleep(seconds)
        return None
    fn = BuiltinFunction("sleep", 1, datetime_sleep_fn)
    fn.is_async = True
    return fn


def make_datetime_async_sleep_fn():
    import asyncio
    def datetime_async_sleep_fn(args: List[Any]) -> None:
        seconds = float(args[0])
        asyncio.run(asyncio.sleep(seconds))
        return None
    fn = BuiltinFunction("async_sleep", 1, datetime_async_sleep_fn)
    fn.is_async = True
    return fn


# ===== OS utilities =====
def make_os_env_fn():
    def os_env_fn(args: List[Any]) -> str | None:
        key = stringify(args[0])
        default = stringify(args[1]) if len(args) > 1 else None
        return os.environ.get(key, default)
    return BuiltinFunction("env", -1, os_env_fn)


def make_os_args_fn():
    def os_args_fn(args: List[Any]) -> List[str]:
        return sys.argv[1:]
    return BuiltinFunction("args", 0, os_args_fn)


def make_os_exit_fn():
    def os_exit_fn(args: List[Any]) -> None:
        code = int(args[0]) if args else 0
        sys.exit(code)
    return BuiltinFunction("exit", -1, os_exit_fn)


def make_os_cwd_fn():
    def os_cwd_fn(args: List[Any]) -> str:
        return os.getcwd()
    return BuiltinFunction("cwd", 0, os_cwd_fn)


def make_os_chdir_fn():
    def os_chdir_fn(args: List[Any]) -> None:
        path = stringify(args[0])
        os.chdir(path)
        return None
    return BuiltinFunction("chdir", 1, os_chdir_fn)


def make_os_platform_fn():
    def os_platform_fn(args: List[Any]) -> str:
        return sys.platform
    return BuiltinFunction("platform", 0, os_platform_fn)


# ===== SQLite =====
def make_sqlite_open_fn():
    def sqlite_open_fn(args: List[Any]) -> int:
        import sqlite3
        path = stringify(args[0])
        conn = sqlite3.connect(path)
        # Store connection in a global registry (simplified)
        conn_id = id(conn)
        _sqlite_connections[conn_id] = conn
        return conn_id
    return BuiltinFunction("sqlite_open", 1, sqlite_open_fn)


def make_sqlite_exec_fn():
    def sqlite_exec_fn(args: List[Any]) -> int:
        import sqlite3
        conn_id = int(args[0])
        sql = stringify(args[1])
        conn = _sqlite_connections.get(conn_id)
        if not conn:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "sqlite_exec", None, 0, 0), "Invalid connection")
        cursor = conn.cursor()
        cursor.execute(sql)
        conn.commit()
        return cursor.rowcount
    return BuiltinFunction("sqlite_exec", 2, sqlite_exec_fn)


def make_sqlite_query_fn():
    def sqlite_query_fn(args: List[Any]) -> list:
        import sqlite3
        conn_id = int(args[0])
        sql = stringify(args[1])
        conn = _sqlite_connections.get(conn_id)
        if not conn:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "sqlite_query", None, 0, 0), "Invalid connection")
        cursor = conn.cursor()
        cursor.execute(sql)
        columns = [desc[0] for desc in cursor.description] if cursor.description else []
        rows = cursor.fetchall()
        result = []
        for row in rows:
            row_dict = {}
            for i, col in enumerate(columns):
                row_dict[col] = row[i]
            result.append(row_dict)
        return result
    return BuiltinFunction("sqlite_query", 2, sqlite_query_fn)


def make_sqlite_close_fn():
    def sqlite_close_fn(args: List[Any]) -> None:
        import sqlite3
        conn_id = int(args[0])
        conn = _sqlite_connections.pop(conn_id, None)
        if conn:
            conn.close()
        return None
    return BuiltinFunction("sqlite_close", 1, sqlite_close_fn)


# SQLite connection registry
_sqlite_connections = {}


# ===== HTTP improvements =====
def make_http_put_fn():
    def http_put_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        data = args[1] if len(args) > 1 else None
        headers = args[2] if len(args) > 2 and isinstance(args[2], dict) else {}
        timeout = float(args[3]) if len(args) > 3 else 30.0
        try:
            if data is not None:
                if isinstance(data, (dict, list)):
                    data = json.dumps(data).encode("utf-8")
                    if "Content-Type" not in headers:
                        headers["Content-Type"] = "application/json"
                else:
                    data = stringify(data).encode("utf-8")
            req = urllib.request.Request(url, data=data, headers=headers, method="PUT")
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": response.read().decode("utf-8")
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_put", None, 0, 0), str(e))
    return BuiltinFunction("http_put", -1, http_put_fn)


def make_http_delete_fn():
    def http_delete_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        headers = args[1] if len(args) > 1 and isinstance(args[1], dict) else {}
        timeout = float(args[2]) if len(args) > 2 else 30.0
        try:
            req = urllib.request.Request(url, headers=headers, method="DELETE")
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": response.read().decode("utf-8")
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_delete", None, 0, 0), str(e))
    return BuiltinFunction("http_delete", -1, http_delete_fn)


def make_http_patch_fn():
    def http_patch_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        data = args[1] if len(args) > 1 else None
        headers = args[2] if len(args) > 2 and isinstance(args[2], dict) else {}
        timeout = float(args[3]) if len(args) > 3 else 30.0
        try:
            if data is not None:
                if isinstance(data, (dict, list)):
                    data = json.dumps(data).encode("utf-8")
                    if "Content-Type" not in headers:
                        headers["Content-Type"] = "application/json"
                else:
                    data = stringify(data).encode("utf-8")
            req = urllib.request.Request(url, data=data, headers=headers, method="PATCH")
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": response.read().decode("utf-8")
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_patch", None, 0, 0), str(e))
    return BuiltinFunction("http_patch", -1, http_patch_fn)


def make_http_head_fn():
    def http_head_fn(args: List[Any]) -> dict:
        url = stringify(args[0])
        headers = args[1] if len(args) > 1 and isinstance(args[1], dict) else {}
        timeout = float(args[2]) if len(args) > 2 else 30.0
        try:
            req = urllib.request.Request(url, headers=headers, method="HEAD")
            with urllib.request.urlopen(req, timeout=timeout) as response:
                return {
                    "status": response.status,
                    "headers": dict(response.headers),
                    "body": ""
                }
        except urllib.error.HTTPError as e:
            return {
                "status": e.code,
                "headers": dict(e.headers),
                "body": e.read().decode("utf-8") if e.fp else ""
            }
        except Exception as e:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "http_head", None, 0, 0), str(e))
    return BuiltinFunction("http_head", -1, http_head_fn)


# ===== JSON Schema validation =====
def make_json_schema_validate_fn():
    def json_schema_validate_fn(args: List[Any]) -> bool:
        import json
        try:
            import jsonschema
        except ImportError:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "json_schema_validate", None, 0, 0), "jsonschema library not installed")
        schema = args[0]
        instance = args[1]
        try:
            jsonschema.validate(instance=instance, schema=schema)
            return True
        except jsonschema.ValidationError as e:
            return False
    return BuiltinFunction("json_schema_validate", 2, json_schema_validate_fn)


def make_json_schema_draft_fn():
    def json_schema_draft_fn(args: List[Any]) -> str:
        import json
        try:
            import jsonschema
        except ImportError:
            raise RuntimeError(Token(TokenType.IDENTIFIER, "json_schema_draft", None, 0, 0), "jsonschema library not installed")
        schema = args[0]
        return jsonschema.Draft7Validator.META_SCHEMAS.get("http://json-schema.org/draft-07/schema#", "unknown")
    return BuiltinFunction("json_schema_draft", 1, json_schema_draft_fn)


def stringify(value: Any) -> str:
    if value is None:
        return "nil"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, float):
        text = str(value)
        if text.endswith(".0"):
            text = text[:-2]
        return text
    if isinstance(value, (list, dict)):
        return str(value)
    return str(value)


def create_globals() -> Environment:
    env = Environment()
    
    # Built-in functions
    env.define("print", make_print_fn())
    env.define("len", make_len_fn())
    env.define("str", make_str_fn())
    env.define("int", make_int_fn())
    env.define("float", make_float_fn())
    env.define("range", make_range_fn())
    env.define("type", make_type_fn())
    env.define("input", make_input_fn())
    env.define("list", make_list_fn())
    env.define("dict", make_dict_fn())
    env.define("assert", make_assert_fn())
    
    # File I/O
    env.define("read_file", make_read_file_fn())
    env.define("write_file", make_write_file_fn())
    env.define("append_file", make_append_file_fn())
    env.define("file_exists", make_file_exists_fn())
    env.define("is_file", make_is_file_fn())
    env.define("is_dir", make_is_dir_fn())
    env.define("list_dir", make_list_dir_fn())
    env.define("mkdir", make_mkdir_fn())
    env.define("remove", make_remove_fn())
    env.define("rename", make_rename_fn())
    env.define("file_size", make_file_size_fn())
    
    # JSON
    env.define("json_parse", make_json_parse_fn())
    env.define("json_stringify", make_json_stringify_fn())
    
    # HTTP
    env.define("http_get", make_http_get_fn())
    env.define("http_post", make_http_post_fn())
    
    # Regex
    env.define("regex_match", make_regex_match_fn())
    env.define("regex_findall", make_regex_findall_fn())
    env.define("regex_replace", make_regex_replace_fn())
    env.define("regex_split", make_regex_split_fn())
    
    # Math
    env.define("sin", make_math_sin_fn())
    env.define("cos", make_math_cos_fn())
    env.define("tan", make_math_tan_fn())
    env.define("sqrt", make_math_sqrt_fn())
    env.define("pow", make_math_pow_fn())
    env.define("log", make_math_log_fn())
    env.define("exp", make_math_exp_fn())
    env.define("floor", make_math_floor_fn())
    env.define("ceil", make_math_ceil_fn())
    env.define("random", make_math_random_fn())
    env.define("randint", make_math_randint_fn())
    env.define("pi", make_math_pi_fn())
    env.define("e", make_math_e_fn())
    
    # Datetime
    env.define("now", make_datetime_now_fn())
    env.define("parse_datetime", make_datetime_parse_fn())
    env.define("timestamp", make_datetime_timestamp_fn())
    env.define("sleep", make_datetime_sleep_fn())
    env.define("async_sleep", make_datetime_async_sleep_fn())
    
    # OS utilities
    env.define("env", make_os_env_fn())
    env.define("args", make_os_args_fn())
    env.define("exit", make_os_exit_fn())
    env.define("cwd", make_os_cwd_fn())
    env.define("chdir", make_os_chdir_fn())
    env.define("platform", make_os_platform_fn())
    
    # SQLite
    env.define("sqlite_open", make_sqlite_open_fn())
    env.define("sqlite_exec", make_sqlite_exec_fn())
    env.define("sqlite_query", make_sqlite_query_fn())
    env.define("sqlite_close", make_sqlite_close_fn())
    
    # HTTP improvements
    env.define("http_put", make_http_put_fn())
    env.define("http_delete", make_http_delete_fn())
    env.define("http_patch", make_http_patch_fn())
    env.define("http_head", make_http_head_fn())
    
    # JSON Schema validation
    env.define("json_schema_validate", make_json_schema_validate_fn())
    env.define("json_schema_draft", make_json_schema_draft_fn())
    
    # Built-in constants
    env.define("true", True)
    env.define("false", False)
    env.define("nil", None)
    
    return env