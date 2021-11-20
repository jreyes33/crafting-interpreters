use crate::expr::{Assign, Binary, Expr, Grouping, Literal, Unary, Variable};
use crate::object::Nil;
use crate::stmt::{Block, Expression, Print, Stmt, Var};
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};
use crate::{Error, Result};
use std::mem::discriminant;
use std::rc::Rc;

// TODO: receive on_error callback to report errors.
pub struct Parser<'p> {
    tokens: &'p [Token],
    current: usize,
}

type ExprResult = Result<Box<dyn Expr>>;
type StmtResult = Result<Box<dyn Stmt>>;

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Stmt>>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> StmtResult {
        let result = if self.matches(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match result {
            Ok(s) => Ok(s),
            Err(e) => {
                // TODO: test if this is working;
                self.synchronize();
                Err(e)
            }
        }
    }

    fn statement(&mut self) -> StmtResult {
        if self.matches(&[Print]) {
            self.print_statement()
        } else if self.matches(&[LeftBrace]) {
            Ok(Block::boxed(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> StmtResult {
        let value = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;
        Ok(Print::boxed(value))
    }

    fn var_declaration(&mut self) -> StmtResult {
        let name = self.consume(&Identifier(Default::default()), "Expect variable name.")?;
        let initializer = if self.matches(&[Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Var::boxed(name, initializer))
    }

    fn expression_statement(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after expression.")?;
        Ok(Expression::boxed(expr))
    }

    fn block(&mut self) -> Result<Vec<Box<dyn Stmt>>> {
        let mut statements = vec![];
        while !self.check(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> ExprResult {
        let expr = self.equality()?;
        if self.matches(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Some(v) = (*expr).as_any().downcast_ref::<Variable>() {
                return Ok(Assign::boxed(v.name.clone(), value));
            }
            self.error(&equals, "Invalid assignment target.");
        }
        Ok(expr)
    }

    fn expression(&mut self) -> ExprResult {
        self.assignment()
    }

    fn equality(&mut self) -> ExprResult {
        let mut expr = self.comparison()?;
        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ExprResult {
        let mut expr = self.term()?;
        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn term(&mut self) -> ExprResult {
        let mut expr = self.factor()?;
        while self.matches(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ExprResult {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ExprResult {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Unary::boxed(operator, right));
        }
        self.primary()
    }

    fn primary(&mut self) -> ExprResult {
        if self.matches(&[False]) {
            Ok(Literal::boxed(Rc::new(false)))
        } else if self.matches(&[True]) {
            Ok(Literal::boxed(Rc::new(true)))
        } else if self.matches(&[TokenType::Nil]) {
            Ok(Literal::boxed(Rc::new(Nil)))
        // TODO: is there a better way to do this without instantiating dummy variants?
        } else if self.matches(&[Number(Default::default()), LoxString(Default::default())]) {
            match self.previous().token_type {
                Number(n) => Ok(Literal::boxed(Rc::new(n))),
                LoxString(s) => Ok(Literal::boxed(Rc::new(s))),
                _ => Err("not a number or string".into()),
            }
        } else if self.matches(&[Identifier(Default::default())]) {
            Ok(Variable::boxed(self.previous()))
        } else if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            Ok(Grouping::boxed(expr))
        } else {
            Err(self.error(&self.peek(), "Expect expression."))
        }
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(&self.peek(), message))
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            discriminant(&self.peek().token_type) == discriminant(token_type)
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    // TODO: can these two calls to clone be removed?
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    // TODO: create own error type.
    fn error(&self, token: &Token, message: &str) -> Error {
        let error_message = format!(
            "[line {}] Error at {}: {}",
            token.line, token.lexeme, message
        );
        eprintln!("{}", error_message);
        error_message.into()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }
            match self.peek().token_type {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => (),
            }
        }
        self.advance();
    }
}
