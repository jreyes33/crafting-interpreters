use crate::lox::Lox;
use std::env::args;
use std::result;

mod ast_printer;
mod callable;
mod class;
mod environment;
mod error;
mod expr;
mod instance;
mod interpreter;
mod lox;
mod macros;
mod object;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;

type Result<T> = result::Result<T, error::Error>;

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
