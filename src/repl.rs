use crate::ast::*;
use crate::interpreter::{Interpreter, RuntimeError};
use crate::lexer::Lexer;
use crate::parser::Parser;
use rustyline::DefaultEditor;

pub fn run_repl() {
    let mut rl = DefaultEditor::new().unwrap();
    let mut interp = Interpreter::new();

    println!("Hole v0.1.0 REPL");
    println!("Type :help for commands, :q to quit");
    println!();

    loop {
        let readline = rl.readline("hole> ");
        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                if line.starts_with(':') {
                    match line.as_str() {
                        ":q" | ":quit" => break,
                        ":help" => {
                            println!("Commands:");
                            println!("  :q, :quit     Exit REPL");
                            println!("  :help         Show this help");
                            println!();
                            println!("Enter a Hole expression to evaluate it.");
                            println!("Multi-line input: end with a blank line.");
                            continue;
                        }
                        _ => {
                            println!("Unknown command: {}", line);
                            continue;
                        }
                    }
                }
                if let Err(e) = eval_line(&line, &mut interp) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(_) => break,
        }
    }
}

fn eval_line(line: &str, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let line = if !line.ends_with('\n') {
        format!("{}\n", line)
    } else {
        line.to_string()
    };

    let mut lexer = Lexer::new(&line);
    let tokens = lexer.tokenize().map_err(|e| RuntimeError::Type {
        span: Span::new(0, 0),
        msg: e.to_string(),
    })?;

    let mut parser = Parser::new(tokens);

    match parser.parse_expr() {
        Ok(expr) => {
            let val = interp.eval(&expr)?;
            let val = interp.run_io(val)?;
            if !matches!(val, crate::interpreter::Value::Unit) {
                println!("{}", val);
            }
            Ok(())
        }
        Err(e) => {
            // Try parsing as a let-binding
            let mut lexer2 = Lexer::new(&line);
            let tokens2 = lexer2.tokenize().map_err(|e| RuntimeError::Type {
                span: Span::new(0, 0),
                msg: e.to_string(),
            })?;
            let mut parser2 = Parser::new(tokens2);

            match parser2.parse_expr() {
                Ok(expr) => {
                    let val = interp.eval(&expr)?;
                    let val = interp.run_io(val)?;
                    if !matches!(val, crate::interpreter::Value::Unit) {
                        println!("{}", val);
                    }
                    Ok(())
                }
                Err(_) => Err(RuntimeError::Type {
                    span: Span::new(0, 0),
                    msg: format!("Parse error: {}", e),
                }),
            }
        }
    }
}
