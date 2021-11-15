use crate::expr::{Binary, Expr, Grouping, Literal, Nil, Unary};
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};
use crate::{Error, Result};
use std::mem::discriminant;

pub struct Parser<'p> {
    tokens: &'p [Token],
    current: usize,
}

type ParserResult = Result<Box<dyn Expr>>;

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParserResult {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult {
        let mut expr = self.comparison()?;
        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult {
        let mut expr = self.term()?;
        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParserResult {
        let mut expr = self.factor()?;
        while self.matches(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Binary::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Unary::boxed(operator, right));
        }
        self.primary()
    }

    fn primary(&mut self) -> ParserResult {
        if self.matches(&[False]) {
            Ok(Literal::boxed(Box::new(false)))
        } else if self.matches(&[True]) {
            Ok(Literal::boxed(Box::new(true)))
        } else if self.matches(&[TokenType::Nil]) {
            Ok(Literal::boxed(Box::new(Nil)))
        // TODO: is there a better way to do this without instantiating dummy variants?
        } else if self.matches(&[Number(Default::default()), LoxString(Default::default())]) {
            match self.previous().token_type {
                Number(n) => Ok(Literal::boxed(Box::new(n))),
                LoxString(s) => Ok(Literal::boxed(Box::new(s))),
                _ => Err("not a number or string".into()),
            }
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
        format!(
            "[line {}] Error at {}: {}",
            token.line, token.lexeme, message
        )
        .into()
    }
}
