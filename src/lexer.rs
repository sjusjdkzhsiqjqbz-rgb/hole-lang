use crate::ast::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("{0}:{1}: illegal character '{2}'")]
    IllegalChar(usize, usize, char),
    #[error("{0}:{1}: unterminated string literal")]
    UnterminatedString(usize, usize),
    #[error("{0}:{1}: unterminated char literal")]
    UnterminatedChar(usize, usize),
    #[error("{0}:{1}: invalid char literal (must be single character)")]
    InvalidCharLiteral(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Module,
    Exposing,
    Type,
    Alias,
    Do,
    If,
    Then,
    Else,
    Match,
    With,
    Let,
    In,
    Return,
    Arrow,
    LeftArrow,
    ColonColon,
    Eq,
    Lambda,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Bar,
    QuestionQuestionQuestion,

    Op(String),

    UpperId(String),
    LowerId(String),

    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    True,
    False,

    Eof,
}

impl Token {
    pub fn span(&self) -> Span {
        Span::new(0, 0)
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '-' && self.peek_n(1) == Some('-') {
                self.advance();
                self.advance();
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        self.advance(); // opening "
        let start_line = self.line;
        let start_col = self.col - 1;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance();
                    return Ok(s);
                }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            s.push('\t');
                            self.advance();
                        }
                        Some('\\') => {
                            s.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            s.push('"');
                            self.advance();
                        }
                        Some(ch) => {
                            s.push(ch);
                            self.advance();
                        }
                        None => break,
                    }
                }
                '\n' => break,
                _ => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        Err(LexError::UnterminatedString(start_line, start_col))
    }

    fn read_char(&mut self) -> Result<char, LexError> {
        self.advance(); // opening '
        let start_line = self.line;
        let start_col = self.col - 1;
        let c = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        self.advance();
                        '\n'
                    }
                    Some('t') => {
                        self.advance();
                        '\t'
                    }
                    Some('\\') => {
                        self.advance();
                        '\\'
                    }
                    Some('\'') => {
                        self.advance();
                        '\''
                    }
                    _ => {
                        return Err(LexError::InvalidCharLiteral(start_line, start_col));
                    }
                }
            }
            Some(ch) if ch != '\'' && ch != '\n' => {
                self.advance();
                ch
            }
            _ => {
                return Err(LexError::UnterminatedChar(start_line, start_col));
            }
        };
        match self.peek() {
            Some('\'') => {
                self.advance();
                Ok(c)
            }
            _ => Err(LexError::UnterminatedChar(start_line, start_col)),
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Int(s.parse().unwrap_or(0))
        }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '\'' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match s.as_str() {
            "module" => Token::Module,
            "exposing" => Token::Exposing,
            "type" => Token::Type,
            "alias" => Token::Alias,
            "do" => Token::Do,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "match" => Token::Match,
            "with" => Token::With,
            "let" => Token::Let,
            "in" => Token::In,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            _ => {
                if s.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Token::UpperId(s)
                } else {
                    Token::LowerId(s)
                }
            }
        }
    }

    fn read_op(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);

        let peek = self.peek();

        match first {
            '?' if peek == Some('?') => {
                while self.peek() == Some('?') {
                    self.advance();
                    s.push('?');
                }
                if s.len() >= 3 {
                    return Token::QuestionQuestionQuestion;
                }
            }
            ':' if peek == Some(':') => {
                self.advance();
                return Token::ColonColon;
            }
            '<' if peek == Some('-') => {
                self.advance();
                s.push('=');
                return Token::Op("!=".into());
            }
            '<' if peek == Some('-') => {
                self.advance();
                return Token::LeftArrow;
            }
            '<' if peek == Some('=') => {
                self.advance();
                s.push('=');
                return Token::Op("<=".into());
            }
            '>' if peek == Some('=') => {
                self.advance();
                s.push('=');
                return Token::Op(">=".into());
            }
            '&' if peek == Some('&') => {
                self.advance();
                s.push('&');
                return Token::Op("&&".into());
            }
            '|' if peek == Some('|') => {
                self.advance();
                s.push('|');
                return Token::Op("||".into());
            }
            '+' if peek == Some('+') => {
                self.advance();
                s.push('+');
                return Token::Op("++".into());
            }
            '-' if peek == Some('>') => {
                self.advance();
                return Token::Arrow;
            }
            _ => {}
        }
        Token::Op(s)
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, Span)>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let span = self.span();
            match self.peek() {
                None => break,
                Some('"') => {
                    let s = self.read_string()?;
                    tokens.push((Token::String(s), span));
                }
                Some('\'') => {
                    let c = self.read_char()?;
                    tokens.push((Token::Char(c), span));
                }
                Some(c) if c.is_ascii_digit() => {
                    self.advance();
                    let tok = self.read_number(c);
                    tokens.push((tok, span));
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    self.advance();
                    let tok = self.read_ident(c);
                    tokens.push((tok, span));
                }
                Some('=') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push((Token::Op("==".into()), span));
                    } else {
                        tokens.push((Token::Eq, span));
                    }
                }
                Some('|') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push((Token::Op("|>".into()), span));
                    } else if self.peek() == Some('|') {
                        self.advance();
                        tokens.push((Token::Op("||".into()), span));
                    } else {
                        tokens.push((Token::Bar, span));
                    }
                }
                Some(c) if is_op_char(c) => {
                    self.advance();
                    let tok = self.read_op(c);
                    match &tok {
                        Token::Op(s) if s == "?" => {
                            return Err(LexError::IllegalChar(
                                span.line,
                                span.col,
                                '?',
                            ));
                        }
                        _ => tokens.push((tok, span)),
                    }
                }
                Some('(') => {
                    self.advance();
                    tokens.push((Token::LParen, span));
                }
                Some(')') => {
                    self.advance();
                    tokens.push((Token::RParen, span));
                }
                Some('[') => {
                    self.advance();
                    tokens.push((Token::LBracket, span));
                }
                Some(']') => {
                    self.advance();
                    tokens.push((Token::RBracket, span));
                }
                Some(',') => {
                    self.advance();
                    tokens.push((Token::Comma, span));
                }
                Some('\\') => {
                    self.advance();
                    tokens.push((Token::Lambda, span));
                }
                Some(c) => {
                    return Err(LexError::IllegalChar(span.line, span.col, c));
                }
            }
        }
        tokens.push((Token::Eof, Span::new(self.line, self.col)));
        Ok(tokens)
    }
}

fn is_op_char(c: char) -> bool {
    matches!(
        c,
        '+' | '-'
            | '*'
            | '/'
            | '%'
            | '<'
            | '>'
            | '!'
            | '&'
            | '?'
            | ':'
    )
}
