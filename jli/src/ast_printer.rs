use crate::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};
use crate::token::{Token, TokenType};

pub struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn visit_grouping_expr(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn visit_literal_expr(&self, expr: &Literal) -> String {
        format!("{}", expr.value)
    }

    fn visit_unary_expr(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
    }
}

impl AstPrinter {
    pub fn print(&self, expr: &dyn Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &str, exprs: &[&dyn Expr]) -> String {
        let mut result = String::new();
        result += "(";
        result += name;
        for expr in exprs {
            result += " ";
            result += &expr.accept(self);
        }
        result += ")";
        result
    }
}

// TODO: this function is only used for testing; delete it.
pub fn run() {
    let expression = Binary::boxed(
        Unary::boxed(
            Token::new(TokenType::Minus, "-".to_string(), 1),
            Literal::boxed(Box::new(123)),
        ),
        Token::new(TokenType::Star, "*".to_string(), 1),
        Grouping::boxed(Literal::boxed(Box::new(45.67))),
    );
    let printer = AstPrinter {};
    println!("{}", printer.print(&*expression));
}
