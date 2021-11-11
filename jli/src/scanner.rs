use crate::Lox;
use std::fmt;

pub struct Scanner<'s> {
    source: String,
    lox: &'s mut Lox,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'s> Scanner<'s> {
    pub fn new(source: String, lox: &'s mut Lox) -> Self {
        Self {
            source,
            lox,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> &[Token] {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), None, self.line));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        use TokenType::*;
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '!' => {
                let token = if self.matches('=') { BangEqual } else { Bang };
                self.add_token(token);
            }
            '=' => {
                let token = if self.matches('=') { EqualEqual } else { Equal };
                self.add_token(token);
            }
            '<' => {
                let token = if self.matches('=') { LessEqual } else { Less };
                self.add_token(token);
            }
            '>' => {
                let token = if self.matches('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(token);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash);
                }
            }
            // Ignore whitespace.
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.lox.error(self.line, "Unexpected character.");
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }
        // TODO: use .get or .is_char_boundary to avoid issues with UTF-8 chars.
        let text = &self.source[self.start..self.current];
        let token_type = match self.keyword(text) {
            Some(t) => t,
            None => TokenType::Identifier,
        };
        self.add_token(token_type);
    }

    fn keyword(&self, text: &str) -> Option<TokenType> {
        use TokenType::*;
        match text {
            "and" => Some(And),
            "class" => Some(Class),
            "else" => Some(Else),
            "false" => Some(False),
            "for" => Some(For),
            "fun" => Some(Fun),
            "if" => Some(If),
            "nil" => Some(Nil),
            "or" => Some(Or),
            "print" => Some(Print),
            "return" => Some(Return),
            "super" => Some(Super),
            "this" => Some(This),
            "true" => Some(True),
            "var" => Some(Var),
            "while" => Some(While),
            _ => None,
        }
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        // Look for a fractional part.
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        // TODO: get rid of unwraps if possible.
        let literal: f64 = self.source[self.start..self.current].parse().unwrap();
        self.add_token_with_literal(TokenType::Number, Some(Box::new(literal)));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.lox.error(self.line, "Unterminated string.");
            return;
        }
        // The closing ".
        self.advance();
        // Trim the surrounding quotes.
        let value = self.source[(self.start + 1)..(self.current - 1)].to_string();
        self.add_token_with_literal(TokenType::String, Some(Box::new(value)));
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.chars().nth(self.current).unwrap() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // TODO: maybe return Option<char> instead?
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_digit(10)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    // TODO: Use enum variants in Token instead of creating two versions of this function.
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Object>) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }
}

type Object = Box<dyn LoxObject>;

trait LoxObject: fmt::Debug {}

impl LoxObject for f64 {}
impl LoxObject for String {}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Object>,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

// TODO: merge this enum into Token.
#[rustfmt::skip]
#[derive(Debug)]
enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Eof,
}
