use crate::environment::Environment;
use crate::expr::{self, Assign, Binary, Expr, Grouping, Literal, Unary, Variable};
use crate::object::{Nil, Object};
use crate::stmt::{self, Block, Expression, Print, Stmt, Var};
use crate::token::TokenType::*;
use crate::Result;
use std::rc::Rc;

#[derive(Default)]
pub struct Interpreter {
    environment: Rc<Environment>,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Box<dyn Stmt>]) -> Result<()> {
        for statement in statements {
            self.execute(&**statement)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &dyn Expr) -> expr::VisitorResult {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &dyn Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &[Box<dyn Stmt>],
        environment: Rc<Environment>,
    ) -> Result<()> {
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
}

impl expr::Visitor<expr::VisitorResult> for Interpreter {
    fn visit_assign_expr(&mut self, expr: &Assign) -> expr::VisitorResult {
        let value = self.evaluate(&*expr.value)?;
        self.environment.assign(&expr.name, value.clone())?;
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

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> expr::VisitorResult {
        self.evaluate(&*expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> expr::VisitorResult {
        Ok(expr.value.clone())
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
        self.environment.get(&expr.name)
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

    fn visit_print_stmt(&mut self, stmt: &Print) -> stmt::VisitorResult {
        let value = self.evaluate(&*stmt.expression)?;
        println!("{}", stringify(&*value));
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &Var) -> stmt::VisitorResult {
        let value = match &stmt.initializer {
            Some(initializer) => self.evaluate(&**initializer)?,
            None => Rc::new(Nil),
        };
        self.environment.define(&stmt.name.lexeme, value);
        Ok(())
    }
}

fn stringify(object: &dyn Object) -> String {
    format!("{}", object)
}
