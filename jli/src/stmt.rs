use crate::ast;
use crate::expr::Expr;
use crate::token::Token;
use crate::Result;

type VisitorResult = Result<()>;

ast!(Stmt -> VisitorResult [
    Block(statements: Vec<Box<dyn Stmt>>),
    Expression(expression: Box<dyn Expr>),
    Print(expression: Box<dyn Expr>),
    Var(name: Token, initializer: Box<dyn Expr>),
]);
