mod ast;
mod builtins;
mod hole;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod typecheck;

use clap::{Parser as ClapParser, Subcommand};
use colored::*;
use std::fs;
use std::process;

#[derive(ClapParser)]
#[command(name = "hole", about = "Hole compiler — an LLM-first programming language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .hl file
    Run {
        /// Source file
        file: String,
    },
    /// Type-check a .hl file (reports errors and holes)
    Check {
        /// Source file
        file: String,
        /// Output as JSON (for LLM tool use)
        #[arg(long)]
        json: bool,
    },
    /// Start interactive REPL
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            if let Err(e) = run_file(&file) {
                eprintln!("{} {}", "error:".red().bold(), e);
                process::exit(1);
            }
        }
        Commands::Check { file, json } => {
            if let Err(e) = check_file(&file, json) {
                eprintln!("{} {}", "error:".red().bold(), e);
                process::exit(1);
            }
        }
        Commands::Repl => {
            repl::run_repl();
        }
    }
}

fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read '{}': {}", path, e))
}

fn parse_program(source: &str) -> Result<crate::ast::Program, String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_program().map_err(|e| e.to_string())
}

fn run_file(path: &str) -> Result<(), String> {
    let source = read_file(path)?;
    let program = parse_program(&source)?;

    let builtins = builtins::BuiltinCtx::default();
    let mut type_env = typecheck::TypeEnv::new(&builtins);

    if let Err(e) = type_env.check_program(&program) {
        return Err(format!("type error: {}", e));
    }

    if !type_env.holes.is_empty() {
        let report = hole::analyze_holes(&type_env.holes, &source);
        for h in &report.holes {
            eprintln!(
                "{} {}:{}: hole of type {}",
                "hole:".yellow().bold(),
                h.line,
                h.col,
                h.expected_type.cyan()
            );
        }
        return Err("program contains holes".into());
    }

    let mut interp = interpreter::Interpreter::new();
    match interp.eval_program(&program) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("runtime error: {}", e)),
    }
}

fn check_file(path: &str, json: bool) -> Result<(), String> {
    let source = read_file(path)?;
    let program = parse_program(&source)?;

    let builtins = builtins::BuiltinCtx::default();
    let mut type_env = typecheck::TypeEnv::new(&builtins);

    let type_errors: Vec<String> = match type_env.check_program(&program) {
        Ok(()) => vec![],
        Err(e) => vec![e.to_string()],
    };

    let hole_report = hole::analyze_holes(&type_env.holes, &source);

    if json {
        #[derive(serde::Serialize)]
        struct JsonReport {
            errors: Vec<serde_json::Value>,
            holes: Vec<serde_json::Value>,
        }

        let mut json_errors = vec![];
        for e in &type_errors {
            json_errors.push(serde_json::json!({
                "message": e,
            }));
        }

        let mut json_holes = vec![];
        for h in &hole_report.holes {
            json_holes.push(serde_json::json!({
                "line": h.line,
                "col": h.col,
                "expected_type": h.expected_type,
                "context": h.context,
            }));
        }

        let report = JsonReport {
            errors: json_errors,
            holes: json_holes,
        };

        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        if type_errors.is_empty() && hole_report.holes.is_empty() {
            println!("{}", "OK".green().bold());
        } else {
            for e in &type_errors {
                eprintln!("{} {}", "error:".red().bold(), e);
            }
            for h in &hole_report.holes {
                eprintln!(
                    "{} {}:{}: hole of type {}\n  {} | {}",
                    "hole:".yellow().bold(),
                    h.line,
                    h.col,
                    h.expected_type.cyan(),
                    h.line,
                    h.context.trim()
                );
            }
        }
    }

    Ok(())
}
