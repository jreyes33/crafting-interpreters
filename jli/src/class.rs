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
    superclass: Option<Rc<Class>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl Class {
    pub fn new(
        name: String,
        superclass: Option<Rc<Class>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method<S: AsRef<str>>(&self, name: S) -> Option<Rc<LoxFunction>> {
        let name_ref = name.as_ref();
        if self.methods.contains_key(name_ref) {
            return self.methods.get(name_ref).cloned();
        } else if let Some(sc) = &self.superclass {
            return sc.find_method(name);
        }
        None
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
