import sys
import os
from pathlib import Path
from .lexer import tokenize
from .parser import parse
from .interpreter import Interpreter
from .repl import REPL
from .errors import BhabexError


def run_file(file_path: str):
    """Run a .bx file."""
    path = Path(file_path)
    if not path.exists():
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
    
    if not path.suffix == '.bx':
        print(f"Warning: File '{file_path}' does not have .bx extension.")
    
    source = path.read_text(encoding='utf-8')
    run_source(source, str(path))


def run_source(source: str, filename: str = "<stdin>"):
    """Run source code."""
    try:
        tokens = tokenize(source)
        program = parse(tokens)
        interpreter = Interpreter()
        interpreter.interpret(program)
    except BhabexError as e:
        print(f"Error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


def check_file(file_path: str):
    """Parse and type check a file without running."""
    path = Path(file_path)
    if not path.exists():
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
    
    source = path.read_text(encoding='utf-8')
    try:
        tokens = tokenize(source)
        program = parse(tokens)
        print(f"OK: {file_path} parsed successfully.")
        print(f"  Statements: {len(program.statements)}")
    except BhabexError as e:
        print(f"Error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


def show_tokens(file_path: str):
    """Show tokens for a file."""
    path = Path(file_path)
    if not path.exists():
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
    
    source = path.read_text(encoding='utf-8')
    try:
        tokens = tokenize(source)
        for token in tokens:
            print(token)
    except BhabexError as e:
        print(f"Error: {e}")
        sys.exit(1)


def run_repl():
    """Start the REPL."""
    repl = REPL()
    repl.run()


def eval_code(code: str):
    """Evaluate inline code."""
    run_source(code, "<eval>")


def init_project(project_name: str):
    """Scaffold a new bhabex project."""
    path = Path(project_name)
    if path.exists():
        print(f"Error: Directory '{project_name}' already exists.")
        sys.exit(1)
    
    path.mkdir(parents=True)
    
    # Create main.bx
    main_bx = path / "main.bx"
    main_bx.write_text('print("Hello, Bhabex!")\n')
    
    # Create README.md
    readme = path / "README.md"
    readme.write_text(f"# {project_name}\n\nA Bhabex project.\n")
    
    print(f"Created project '{project_name}'")
    print(f"  {main_bx}")
    print(f"  {readme}")


def pkg_command(args: list):
    """Handle pkg subcommands."""
    if not args:
        print("Error: Missing pkg subcommand.")
        print("Usage: bx pkg <init|add|install|list> [args]")
        sys.exit(1)
    
    subcommand = args[0]
    
    if subcommand == "init":
        pkg_init(args[1:])
    elif subcommand == "add":
        if len(args) < 2:
            print("Error: Missing package name.")
            print("Usage: bx pkg add <package>")
            sys.exit(1)
        pkg_add(args[1])
    elif subcommand == "install":
        pkg_install()
    elif subcommand == "list":
        pkg_list()
    else:
        print(f"Error: Unknown pkg subcommand '{subcommand}'")
        print("Usage: bx pkg <init|add|install|list> [args]")
        sys.exit(1)


def pkg_init(args: list):
    """Initialize a new package."""
    import json
    from pathlib import Path
    
    name = args[0] if args else Path.cwd().name
    version = "0.1.0"
    description = ""
    
    pkg_json = {
        "name": name,
        "version": version,
        "description": description,
        "main": "main.bx",
        "scripts": {
            "test": "bx test"
        },
        "dependencies": {},
        "devDependencies": {}
    }
    
    pkg_path = Path("bxpkg.json")
    if pkg_path.exists():
        print(f"Package already initialized at {pkg_path}")
        return
    
    pkg_path.write_text(json.dumps(pkg_json, indent=2))
    print(f"Initialized package '{name}' at {pkg_path}")


def pkg_add(package: str):
    """Add a dependency."""
    import json
    from pathlib import Path
    
    pkg_path = Path("bxpkg.json")
    if not pkg_path.exists():
        print("Error: No bxpkg.json found. Run 'bx pkg init' first.")
        sys.exit(1)
    
    pkg = json.loads(pkg_path.read_text())
    
    # Parse package@version
    if "@" in package:
        name, version = package.split("@", 1)
    else:
        name = package
        version = "*"
    
    pkg["dependencies"][name] = version
    
    pkg_path.write_text(json.dumps(pkg, indent=2))
    print(f"Added {name}@{version} to dependencies")


def pkg_install():
    """Install dependencies."""
    import json
    from pathlib import Path
    
    pkg_path = Path("bxpkg.json")
    if not pkg_path.exists():
        print("Error: No bxpkg.json found. Run 'bx pkg init' first.")
        sys.exit(1)
    
    pkg = json.loads(pkg_path.read_text())
    deps = pkg.get("dependencies", {})
    
    if not deps:
        print("No dependencies to install")
        return
    
    print("Installing dependencies...")
    for name, version in deps.items():
        print(f"  {name}@{version} - would download from registry")
    
    print("Done!")


def pkg_list():
    """List installed packages."""
    import json
    from pathlib import Path
    
    pkg_path = Path("bxpkg.json")
    if not pkg_path.exists():
        print("No bxpkg.json found")
        return
    
    pkg = json.loads(pkg_path.read_text())
    deps = pkg.get("dependencies", {})
    dev_deps = pkg.get("devDependencies", {})
    
    if deps:
        print("Dependencies:")
        for name, version in deps.items():
            print(f"  {name}@{version}")
    
    if dev_deps:
        print("Dev Dependencies:")
        for name, version in dev_deps.items():
            print(f"  {name}@{version}")
    
    if not deps and not dev_deps:
        print("No dependencies")


def fmt_file(args: list):
    """Format a .bx file."""
    from pathlib import Path
    from .formatter import format_file
    
    if not args:
        print("Error: Missing file argument.")
        print("Usage: bx fmt <file.bx> [--indent N]")
        sys.exit(1)
    
    file_path = args[0]
    indent_size = 4
    
    if len(args) > 1 and args[1] == "--indent":
        if len(args) < 3:
            print("Error: Missing indent value.")
            sys.exit(1)
        try:
            indent_size = int(args[2])
        except ValueError:
            print("Error: Invalid indent value.")
            sys.exit(1)
    
    path = Path(file_path)
    if not path.exists():
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
    
    if not path.suffix == '.bx':
        print(f"Warning: File '{file_path}' does not have .bx extension.")
    
    try:
        from .formatter import format_file as fmt_file_func
        fmt_file_func(file_path, indent_size)
        print(f"Formatted {file_path}")
    except Exception as e:
        print(f"Error formatting file: {e}")
        sys.exit(1)


def run_tests(path: str):
    """Run tests from a file or directory."""
    from pathlib import Path
    p = Path(path)
    
    if not p.exists():
        print(f"Error: Path '{path}' not found.")
        sys.exit(1)
    
    test_files = []
    if p.is_file():
        test_files = [p]
    else:
        test_files = list(p.rglob("*_test.bx")) + list(p.rglob("test_*.bx"))
    
    if not test_files:
        print(f"No test files found in '{path}'")
        sys.exit(0)
    
    total_passed = 0
    total_failed = 0
    
    for test_file in test_files:
        print(f"\nRunning tests from {test_file}...")
        passed, failed = run_test_file(test_file)
        total_passed += passed
        total_failed += failed
    
    print(f"\n=== Test Summary ===")
    print(f"Passed: {total_passed}")
    print(f"Failed: {total_failed}")
    print(f"Total:  {total_passed + total_failed}")
    
    if total_failed > 0:
        sys.exit(1)


def run_test_file(file_path: Path) -> tuple:
    """Run a single test file and return (passed, failed) counts."""
    from .lexer import tokenize
    from .parser import parse
    from .interpreter import Interpreter
    from .errors import BhabexError
    
    source = file_path.read_text(encoding='utf-8')
    
    try:
        tokens = tokenize(source)
        program = parse(tokens)
        interpreter = Interpreter()
        interpreter.interpret(program)
        # If we get here without errors, the test passed
        print(f"  PASS: {file_path.name}")
        return (1, 0)
    except BhabexError as e:
        print(f"  FAIL: {file_path.name} - {e}")
        return (0, 1)
    except Exception as e:
        print(f"  ERROR: {file_path.name} - {e}")
        return (0, 1)


def show_help():
    print("""
Bhabex (.bx) - A modern, multi-paradigm programming language

Usage:
  bx <command> [args]

Commands:
  run <file.bx>        Run a Bhabex file
  check <file.bx>      Parse and type check a file
  tokens <file.bx>     Show tokens for a file
  repl                 Start interactive REPL
  eval "code"          Evaluate inline code
  init <project>       Scaffold a new project
  test <file|dir>      Run tests
  fmt <file.bx>        Format a file
  lsp                  Start LSP server (stdio)
  pkg <subcommand>     Package management
  help                 Show this help
  version              Show version

Package Commands:
  pkg init             Initialize a new package (creates bxpkg.json)
  pkg add <pkg@ver>    Add a dependency
  pkg install          Install dependencies
  pkg list             List installed packages

Examples:
  bx run examples/demo.bx
  bx check main.bx
  bx tokens main.bx
  bx repl
  bx eval "print(2+2)"
  bx init myproject
  bx test tests/
  bx fmt main.bx
  bx lsp
  bx pkg init
  bx pkg add requests@1.0.0
  bx pkg install
  bx pkg list
""")


def show_version():
    print("Bhabex v0.1.0")


def main():
    if len(sys.argv) < 2:
        show_help()
        sys.exit(0)
    
    command = sys.argv[1]
    
    if command == "run":
        if len(sys.argv) < 3:
            print("Error: Missing file argument.")
            print("Usage: bx run <file.bx>")
            sys.exit(1)
        run_file(sys.argv[2])
    
    elif command == "check":
        if len(sys.argv) < 3:
            print("Error: Missing file argument.")
            print("Usage: bx check <file.bx>")
            sys.exit(1)
        check_file(sys.argv[2])
    
    elif command == "tokens":
        if len(sys.argv) < 3:
            print("Error: Missing file argument.")
            print("Usage: bx tokens <file.bx>")
            sys.exit(1)
        show_tokens(sys.argv[2])
    
    elif command == "repl":
        run_repl()
    
    elif command == "eval":
        if len(sys.argv) < 3:
            print("Error: Missing code argument.")
            print("Usage: bx eval \"code\"")
            sys.exit(1)
        code = " ".join(sys.argv[2:])
        eval_code(code)
    
    elif command == "init":
        if len(sys.argv) < 3:
            print("Error: Missing project name.")
            print("Usage: bx init <project>")
            sys.exit(1)
        init_project(sys.argv[2])
    
    elif command == "test":
        if len(sys.argv) < 3:
            print("Error: Missing test file or directory.")
            print("Usage: bx test <file.bx> or bx test <dir>")
            sys.exit(1)
        run_tests(sys.argv[2])
    
    elif command == "fmt":
        if len(sys.argv) < 3:
            print("Error: Missing file argument.")
            print("Usage: bx fmt <file.bx> [--indent N]")
            sys.exit(1)
        fmt_file(sys.argv[2:])
    
    elif command == "lsp":
        from .lsp import run_stdio
        run_stdio()
    
    elif command == "pkg":
        if len(sys.argv) < 3:
            print("Error: Missing pkg subcommand.")
            print("Usage: bx pkg <init|add|install|list> [args]")
            sys.exit(1)
        pkg_command(sys.argv[2:])
    
    elif command == "help":
        show_help()
    
    elif command == "version":
        show_version()
    
    else:
        print(f"Error: Unknown command '{command}'")
        show_help()
        sys.exit(1)


if __name__ == "__main__":
    main()