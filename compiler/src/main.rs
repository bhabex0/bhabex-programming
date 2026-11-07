//! Bhabex Compiler Main Entry Point
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use std::env;
use std::fs;
use std::path::Path;
use colored::*;

mod cli;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }

    let command = &args[1];
    
    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: bx build <file.bx> [--release] [--target <target>]".red());
                return;
            }
            let file = &args[2];
            let release = args.contains(&"--release".to_string());
            let target = args.iter().position(|a| a == "--target").and_then(|i| args.get(i + 1)).map(|s| s.as_str());
            build_file(file, release, target);
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: bx run <file.bx> [args...]".red());
                return;
            }
            let file = &args[2];
            let run_args = &args[3..];
            run_file(file, run_args);
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: bx check <file.bx>".red());
                return;
            }
            check_file(&args[2]);
        }
        "fmt" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: bx fmt <file.bx> [--write]".red());
                return;
            }
            format_file(&args[2], args.contains(&"--write".to_string()));
        }
        "test" => {
            test_project(&args[2..]);
        }
        "new" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: bx new <project_name> [--bin|--lib]".red());
                return;
            }
            new_project(&args[2], args.contains(&"--lib".to_string()));
        }
        "pkg" => {
            pkg_command(&args[2..]);
        }
        "version" | "--version" | "-v" => {
            println!("Bhabex Compiler v0.1.0");
            println!("Copyright BHABEX 2026 | Securely designed & developed by bxLabs");
            println!("Protected by bxSecurity | Powered by BHABEX®");
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            eprintln!("{}", format!("Unknown command: {}", command).red());
            print_help();
        }
    }
}

fn print_help() {
    println!("{}", "Bhabex Compiler (bx)".bold().cyan());
    println!("{}", "Copyright BHABEX 2026 | Securely designed & developed by bxLabs");
    println!("{}", "Protected by bxSecurity | Powered by BHABEX®");
    println!();
    println!("{}", "USAGE:".bold());
    println!("  bx <COMMAND> [OPTIONS]");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  build    Build a .bx file to bytecode or native binary");
    println!("  run      Build and run a .bx file");
    println!("  check    Check a .bx file for errors without building");
    println!("  fmt      Format .bx source code");
    println!("  test     Run tests");
    println!("  new      Create a new Bhabex project");
    println!("  pkg      Package manager commands");
    println!("  help     Show this help");
    println!("  version  Show version info");
    println!();
    println!("{}", "BUILD OPTIONS:".bold());
    println!("  --release     Build with optimizations");
    println!("  --target      Target platform (native, wasm, android, ios)");
    println!();
    println!("{}", "EXAMPLES:".bold());
    println!("  bx build main.bx --release");
    println!("  bx run hello.bx");
    println!("  bx fmt main.bx --write");
    println!("  bx new my-app");
    println!("  bx pkg add http@0.5");
}

fn build_file(file: &str, release: bool, target: Option<&str>) {
    println!("{} {}", "Building".bold().green(), file);
    
    let path = Path::new(file);
    if !path.exists() {
        eprintln!("{}", format!("File not found: {}", file).red());
        return;
    }
    
    let source = fs::read_to_string(path).expect("Failed to read file");
    
    // Lex
    let (tokens, lexer_diagnostics) = crate::lexer::tokenize(&source);
    for diag in lexer_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    if lexer_diagnostics.has_errors() {
        return;
    }
    
    // Parse
    let (program, parse_diagnostics, symbols) = crate::parser::parse(&source);
    for diag in parse_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    if parse_diagnostics.has_errors() {
        return;
    }
    
    // Compile
    let compiler = crate::compiler::Compiler::new();
    let (bytecode, compile_diagnostics, _) = compiler.compile(program);
    for diag in compile_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    if compile_diagnostics.has_errors() {
        return;
    }
    
    // Output
    let output_path = if release {
        path.with_extension("bxc")
    } else {
        path.with_extension("bxc")
    };
    
    let binary = crate::bytecode::serialize_bytecode(&bytecode);
    fs::write(&output_path, binary).expect("Failed to write output");
    
    println!("{} {}", "Success!".bold().green(), output_path.display());
    
    if target == Some("native") || target == Some("wasm") {
        println!("{} Native/WASM compilation not yet implemented", "TODO:".yellow());
    }
}

fn run_file(file: &str, args: &[String]) {
    println!("{} {}", "Running".bold().blue(), file);
    
    let path = Path::new(file);
    if !path.exists() {
        eprintln!("{}", format!("File not found: {}", file).red());
        return;
    }
    
    let source = fs::read_to_string(path).expect("Failed to read file");
    
    // Quick compile and run
    let (tokens, _) = crate::lexer::tokenize(&source);
    let (program, diagnostics, _) = crate::parser::parse(&source);
    
    for diag in diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    if diagnostics.has_errors() {
        return;
    }
    
    let compiler = crate::compiler::Compiler::new();
    let (bytecode, compile_diagnostics, _) = compiler.compile(program);
    
    for diag in compile_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    if compile_diagnostics.has_errors() {
        return;
    }
    
    // Run in VM
    let mut vm = crate::vm::VM::new(bytecode);
    match vm.run() {
        Ok(result) => {
            println!("{} {}", "Result:".bold().green(), result);
        }
        Err(e) => {
            eprintln!("{} {}", "Runtime error:".bold().red(), e);
        }
    }
}

