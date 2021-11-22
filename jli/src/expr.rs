use crate::ast;
use crate::object::Object;
use crate::token::Token;
use crate::Result;
use std::rc::Rc;

pub type VisitorResult = Result<Rc<dyn Object>>;

ast!(Expr -> VisitorResult [
    Assign(name: Token, value: Box<dyn Expr>),
    Binary(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>),
    Grouping(expression: Box<dyn Expr>),
    Literal(value: Rc<dyn Object>),
    Logical(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>),
    Unary(operator: Token, right: Box<dyn Expr>),
    Variable(name: Token),
]);
