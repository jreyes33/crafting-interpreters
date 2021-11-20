use crate::lox::Lox;
use std::env::args;
use std::{error, result};

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod lox;
mod macros;
mod object;
mod parser;
mod scanner;
mod stmt;
mod token;

type Error = Box<dyn error::Error + Send + Sync + 'static>;
type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    if args().count() > 2 {
        println!("Usage: jli [--print-ast] [script]");
    } else if let Some(arg) = args().nth(1) {
        if arg == "--print-ast" {
            ast_printer::run();
        } else {
            Lox::new().run_file(arg)?;
        }
    } else {
        Lox::new().run_prompt()?;
    }
    Ok(())
}
