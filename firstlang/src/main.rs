//! Firstlang CLI - Run or interact with Firstlang programs
//!
//! Usage:
//!   firstlang <file.fl>     Run a file
//!   firstlang               Start REPL

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use firstlang::{parse, Interpreter, Value};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let filename = &args[1];
        run_file(filename);
    } else {
        repl();
    }
}

fn run_file(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    match firstlang::run(&source) {
        Ok(value) => println!("{}", value),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Count unmatched opening brackets in a string
/// Returns the nesting depth (positive = unclosed brackets)
fn bracket_depth(s: &str) -> i32 {
    let mut depth = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for c in s.chars() {
        // Brackets inside string literals should not count toward depth.
        if c == '"' && prev_char != '\\' {
            in_string = !in_string;
        }

        if !in_string {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }
        prev_char = c;
    }

    depth
}

fn repl() {
    println!("Firstlang REPL v0.1.0");
    println!("Type expressions to evaluate, or 'quit' to exit.");
    println!();

    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!(">>> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        let mut line = String::new();

        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break; // EOF (Ctrl-D)
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "quit" || trimmed == "exit" {
            println!("Goodbye!");
            break;
        }

        input.push_str(&line);

        // Keep reading continuation lines until every bracket is closed.
        while bracket_depth(&input) > 0 {
            print!("... ");
            stdout.flush().unwrap();

            line.clear();
            if stdin.lock().read_line(&mut line).unwrap() == 0 {
                break;
            }
            input.push_str(&line);
        }

        let input = input.trim();

        match parse(input) {
            Ok(program) => match interpreter.run(&program) {
                Ok(value) => {
                    if value != Value::Unit {
                        println!("{}", value);
                    }
                }
                Err(e) => eprintln!("Runtime error: {}", e),
            },
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
