use crate::{
    common::Chunk,
    common::{ObjectRef, OpCode, Value, ObjectHeap},
    compiler::scanner::Scanner,
    vm::InterpretError,
};
use std::{cell::RefCell, io::Write};
use std::{rc::Rc, str::FromStr};

use super::{Token, TokenType};

struct Parser<'a> {
    // FIXME: can we avoid doing this?
    scanner: Rc<RefCell<Scanner>>,
    chunk: Rc<RefCell<Chunk>>,
    heap: &'a ObjectHeap,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

#[derive(PartialEq, PartialOrd)]
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

struct ParseRule {
    pub prefix: Option<fn(&mut Parser<'_>)>,
    pub infix: Option<fn(&mut Parser<'_>)>,
    pub precedence: Precedence,
}

// impl<'a, '> ParseRule<'parser> {}

impl<'a> Parser<'a> {
    pub fn init(scanner: Scanner, chunk: Rc<RefCell<Chunk>>, heap: &'a ObjectHeap) -> Self {
        Self {
            scanner: Rc::new(RefCell::new(scanner)),
            chunk,
            heap,
            current: Token {
                token_type: TokenType::EOF,
                source: "".to_owned(),
                line: u32::MAX,
            },
            previous: Token {
                token_type: TokenType::EOF,
                source: "".to_owned(),
                line: u32::MAX,
            },
            had_error: false,
            panic_mode: false,
        }
    }
    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        println!("{:?}", self.previous);
        loop {
            // let token = self.scanner.scan_token();
            // let token = self.scanner.borrow_mut().scan_token();
            let scanner_ref = self.scanner.clone();
            let mut scanner = RefCell::borrow_mut(&scanner_ref);

            let token = scanner.scan_token();
            match token.token_type {
                TokenType::Error => {
                    let lexeme = self.current.source.clone();
                    self.error_at_current(&lexeme);
                }
                _ => {
                    self.current = token;
                    break;
                }
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message);
    }

    // FIXME: pass error details up in result type
    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        };
        self.panic_mode = true;
        let mut stderr = std::io::stderr();
        let token = token;
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
        println!("NUMBERRRR!");
        println!("{}", self.previous.source);
        match f64::from_str(self.previous.source.as_str()) {
            Ok(number) => self.emit_constant(Value::Double(number)),
            // TODO: use InterprerError type?
            Err(_) => self.error("Failed to parse number."),
        }
    }

    fn unary(&mut self) {
        let prev_token = self.previous.clone();
        let op_type = prev_token.token_type;
        self.parse_precedence(Precedence::Unary);
        let chunk_ref = self.current_chunk();
        let mut chunk = RefCell::borrow_mut(&chunk_ref);
        match op_type {
            TokenType::Minus => {
                let line = prev_token.line;
                chunk.add_code_op(OpCode::Negate, line)
            }
            TokenType::Bang => {
                let line = prev_token.line;
                chunk.add_code_op(OpCode::Not, line);
            }
            _ => panic!("Unexpected token type for unary operator."),
        }
    }

    fn binary(&mut self) {
        let prev_token = self.previous.clone();
        let op_type = prev_token.token_type;
        let parse_rule = self.get_rule(op_type);
        self.parse_precedence(parse_rule.precedence.next());

        let chunk_ref = self.current_chunk();
        let mut chunk = RefCell::borrow_mut(&chunk_ref);
        let line = prev_token.line;
        match op_type {
            TokenType::Plus => chunk.add_code_op(OpCode::Add, line),
            TokenType::Minus => chunk.add_code_op(OpCode::Subtract, line),
            TokenType::Star => chunk.add_code_op(OpCode::Multiply, line),
            TokenType::Slash => chunk.add_code_op(OpCode::Divide, line),
            TokenType::EqualEqual => chunk.add_code_op(OpCode::Equal, line),
            TokenType::BangEqual => {
                chunk.add_code_op(OpCode::Equal, line);
                chunk.add_code_op(OpCode::Not, line)
            }
            TokenType::Less => chunk.add_code_op(OpCode::Less, line),
            TokenType::LessEqual => {
                chunk.add_code_op(OpCode::Greater, line);
                chunk.add_code_op(OpCode::Not, line)
            }
            TokenType::Greater => chunk.add_code_op(OpCode::Greater, line),
            TokenType::GreaterEqual => {
                chunk.add_code_op(OpCode::Less, line);
                chunk.add_code_op(OpCode::Not, line)
            }
            _ => panic!("Unexpected token type for binary operator."),
        }
    }

    fn literal(&mut self) {
        let token = &self.previous;
        let chunk_ref = self.current_chunk();
        let mut chunk = RefCell::borrow_mut(&chunk_ref);
        let line = token.line;
        match token.token_type {
            TokenType::True => chunk.add_code_op(OpCode::True, line),
            TokenType::False => chunk.add_code_op(OpCode::False, line),
            TokenType::Nil => chunk.add_code_op(OpCode::Nil, line),
            _ => panic!("Unexpected token type for literal expression."),
        }
    }

    fn string(&mut self) {
        let token = &self.previous;
        let string_ref = self.heap.alloc_string(&token.source);
        self.emit_constant(Value::Object(string_ref));
    }

    fn emit_constant(&mut self, value: Value) {
        println!("emitting constatns.....");
        let line = self.previous.line;
        let chunk_ref = self.current_chunk();
        println!("got chunk ref..");
        let mut chunk = RefCell::borrow_mut(&chunk_ref);
        println!("borrowed chunk...");

        let constant_idx = chunk.add_constant(value);
        if constant_idx.0 > 16777216 {
            // FIXME: error message
            self.error("Too many constants in one chunk.");
        } else if constant_idx.0 > 255 {
            chunk.add_code_op(OpCode::ConstantLong, line);
            chunk.add_code_constant_long(constant_idx, line);
        } else {
            chunk.add_code_op(OpCode::Constant, line);
            chunk.add_code_constant(constant_idx, line);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let token_type = self.previous.token_type;
        let prefix_rule = self.get_rule(token_type).prefix;
        println!("{:?}", token_type);
        match prefix_rule {
            Some(prefix_rule) => prefix_rule(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }
        while precedence <= self.get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.token_type).infix;
            infix_rule.unwrap()(self);
        }
    }

    // TODO: investigate refcell<chunk>. what is going on here. is this bad? how is this compiled?
    fn current_chunk(&self) -> Rc<RefCell<Chunk>> {
        self.chunk.clone()
    }

    // Vaughan-Pratt precendence rule lookup.
    // TODO: revisit get ParseRule match -- Can we make this more performant? Does the compiler turn this into a LUT?
    fn get_rule(&self, op_type: TokenType) -> ParseRule {
        match op_type {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Self::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Self::unary),
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Self::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: Some(Self::string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Self::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EOF => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

pub fn compile(source: String, chunk: Rc<RefCell<Chunk>>, heap: &ObjectHeap) -> Result<(), InterpretError> {
    let scanner = Scanner::init(source);
    let mut parser = Parser::init(scanner, chunk.clone(), heap);
    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expect end of expression.");
    let chunk_ref = chunk.clone();
    let mut chunk = RefCell::borrow_mut(&chunk_ref);
    chunk.add_code_op(OpCode::Return, 5);

    println!("{}", chunk);

    match parser.had_error {
        true => Err(InterpretError::CompileError),
        false => Ok(()),
    }
}
