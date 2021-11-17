use crate::expr::{Binary, Expr, Grouping, Literal, Object, Unary, Visitor, VisitorResult};
use crate::token::TokenType::*;
use crate::Result;
use std::rc::Rc;

#[derive(Default)]
pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&self, expr: &dyn Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{}", stringify(&*value));
        Ok(())
    }

    fn evaluate(&self, expr: &dyn Expr) -> VisitorResult {
        expr.accept(self)
    }
}

impl Visitor<VisitorResult> for Interpreter {
    fn visit_binary_expr(&self, expr: &Binary) -> VisitorResult {
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

    fn visit_grouping_expr(&self, expr: &Grouping) -> VisitorResult {
        self.evaluate(&*expr.expression)
    }

    fn visit_literal_expr(&self, expr: &Literal) -> VisitorResult {
        Ok(expr.value.clone())
    }

    fn visit_unary_expr(&self, expr: &Unary) -> VisitorResult {
        let right = &*self.evaluate(&*expr.right)?;
        Ok(match expr.operator.token_type {
            Bang => Rc::new(!right.truthy()),
            Minus => Rc::new(right.try_neg()?),
            _ => unreachable!(),
        })
    }
}

fn stringify(object: &dyn Object) -> String {
    format!("{}", object)
}
