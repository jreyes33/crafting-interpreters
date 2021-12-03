use crate::ast;
use crate::object::Object;
use crate::token::Token;
use crate::Result;
use std::rc::Rc;

pub type VisitorResult = Result<Rc<dyn Object>>;

ast!(Expr -> VisitorResult [
    Assign(name: Token, value: Box<dyn Expr>),
    Binary(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>),
    Call(callee: Box<dyn Expr>, paren: Token, arguments: Vec<Box<dyn Expr>>),
    Get(object: Rc<dyn Expr>, name: Token),
    Grouping(expression: Box<dyn Expr>),
    Literal(value: Rc<dyn Object>),
    Logical(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>),
    Set(object: Rc<dyn Expr>, name: Token, value: Box<dyn Expr>),
    This(keyword: Token),
    Unary(operator: Token, right: Box<dyn Expr>),
    Variable(name: Token),
]);
