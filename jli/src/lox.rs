use crate::ast_printer::AstPrinter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::Result;
use std::io::Write;
use std::{fs, io, process};

#[derive(Default)]
pub struct Lox {
    had_error: bool,
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
        // TODO: don't pass a mutable Lox reference only to report errors from other places.
        // Alternatives are:
        // - making had_error an AtomicBool, getting rid of the Lox struct
        // - passing down an error callback instead of the reference to the whole struct
        let mut scanner = Scanner::new(source, self);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            // Stop if there was a syntax error.
            Err(e) => {
                eprintln!("{}", e);
                process::exit(65);
            }
            Ok(expr) => {
                let printer = AstPrinter {};
                println!("{}", printer.print(&*expr));
            }
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }
}
