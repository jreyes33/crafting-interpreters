use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::object::{Nil, Object};
use crate::stmt::Function;
use crate::Result;
use std::fmt;
use std::rc::Rc;

type CallResult = Result<Rc<dyn Object>>;

pub trait Callable: Object {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Rc<dyn Object>]) -> CallResult;
}

#[derive(Debug)]
pub struct LoxFunction {
    declaration: Function,
    closure: Rc<Environment>,
}

impl LoxFunction {
    pub fn new(declaration: Function, closure: Rc<Environment>) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl Object for LoxFunction {}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Rc<dyn Object>]) -> CallResult {
        let environment = Rc::new(Environment::new_with_enclosing(self.closure.clone()));
        for (param, argument) in self.declaration.params.iter().zip(arguments) {
            environment.define(&param.lexeme, argument.clone());
        }
        let result = interpreter.execute_block(&self.declaration.body, environment);
        match result {
            Ok(_) => (),
            Err(e) => {
                return match e {
                    Error::Return(v) => Ok(v),
                    _ => Err(e),
                }
            }
        }
        Ok(Rc::new(Nil))
    }
}

#[derive(Debug, PartialEq)]
pub enum NativeFunction {
    Clock,
}

impl NativeFunction {
    fn clock(&self) -> CallResult {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("NativeFunction unavailable")
            .as_secs() as f64;
        Ok(Rc::new(secs))
    }
}

impl Object for NativeFunction {}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        match self {
            Self::Clock => 0,
        }
    }

    fn call(&self, _: &mut Interpreter, _: &[Rc<dyn Object>]) -> CallResult {
        match self {
            Self::Clock => self.clock(),
        }
    }
}
