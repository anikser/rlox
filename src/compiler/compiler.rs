use crate::{common::chunk::Chunk, compiler::scanner::Scanner, vm::InterpretError};
use std::io::Write;

use super::{Token, TokenType};

struct Parser<'a> {
    scanner: &'a mut Scanner,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
}
impl<'a> Parser<'a> {
    pub fn init(scanner: &'a mut Scanner) -> Self {
        Self {
            scanner: scanner,
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
}

pub fn compile(source: String, chunk: &mut Chunk) -> Result<(), InterpretError> {
    let mut scanner = Scanner::init(source);
    let mut parser = Parser::init(&mut scanner);
    parser.advance();
    parser.consume(TokenType::EOF, "Expect end of expression.");

    match parser.had_error {
        true => Err(InterpretError::CompileError),
        false => Ok(()),
    }
}
