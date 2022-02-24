use crate::ast;
use crate::expr::{Expr, Variable};
use crate::token::Token;
use crate::Result;
use std::rc::Rc;

pub type VisitorResult = Result<()>;

ast!(Stmt -> VisitorResult [
    Block(statements: Vec<Box<dyn Stmt>>),
    Class(name: Token, superclass: Option<Variable>, methods: Vec<Rc<Function>>),
    Expression(expression: Box<dyn Expr>),
    Function(name: Token, params: Vec<Token>, body: Vec<Rc<dyn Stmt>>),
    If(condition: Box<dyn Expr>, then_branch: Box<dyn Stmt>, else_branch: Option<Box<dyn Stmt>>),
    Print(expression: Box<dyn Expr>),
    Return(keyword: Token, value: Option<Box<dyn Expr>>),
    Var(name: Token, initializer: Option<Box<dyn Expr>>),
    While(condition: Box<dyn Expr>, body: Box<dyn Stmt>),
]);
