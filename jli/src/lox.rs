use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;
use crate::Result;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::{fs, io, process};

#[derive(Default)]
pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Rc<RefCell<Interpreter>>,
}

impl Lox {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run_file(&mut self, path: String) -> Result<()> {
        let source = fs::read_to_string(path)?;
        self.run(source);
        // Indicate an error in the exit code.
        if self.had_error {
            // TODO: bubble up error to main and exit there.
            process::exit(65);
        } else if self.had_runtime_error {
            // TODO: right now this is never true.
            process::exit(70);
        }
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        loop {
            let mut line = String::new();
            print!("> ");
            io::stdout().flush()?;
            let bytes_read = io::stdin().read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }
            self.run(line);
            self.had_error = false;
        }
        Ok(())
    }

    fn run(&mut self, source: String) {
        let mut on_error = |line, message| {
            self.report(line, "", message);
        };
        let mut scanner = Scanner::new(source, &mut on_error);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            // Stop if there was a syntax error.
            Err(e) => {
                eprintln!("{}", e);
                process::exit(65);
            }
            Ok(statements) => {
                let mut resolver = Resolver::new(self.interpreter.clone());
                resolver.resolve(&statements).unwrap();
                self.interpreter
                    .borrow_mut()
                    .interpret(&statements)
                    .unwrap();
            }
        }
    }

    fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }
}
