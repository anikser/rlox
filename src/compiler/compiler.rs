use crate::{
    common::Chunk,
    common::{OpCode, Value},
    compiler::scanner::Scanner,
    vm::InterpretError,
};
use std::str::FromStr;
use std::{cell::RefCell, io::Write};

use super::{Token, TokenType};

struct Parser<'a> {
    scanner: &'a mut Scanner,
    // FIXME: can we avoid doing this?
    chunk: &'a RefCell<Chunk>,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
}
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

#[repr(C)]
struct ParseRule {
    pub prefix: Option<for<'a> fn(&'a mut Parser<'a>)>,
    pub infix: Option<for<'a> fn(&'a mut Parser<'_>)>,
    pub precedence: Precedence,
}

impl ParseRule {}

impl<'a> Parser<'a> {
    pub fn init(scanner: &'a mut Scanner, chunk: &'a RefCell<Chunk>) -> Self {
        Self {
            scanner: scanner,
            chunk: chunk,
            current: Token {
                token_type: TokenType::EOF,
                source: "",
                line: u32::MAX,
            },
            previous: Token {
                token_type: TokenType::EOF,
                source: "",
                line: u32::MAX,
            },
            had_error: false,
            panic_mode: false,
        }
    }
    pub fn advance(&mut self) {
        self.previous = self.current;
        loop {
            match self.scanner.scan_token().token_type {
                TokenType::Error => self.error_at_current(self.current.source),
                _ => break,
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.current;
        self.error_at(&token, message);
    }

    fn error(&mut self, message: &str) {
        let token = self.previous;
        self.error_at(&token, message);
    }

    // FIXME: pass error details up in result type
    fn error_at(&mut self, token: &Token<'_>, message: &str) {
        if self.panic_mode {
            return;
        };
        self.panic_mode = true;
        let mut stderr = std::io::stderr();
        write!(stderr, "[line {}] Error", token.line).unwrap();
        match token.token_type {
            TokenType::EOF => write!(stderr, " at end").unwrap(),
            TokenType::Error => (),
            _ => write!(stderr, " at {}", token.source).unwrap(),
        }
        writeln!(stderr, ": {}.", message).unwrap();
        self.had_error = true;
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance()
        } else {
            self.error_at_current(message)
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn number(&mut self) {
        // let number = f64::From(self.previous.source);
        match f64::from_str(self.previous.source) {
            Ok(number) => self.emit_constant(Value(number)),
            // TODO: use InterprerError type?
            Err(e) => self.error("Failed to parse number."),
        }
    }

    fn unary(&mut self) {
        let op_type = self.previous.token_type;
        self.parse_precedence(Precedence::Unary);
        match op_type {
            TokenType::Minus => {
                let line = self.previous.line;
                self.current_chunk()
                    .borrow_mut()
                    .add_code_op(OpCode::Negate, line)
            }
            _ => panic!("Unexpected token type for unary operator."),
        }
    }

    fn binary(&mut self) {
        let op_type = self.previous.token_type;
        let parse_rule = self.get_rule(Self::binary);
        self.parse_precedence(parse_rule.precedence.next());

        let line = self.previous.line;
        match op_type {
            TokenType::Plus => self
                .current_chunk()
                .borrow_mut()
                .add_code_op(OpCode::Add, line),
            TokenType::Minus => self
                .current_chunk()
                .borrow_mut()
                .add_code_op(OpCode::Subtract, line),
            TokenType::Star => self
                .current_chunk()
                .borrow_mut()
                .add_code_op(OpCode::Multiply, line),
            TokenType::Slash => self
                .current_chunk()
                .borrow_mut()
                .add_code_op(OpCode::Divide, line),
            _ => panic!("Unexpected token type for binary operator."),
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let chunk = self.current_chunk();
        let line = self.previous.line;
        let constant_idx = self.current_chunk().borrow_mut().add_constant(value);
        if constant_idx.0 > 16777216 {
            // FIXME: error message
            self.error("Too many constants in one chunk.");
        } else if constant_idx.0 > 255 {
            self.current_chunk()
                .borrow_mut()
                .add_code_constant_long(constant_idx, line)
        } else {
            self.current_chunk()
                .borrow_mut()
                .add_code_constant(constant_idx, line)
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {}

    fn current_chunk(&self) -> &RefCell<Chunk> {
        self.chunk
    }

    fn get_rule(&self, op_type: TokenType) -> ParseRule {
        match op_type {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Self::grouping),
                infix: todo!(),
                precedence: todo!(),
            },
            TokenType::RightParen => todo!(),
            TokenType::LeftBrace => todo!(),
            TokenType::RightBrace => todo!(),
            TokenType::Comma => todo!(),
            TokenType::Dot => todo!(),
            TokenType::Minus => todo!(),
            TokenType::Plus => todo!(),
            TokenType::Semicolon => todo!(),
            TokenType::Slash => todo!(),
            TokenType::Star => todo!(),
            TokenType::Bang => todo!(),
            TokenType::BangEqual => todo!(),
            TokenType::Equal => todo!(),
            TokenType::EqualEqual => todo!(),
            TokenType::Greater => todo!(),
            TokenType::GreaterEqual => todo!(),
            TokenType::Less => todo!(),
            TokenType::LessEqual => todo!(),
            TokenType::Identifier => todo!(),
            TokenType::String => todo!(),
            TokenType::Number => todo!(),
            TokenType::And => todo!(),
            TokenType::Class => todo!(),
            TokenType::Else => todo!(),
            TokenType::False => todo!(),
            TokenType::For => todo!(),
            TokenType::Fun => todo!(),
            TokenType::If => todo!(),
            TokenType::Nil => todo!(),
            TokenType::Or => todo!(),
            TokenType::Print => todo!(),
            TokenType::Return => todo!(),
            TokenType::Super => todo!(),
            TokenType::This => todo!(),
            TokenType::True => todo!(),
            TokenType::Var => todo!(),
            TokenType::While => todo!(),
            TokenType::Error => todo!(),
            TokenType::EOF => todo!(),
        }
    }
}

pub fn compile(source: String, chunk: &RefCell<Chunk>) -> Result<(), InterpretError> {
    let mut scanner = Scanner::init(source);
    let mut parser = Parser::init(&mut scanner, chunk);
    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expect end of expression.");

    match parser.had_error {
        true => Err(InterpretError::CompileError),
        false => Ok(()),
    }
}
