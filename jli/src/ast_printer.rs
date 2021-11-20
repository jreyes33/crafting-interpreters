use crate::expr::{
    Assign, Binary, Expr, Grouping, Literal, Unary, Variable, Visitor, VisitorResult,
};
use crate::token::{Token, TokenType};
use std::rc::Rc;

pub struct AstPrinter;

impl Visitor<VisitorResult> for AstPrinter {
    fn visit_assign_expr(&mut self, _expr: &Assign) -> VisitorResult {
        todo!();
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> VisitorResult {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> VisitorResult {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> VisitorResult {
        Ok(Rc::new(format!("{}", expr.value)))
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> VisitorResult {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
    }

    fn visit_variable_expr(&mut self, expr: &Variable) -> VisitorResult {
        Ok(Rc::new(expr.name.lexeme.clone()))
    }
}

impl AstPrinter {
    pub fn print(&mut self, expr: &dyn Expr) -> String {
        format!("{}", expr.accept(self).unwrap())
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&dyn Expr]) -> VisitorResult {
        let mut result = String::new();
        result += "(";
        result += name;
        for expr in exprs {
            result += " ";
            result += &format!("{}", expr.accept(self).unwrap());
        }
        result += ")";
        Ok(Rc::new(result))
    }
}

// TODO: this function is only used for testing; delete it.
pub fn run() {
    let expression = Binary::boxed(
        Unary::boxed(
            Token::new(TokenType::Minus, "-".to_string(), 1),
            Literal::boxed(Rc::new(123.0)),
        ),
        Token::new(TokenType::Star, "*".to_string(), 1),
        Grouping::boxed(Literal::boxed(Rc::new(45.67))),
    );
    let mut printer = AstPrinter {};
    println!("{}", printer.print(&*expression));
}
