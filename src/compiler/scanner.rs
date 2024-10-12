pub struct Scanner {
    source: String,
    current: usize,
    start: usize,
    line: u32,
}
// impl DerefMut for Scanner {
//     type Target = Scanner;
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self
//     }
// }
#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    // FIXME: fix when you're better
    // pub source: &'source str,
    pub source: String,
    pub line: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single character token
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EOF,
}

impl From<&str> for TokenType {
    fn from(value: &str) -> Self {
        match value {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }
}

impl Scanner {
    pub fn init(source: String) -> Self {
        Self {
            source,
            current: 0,
            start: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) {
        let mut line = u32::MAX;
        loop {
            let token = self.scan_token();
            if token.line != line {
                print!("{:03x} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!("{:?} {}", token.token_type, token.source);
            if token.token_type == TokenType::EOF {
                break;
            }
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        match self.current == self.source.len() {
            true => self.make_token(TokenType::EOF),
            false => match self.advance() {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ';' => self.make_token(TokenType::Semicolon),
                '.' => self.make_token(TokenType::Dot),
                ',' => self.make_token(TokenType::Comma),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '/' => self.make_token(TokenType::Slash),
                '*' => self.make_token(TokenType::Star),
                '!' => {
                    let token_type = if self.peek_match('=') {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    };
                    self.make_token(token_type)
                }
                '=' => {
                    let token_type = if self.peek_match('=') {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    };
                    self.make_token(token_type)
                }
                '<' => {
                    let token_type = if self.peek_match('=') {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    };
                    self.make_token(token_type)
                }
                '>' => {
                    let token_type = if self.peek_match('=') {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    };
                    self.make_token(token_type)
                }
                '"' => self.string(),
                c if c.is_ascii_digit() => self.number(),
                c if c.is_alphabetic() || c == '_' => self.identifier(),

                _ => self.error_token("Unexpected character.".to_owned()),
            },
        }
    }

    #[inline(always)]
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.as_bytes()[self.current - 1] as char
    }

    fn peek(&self) -> char {
        if !self.is_at_end() {
            self.source.as_bytes()[self.current] as char
        } else {
            '\0'
        }
    }

    fn peek_next(&self) -> char {
        if self.current < self.source.len() - 1 {
            self.source.as_bytes()[self.current + 1] as char
        } else {
            '\0'
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek_match(&mut self, c: char) -> bool {
        if c == self.peek() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            source: self.source[self.start..self.current].to_string(),
            line: self.line,
        }
    }

    fn make_string_token(&self) -> Token {
        Token {
            token_type: TokenType::String,
            source: self.source[self.start + 1..self.current - 1].to_string(),
            line: self.line,
        }
    }

    fn error_token(&self, message: String) -> Token {
        Token {
            token_type: TokenType::Error,
            source: message,
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                '/' => {
                    if self.peek_match('/') {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                c if c.is_ascii_whitespace() => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error_token("Unterminated string literal.".to_owned())
        } else {
            self.advance();
            self.make_string_token()
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphabetic() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        self.make_token(TokenType::from(text))
    }
}
