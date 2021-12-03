use crate::callable::{CallResult, Callable, LoxFunction};
use crate::instance::Instance;
use crate::interpreter::Interpreter;
use crate::object::Object;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Class {
    pub name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn find_method<S: AsRef<str>>(&self, name: S) -> Option<Rc<LoxFunction>> {
        self.methods.get(name.as_ref()).cloned()
    }
}

impl Object for Class {}
impl Object for Rc<Class> {}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Callable for Rc<Class> {
    fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(initializer) => initializer.arity(),
            None => 0,
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Rc<dyn Object>]) -> CallResult {
        let instance = Rc::new(Instance::new(self.clone()));
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(instance).call(interpreter, arguments)
        } else {
            Ok(instance)
        }
    }
}
