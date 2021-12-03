use crate::class::Class;
use crate::object::Object;
use crate::token::Token;
use crate::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct Instance {
    class: Rc<Class>,
    fields: RefCell<HashMap<String, Rc<dyn Object>>>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: RefCell::new(HashMap::new()),
        }
    }
}

pub trait InstanceGet {
    fn get(&self, name: &Token) -> Result<Rc<dyn Object>>;
}

impl InstanceGet for Rc<Instance> {
    fn get(&self, name: &Token) -> Result<Rc<dyn Object>> {
        if let Some(field) = self.fields.borrow().get(&name.lexeme) {
            return Ok(field.clone());
        }
        if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(Rc::new(method.bind(self.clone())))
        } else {
            Err(format!("Undefined property '{}'.", name.lexeme).into())
        }
    }
}

impl Instance {
    pub fn set(&self, name: &Token, value: Rc<dyn Object>) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}

impl Object for Instance {}

impl PartialEq for Instance {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
