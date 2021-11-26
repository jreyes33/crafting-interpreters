use crate::callable::{Callable, LoxFunction, NativeFunction};
use crate::environment::Environment;
use crate::error::Error;
use crate::expr::{self, Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable};
use crate::object::{Nil, Object};
use crate::stmt::{self, Block, Expression, Function, If, Print, Return, Stmt, Var, While};
use crate::token::Token;
use crate::token::TokenType::*;
use crate::Result;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct Interpreter {
    globals: Rc<Environment>,
    environment: Rc<Environment>,
    locals: HashMap<usize, usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let globals: Rc<Environment> = Default::default();
        globals.define("clock", Rc::new(NativeFunction::Clock));
        Interpreter {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Box<dyn Stmt>]) -> Result<()> {
        for statement in statements {
            self.execute(&**statement)?;
        }
        Ok(())
    }

    pub fn resolve(&mut self, expr: &dyn Expr, depth: usize) {
        let expr_ptr = expr as *const dyn Expr as *const () as usize;
        self.locals.insert(expr_ptr, depth);
    }

    pub fn execute_block<S>(&mut self, statements: &[S], environment: Rc<Environment>) -> Result<()>
    where
        S: Deref<Target = dyn Stmt>,
    {
        let previous = self.environment.clone();
        self.environment = environment;
        for statement in statements {
            let result = self.execute(&**statement);
            match result {
                Ok(_) => (),
                Err(e) => {
                    self.environment = previous;
                    return Err(e);
                }
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> expr::VisitorResult {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &dyn Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn call_function<F>(
        &mut self,
        function: &F,
        arguments: &[Rc<dyn Object>],
    ) -> expr::VisitorResult
    where
        F: Callable,
    {
        if arguments.len() != function.arity() {
            let message = format!(
                "Expected {} arguments but got {}.",
                function.arity(),
                arguments.len()
            );
            return Err(message.into());
        }
        function.call(self, arguments)
    }

    fn look_up_variable(&self, name: &Token, expr: &dyn Expr) -> expr::VisitorResult {
        let expr_ptr = expr as *const dyn Expr as *const () as usize;
        if let Some(distance) = self.locals.get(&expr_ptr) {
            self.environment.get_at(*distance, &name.lexeme)
        } else {
            self.globals.get(name)
        }
    }
}

impl expr::Visitor<expr::VisitorResult> for Interpreter {
    fn visit_assign_expr(&mut self, expr: &Assign) -> expr::VisitorResult {
        let value = self.evaluate(&*expr.value)?;
        let expr_ptr = expr as *const dyn Expr as *const () as usize;
        if let Some(distance) = self.locals.get(&expr_ptr) {
            self.environment
                .assign_at(*distance, &expr.name, value.clone());
        } else {
            self.globals.assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> expr::VisitorResult {
        let left = &*self.evaluate(&*expr.left)?;
        let right = &*self.evaluate(&*expr.right)?;
        Ok(match expr.operator.token_type {
            BangEqual => Rc::new(!left.equal(right)),
            EqualEqual => Rc::new(left.equal(right)),
            Greater => Rc::new(left.try_gt(right)?),
            GreaterEqual => Rc::new(left.try_ge(right)?),
            Less => Rc::new(left.try_lt(right)?),
            LessEqual => Rc::new(left.try_le(right)?),
            Minus => Rc::new(left.try_sub(right)?),
            Plus => left.try_add(right)?,
            Slash => Rc::new(left.try_div(right)?),
            Star => Rc::new(left.try_mul(right)?),
            _ => unreachable!(),
        })
    }

    fn visit_call_expr(&mut self, expr: &Call) -> expr::VisitorResult {
        let callee = self.evaluate(&*expr.callee)?;
        let mut arguments = vec![];
        for argument in &expr.arguments {
            arguments.push(self.evaluate(&**argument)?);
        }
        let callee_any = callee.as_any();
        // Sadly, downcast_ref::<dyn Callable> doesn't work.
        if let Some(function) = callee_any.downcast_ref::<LoxFunction>() {
            self.call_function(function, &arguments)
        } else if let Some(function) = callee_any.downcast_ref::<NativeFunction>() {
            self.call_function(function, &arguments)
        } else {
            Err("Can only call functions and classes.".into())
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> expr::VisitorResult {
        self.evaluate(&*expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> expr::VisitorResult {
        Ok(expr.value.clone())
    }

    fn visit_logical_expr(&mut self, expr: &Logical) -> expr::VisitorResult {
        let left = self.evaluate(&*expr.left)?;
        if expr.operator.token_type == Or {
            if left.truthy() {
                return Ok(left);
            }
        } else if !left.truthy() {
            return Ok(left);
        }
        self.evaluate(&*expr.right)
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> expr::VisitorResult {
        let right = &*self.evaluate(&*expr.right)?;
        Ok(match expr.operator.token_type {
            Bang => Rc::new(!right.truthy()),
            Minus => Rc::new(right.try_neg()?),
            _ => unreachable!(),
        })
    }

    fn visit_variable_expr(&mut self, expr: &Variable) -> expr::VisitorResult {
        self.look_up_variable(&expr.name, expr)
    }
}

impl stmt::Visitor<stmt::VisitorResult> for Interpreter {
    fn visit_block_stmt(&mut self, stmt: &Block) -> stmt::VisitorResult {
        self.execute_block(
            stmt.statements.as_slice(),
            Rc::new(Environment::new_with_enclosing(self.environment.clone())),
        )?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &Expression) -> stmt::VisitorResult {
        self.evaluate(&*stmt.expression)?;
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &Function) -> stmt::VisitorResult {
        // Poor man's Clone.
        let declaration = Function::new(stmt.name.clone(), stmt.params.clone(), stmt.body.clone());
        let function = Rc::new(LoxFunction::new(declaration, self.environment.clone()));
        self.environment.define(&stmt.name.lexeme, function);
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &If) -> stmt::VisitorResult {
        let value = self.evaluate(&*stmt.condition)?;
        if value.truthy() {
            self.execute(&*stmt.then_branch)?;
        } else if let Some(branch) = &stmt.else_branch {
            self.execute(&**branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &Print) -> stmt::VisitorResult {
        let value = self.evaluate(&*stmt.expression)?;
        println!("{}", stringify(&*value));
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &Return) -> stmt::VisitorResult {
        let value = match &stmt.value {
            Some(v) => self.evaluate(&**v)?,
            None => Rc::new(Nil),
        };
        Err(Error::Return(value))
    }

    fn visit_var_stmt(&mut self, stmt: &Var) -> stmt::VisitorResult {
        let value = match &stmt.initializer {
            Some(initializer) => self.evaluate(&**initializer)?,
            None => Rc::new(Nil),
        };
        self.environment.define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &While) -> stmt::VisitorResult {
        let mut value = self.evaluate(&*stmt.condition)?;
        while value.truthy() {
            self.execute(&*stmt.body)?;
            value = self.evaluate(&*stmt.condition)?;
        }
        Ok(())
    }
}

fn stringify(object: &dyn Object) -> String {
    format!("{}", object)
}
