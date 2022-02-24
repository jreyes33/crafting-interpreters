use crate::object::Object;
use crate::token::Token;
use crate::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Environment {
    pub enclosing: Option<Rc<Environment>>,
    values: RefCell<HashMap<String, Rc<dyn Object>>>,
}

impl Environment {
    pub fn new_with_enclosing(enclosing: Rc<Self>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }
    }

    pub fn get(&self, name: &Token) -> Result<Rc<dyn Object>> {
        if let Some(v) = self.values.borrow().get(name.lexeme.as_str()) {
            Ok(v.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(format!("Undefined variable {}.", name.lexeme).into())
        }
    }

    pub fn assign(&self, name: &Token, value: Rc<dyn Object>) -> Result<()> {
        if self.values.borrow().contains_key(&name.lexeme) {
            self.values.borrow_mut().insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(format!("Undefined variable {}.", name.lexeme).into())
        }
    }

    pub fn define(&self, name: &str, value: Rc<dyn Object>) {
        self.values.borrow_mut().insert(name.to_string(), value);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Rc<dyn Object>> {
        let option = self.ancestor(distance).values.borrow().get(name).cloned();
        option.ok_or_else(|| format!("Undefined variable {}", name).into())
    }

    pub fn assign_at(&self, distance: usize, name: &Token, value: Rc<dyn Object>) {
        self.ancestor(distance)
            .values
            .borrow_mut()
            .insert(name.lexeme.clone(), value);
    }

    fn ancestor(&self, distance: usize) -> &Self {
        let mut environment = self;
        for _ in 0..distance {
            environment = environment.enclosing.as_ref().expect("No ancestor");
        }
        environment
    }
}