fn check_file(file: &str) {
    println!("{} {}", "Checking".bold().yellow(), file);
    
    let source = fs::read_to_string(file).expect("Failed to read file");
    
    let (tokens, lexer_diagnostics) = crate::lexer::tokenize(&source);
    for diag in lexer_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    
    let (program, parse_diagnostics, _) = crate::parser::parse(&source);
    for diag in parse_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    
    let compiler = crate::compiler::Compiler::new();
    let (_, compile_diagnostics, _) = compiler.compile(program);
    for diag in compile_diagnostics.all() {
        print_diagnostic(diag, &source);
    }
    
    let total_errors = lexer_diagnostics.errors().len() + parse_diagnostics.errors().len() + compile_diagnostics.errors().len();
    let total_warnings = lexer_diagnostics.warnings().len() + parse_diagnostics.warnings().len() + compile_diagnostics.warnings().len();
    
    if total_errors == 0 {
        println!("{} No errors found!", "✓".green());
    } else {
        println!("{} Found {} error(s), {} warning(s)", "✗".red(), total_errors, total_warnings);
    }
}

fn format_file(file: &str, write: bool) {
    println!("{} {}", "Formatting".bold().cyan(), file);
    
    let source = fs::read_to_string(file).expect("Failed to read file");
    // Formatter would go here
    let formatted = source; // Placeholder
    
    if write {
        fs::write(file, formatted).expect("Failed to write file");
        println!("{} Written to {}", "✓".green(), file);
    } else {
        println!("{}", formatted);
    }
}

fn test_project(args: &[String]) {
    println!("{} {}", "Testing".bold().magenta(), args.join(" "));
    // Test runner would go here
}

fn new_project(name: &str, is_lib: bool) {
    println!("{} {}", "Creating".bold().green(), name);
    
    fs::create_dir_all(name).expect("Failed to create directory");
    fs::create_dir_all(format!("{}/src", name)).expect("Failed to create src");
    fs::create_dir_all(format!("{}/tests", name)).expect("Failed to create tests");
    
    // bxpkg.toml
    let manifest = format!(r#"[package]
name = "{}"
version = "0.1.0"
description = "A Bhabex project"
license = "MIT"
authors = ["bxLabs <contact@bhabex.dev>"]

[dependencies]

[dev-dependencies]
test = "1.0"
"#, name);
    
    fs::write(format!("{}/bxpkg.toml", name), manifest).expect("Failed to write manifest");
    
    // Main file
    let main_file = if is_lib {
        format!(r#"//! {} library

pub fn hello() -> String {{
    "Hello from {}!"
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_hello() {{
        assert_eq!(hello(), "Hello from {}!");
    }}
}}
"#, name, name, name)
    } else {
        format!(r#"fn main() {{
    println!("Hello from {}!");
}}
"#, name)
    };
    
    fs::write(format!("{}/src/main.bx", name), main_file).expect("Failed to write main");
    
    // Gitignore
    fs::write(format!("{}/.gitignore", name), "target/\n*.bxc\n").expect("Failed to write gitignore");
    
    println!("{} Created project {}", "✓".green(), name);
}

fn pkg_command(args: &[String]) {
    if args.is_empty() {
        eprintln!("{}", "Usage: bx pkg <add|remove|update|install|publish>".red());
        return;
    }
    
    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                eprintln!("{}", "Usage: bx pkg add <package>[@version]".red());
                return;
            }
            println!("{} Adding package: {}", "✓".green(), args[1]);
        }
        "remove" => {
            println!("{} Removing package", "✓".green());
        }
        "update" => {
            println!("{} Updating packages", "✓".green());
        }
        "install" => {
            println!("{} Installing package globally", "✓".green());
        }
        "publish" => {
            println!("{} Publishing package", "✓".green());
        }
        _ => {
            eprintln!("{} Unknown pkg command: {}", "✗".red(), args[0]);
        }
    }
}

fn print_diagnostic(diag: &crate::diagnostics::Diagnostic, source: &str) {
    let severity_str = match diag.severity {
        crate::diagnostics::DiagnosticSeverity::Error => "error".red().bold(),
        crate::diagnostics::DiagnosticSeverity::Warning => "warning".yellow().bold(),
        crate::diagnostics::DiagnosticSeverity::Info => "info".blue().bold(),
        crate::diagnostics::DiagnosticSeverity::Hint => "hint".cyan().bold(),
    };
    
    let line_content = source.lines()
        .nth(diag.span.line.saturating_sub(1))
        .unwrap_or("");
    
    eprintln!("{} {} @ {}", severity_str, diag.message, diag.span);
    eprintln!("{}", format!("  --> {}", line_content.trim()).dimmed());
    
    if let Some(help) = &diag.help {
        eprintln!("  {}", format!("help: {}", help).cyan());
    }
}