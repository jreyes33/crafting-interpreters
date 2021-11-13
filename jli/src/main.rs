use crate::lox::Lox;
use std::env::args;
use std::{error, result};

mod ast_printer;
mod expr;
mod lox;
mod scanner;
mod token;

type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync + 'static>>;

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
