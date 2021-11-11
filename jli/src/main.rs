use crate::lox::Lox;
use std::env::args;
use std::{error, result};

mod lox;
mod scanner;

type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync + 'static>>;

fn main() -> Result<()> {
    if args().count() > 2 {
        println!("Usage: jli [script]");
    } else if let Some(path) = args().nth(1) {
        Lox::new().run_file(path)?;
    } else {
        Lox::new().run_prompt()?;
    }
    Ok(())
}
