use crate::error::Error;
use crate::expr::{
    Assign, Binary, Call, Expr, Get, Grouping, Literal, Logical, Set, This, Unary, Variable,
};
use crate::object::Nil;
use crate::stmt::{Block, Class, Expression, Function, If, Print, Return, Stmt, Var, While};
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};
use crate::Result;
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
        let result: StmtResult = if self.matches(&[Class]) {
            self.class_declaration()
        } else if self.matches(&[Fun]) {
            Ok(Box::new(self.function("function")?))
        } else if self.matches(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match result {
            Ok(s) => Ok(s),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn class_declaration(&mut self) -> StmtResult {
        let name = self.consume(&Identifier(Default::default()), "Expect class name.")?;
        self.consume(&LeftBrace, "Expect '{' before class body.")?;
        let mut methods = vec![];
        while !self.check(&RightBrace) && !self.is_at_end() {
            methods.push(Rc::new(self.function("method")?));
        }
        self.consume(&RightBrace, "Expect '}' after class body.")?;
        Ok(Class::boxed(name, methods))
    }

    fn statement(&mut self) -> StmtResult {
        if self.matches(&[For]) {
            self.for_statement()
        } else if self.matches(&[If]) {
            self.if_statement()
        } else if self.matches(&[Print]) {
            self.print_statement()
        } else if self.matches(&[Return]) {
            self.return_statement()
        } else if self.matches(&[While]) {
            self.while_statement()
        } else if self.matches(&[LeftBrace]) {
            Ok(Block::boxed(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> StmtResult {
        self.consume(&LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.matches(&[Semicolon]) {
            None
        } else if self.matches(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if self.check(&Semicolon) {
            Literal::boxed(Rc::new(true))
        } else {
            self.expression()?
        };
        self.consume(&Semicolon, "Expect ';' after loop condition.")?;
        let increment = if self.check(&RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&RightParen, "Expect ')' after for clauses.")?;
        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Block::boxed(vec![body, Expression::boxed(inc)]);
        }
        body = While::boxed(condition, body);
        if let Some(init) = initializer {
            body = Block::boxed(vec![init, body]);
        }
        Ok(body)
    }

    fn if_statement(&mut self) -> StmtResult {
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after condition.")?;
        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[Else]) {
            Some(self.statement()?)
        } else {
            None
        };
        Ok(If::boxed(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> StmtResult {
        let value = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;
        Ok(Print::boxed(value))
    }

    fn return_statement(&mut self) -> StmtResult {
        let keyword = self.previous();
        let value = if !self.check(&Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Semicolon, "Expect ';' after return value.")?;
        Ok(Return::boxed(keyword, value))
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

    fn while_statement(&mut self) -> StmtResult {
        self.consume(&LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(While::boxed(condition, body))
    }

    fn expression_statement(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after expression.")?;
        Ok(Expression::boxed(expr))
    }

    fn function(&mut self, kind: &str) -> Result<Function> {
        let name = self.consume(
            &Identifier(Default::default()),
            &format!("Expect {} name.", kind),
        )?;
        self.consume(&LeftParen, &format!("Expect '(' after {} name.", kind))?;
        let mut parameters = vec![];
        if !self.check(&RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(&self.peek(), "Can't have more than 255 parameters.");
                }
                let p = self.consume(&Identifier(Default::default()), "Expect parameter name.")?;
                parameters.push(p);
                if !self.matches(&[Comma]) {
                    break;
                }
            }
        }
        self.consume(&RightParen, "Expect ')' after parameters.")?;
        self.consume(&LeftBrace, &format!("Expect '{{' before {} body.", kind))?;
        // Convert from a Vec<Box> into a Vec<Rc>.
        let body = self.block()?.into_iter().map(From::from).collect();
        Ok(Function::new(name, parameters, body))
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
        let expr = self.or()?;
        if self.matches(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            let expr_any = (*expr).as_any();
            if let Some(v) = expr_any.downcast_ref::<Variable>() {
                return Ok(Assign::boxed(v.name.clone(), value));
            } else if let Some(g) = expr_any.downcast_ref::<Get>() {
                return Ok(Set::boxed(g.object.clone(), g.name.clone(), value));
            }
            self.error(&equals, "Invalid assignment target.");
        }
        Ok(expr)
    }

    // TODO: extract left_assoc_binary and left_assoc_logical helpers.
    fn or(&mut self) -> ExprResult {
        let mut expr = self.and()?;
        while self.matches(&[Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Logical::boxed(expr, operator, right);
        }
        Ok(expr)
    }

    fn and(&mut self) -> ExprResult {
        let mut expr = self.equality()?;
        while self.matches(&[And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Logical::boxed(expr, operator, right);
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
        self.call()
    }

    fn call(&mut self) -> ExprResult {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[Dot]) {
                let name = self.consume(
                    &Identifier(Default::default()),
                    "Expect property name after '.'.",
                )?;
                expr = Get::boxed(expr.into(), name);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<dyn Expr>) -> ExprResult {
        let mut arguments = vec![];
        if !self.check(&RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error(&self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.matches(&[Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(&RightParen, "Expect ')' after arguments.")?;
        Ok(Call::boxed(callee, paren, arguments))
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
        } else if self.matches(&[This]) {
            Ok(This::boxed(self.previous()))
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

    // TODO: create new error enum variant to include detailed information.
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
            self.advance();
        }
    }
}
