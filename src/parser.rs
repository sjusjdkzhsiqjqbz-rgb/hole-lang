use crate::ast::*;
use crate::lexer::{Token, LexError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{span}: expected {expected}, got {got:?}")]
    UnexpectedToken {
        span: Span,
        expected: String,
        got: String,
    },
    #[error("{span}: {msg}")]
    Parse { span: Span, msg: String },
    #[error("lexer error: {0}")]
    Lex(#[from] LexError),
}

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Span)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens.get(self.pos).map(|t| &t.0).unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|t| t.1).unwrap_or(Span::new(1, 1))
    }

    fn advance(&mut self) -> (Token, Span) {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &str) -> Result<Span, ParseError> {
        let (tok, span) = self.advance();
        let ok = match (&tok, expected) {
            (Token::Module, "module") => true,
            (Token::Exposing, "exposing") => true,
            (Token::Type, "type") => true,
            (Token::Alias, "alias") => true,
            (Token::Do, "do") => true,
            (Token::If, "if") => true,
            (Token::Then, "then") => true,
            (Token::Else, "else") => true,
            (Token::Match, "match") => true,
            (Token::With, "with") => true,
            (Token::Let, "let") => true,
            (Token::In, "in") => true,
            (Token::Return, "return") => true,
            (Token::Arrow, "->") => true,
            (Token::ColonColon, "::") => true,
            (Token::Eq, "=") => true,
            (Token::Lambda, "\\") => true,
            (Token::LParen, "(") => true,
            (Token::RParen, ")") => true,
            (Token::LBracket, "[") => true,
            (Token::RBracket, "]") => true,
            (Token::Comma, ",") => true,
            (Token::Bar, "|") => true,
            (Token::Eof, "EOF") => true,
            (Token::QuestionQuestionQuestion, "???") => true,
            (Token::True, "true") => true,
            (Token::False, "false") => true,
            _ => false,
        };
        if ok {
            Ok(span)
        } else {
            Err(ParseError::UnexpectedToken {
                span,
                expected: expected.into(),
                got: format!("{:?}", tok),
            })
        }
    }

    fn check(&self, expected: &str) -> bool {
        match (self.peek(), expected) {
            (Token::Module, "module") => true,
            (Token::Exposing, "exposing") => true,
            (Token::Type, "type") => true,
            (Token::Alias, "alias") => true,
            (Token::Do, "do") => true,
            (Token::If, "if") => true,
            (Token::Then, "then") => true,
            (Token::Else, "else") => true,
            (Token::Match, "match") => true,
            (Token::With, "with") => true,
            (Token::Let, "let") => true,
            (Token::In, "in") => true,
            (Token::Return, "return") => true,
            (Token::Arrow, "->") => true,
            (Token::ColonColon, "::") => true,
            (Token::Eq, "=") => true,
            (Token::Lambda, "\\") => true,
            (Token::LParen, "(") => true,
            (Token::RParen, ")") => true,
            (Token::LBracket, "[") => true,
            (Token::RBracket, "]") => true,
            (Token::Comma, ",") => true,
            (Token::Bar, "|") => true,
            (Token::Eof, "EOF") => true,
            (Token::Op(_), _) => false,
            (Token::UpperId(_), "upper") => true,
            (Token::LowerId(_), "lower") => true,
            (Token::Int(_), "int") => true,
            (Token::Float(_), "float") => true,
            (Token::String(_), "string") => true,
            (Token::Char(_), "char") => true,
            (Token::True, "bool") => true,
            (Token::False, "bool") => true,
            (Token::QuestionQuestionQuestion, "???") => true,
            _ => false,
        }
    }

    fn is_atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::LParen
                | Token::LBracket
                | Token::Lambda
                | Token::LowerId(_)
                | Token::UpperId(_)
                | Token::Int(_)
                | Token::Float(_)
                | Token::String(_)
                | Token::Char(_)
                | Token::True
                | Token::False
                | Token::QuestionQuestionQuestion
        )
    }

    fn is_operator(&self) -> bool {
        matches!(self.peek(), Token::Op(_))
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let module = self.parse_module()?;
        Ok(Program { module })
    }

    fn parse_module(&mut self) -> Result<Module, ParseError> {
        let name = if self.check("module") {
            self.expect("module")?;
            let (tok, _) = self.advance();
            match tok {
                Token::UpperId(n) => {
                    if self.check("exposing") {
                        self.expect("exposing")?;
                        self.expect("(")?;
                        let mut _exports = vec![];
                        loop {
                            if self.check(")") {
                                break;
                            }
                            let (id_tok, _) = self.advance();
                            match id_tok {
                                Token::LowerId(id) | Token::UpperId(id) => _exports.push(id),
                                _ => return Err(ParseError::UnexpectedToken {
                                    span: self.peek_span(),
                                    expected: "identifier".into(),
                                    got: format!("{:?}", id_tok),
                                }),
                            }
                            if !self.check(",") {
                                break;
                            }
                            self.expect(",")?;
                        }
                        self.expect(")")?;
                    }
                    n
                }
                _ => return Err(ParseError::UnexpectedToken {
                    span: self.peek_span(),
                    expected: "module name".into(),
                    got: format!("{:?}", tok),
                }),
            }
        } else {
            "Main".into()
        };

        let mut declarations = vec![];

        while !matches!(self.peek(), Token::Eof) {
            declarations.push(self.parse_top_decl()?);
        }

        Ok(Module { name, declarations })
    }

    fn parse_top_decl(&mut self) -> Result<TopDecl, ParseError> {
        let span = self.peek_span();
        match self.peek() {
            Token::Type => {
                self.advance();
                if self.check("alias") {
                    self.advance();
                    self.parse_type_alias(span)
                } else {
                    self.parse_type_def(span)
                }
            }
            Token::LowerId(_) => {
                let (id_tok, _) = self.advance();
                let name = match id_tok {
                    Token::LowerId(n) => n,
                    _ => unreachable!(),
                };

                let mut params = vec![];
                while self.is_atom_start() && !self.check("::") && !self.check("=") {
                    match self.peek() {
                        Token::LowerId(_) => {
                            let (p_tok, _) = self.advance();
                            match p_tok {
                                Token::LowerId(p) => params.push(p),
                                _ => unreachable!(),
                            }
                        }
                        _ => break,
                    }
                }

                let type_ann = if self.check("::") {
                    self.expect("::")?;
                    self.parse_type()?
                } else {
                    return Err(ParseError::Parse {
                        span: self.peek_span(),
                        msg: "expected :: type annotation".into(),
                    });
                };
                let body = if self.check("=") {
                    self.expect("=")?;
                    self.parse_expr()?
                } else {
                    return Err(ParseError::Parse {
                        span: self.peek_span(),
                        msg: "expected = after function parameters".into(),
                    });
                };

                Ok(TopDecl::Fun(FunDef { name, type_ann, params, body }))
            }
            _ => Err(ParseError::UnexpectedToken {
                span,
                expected: "type or function declaration".into(),
                got: format!("{:?}", self.peek()),
            }),
        }
    }

    fn parse_type_def(&mut self, start_span: Span) -> Result<TopDecl, ParseError> {
        let (name_tok, _) = self.advance();
        let name = match name_tok {
            Token::UpperId(n) => n,
            _ => return Err(ParseError::UnexpectedToken {
                span: self.peek_span(),
                expected: "type name (uppercase)".into(),
                got: format!("{:?}", name_tok),
            }),
        };

        let mut params = vec![];
        while self.check("lower") {
            let (p_tok, _) = self.advance();
            match p_tok {
                Token::LowerId(p) => params.push(p),
                _ => unreachable!(),
            }
        }

        self.expect("=")?;

        let mut variants = vec![];
        loop {
            let (ctor_tok, ctor_span) = self.advance();
            let ctor_name = match ctor_tok {
                Token::UpperId(n) => n,
                _ => return Err(ParseError::UnexpectedToken {
                    span: self.peek_span(),
                    expected: "constructor name".into(),
                    got: format!("{:?}", ctor_tok),
                }),
            };

            let mut args = vec![];
            while !self.check("|")
                && (self.check("upper")
                    || self.check("lower")
                    || self.check("(")
                    || self.check("string")
                    || self.check("int"))
            {
                let next_span = self.peek_span();
                if next_span.line != ctor_span.line
                    && matches!(self.peek(), Token::LowerId(_))
                {
                    break;
                }
                args.push(self.parse_type()?);
            }

            variants.push((ctor_name, args));

            if !self.check("|") {
                break;
            }
            self.expect("|")?;
        }

        Ok(TopDecl::Type(ADTDef { name, params, variants }))
    }

    fn parse_type_alias(&mut self, _start_span: Span) -> Result<TopDecl, ParseError> {
        let (name_tok, _) = self.advance();
        let name = match name_tok {
            Token::UpperId(n) => n,
            _ => return Err(ParseError::UnexpectedToken {
                span: self.peek_span(),
                expected: "type alias name".into(),
                got: format!("{:?}", name_tok),
            }),
        };

        let mut params = vec![];
        while self.check("lower") {
            let (p_tok, _) = self.advance();
            match p_tok {
                Token::LowerId(p) => params.push(p),
                _ => unreachable!(),
            }
        }

        self.expect("=")?;
        let ty = self.parse_type()?;

        Ok(TopDecl::Alias(AliasDef { name, params, ty }))
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let left = self.parse_type_app()?;
        if matches!(self.peek(), Token::Arrow) {
            self.advance();
            let right = self.parse_type()?;
            Ok(Type::TFun {
                from: Box::new(left),
                to: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_type_app(&mut self) -> Result<Type, ParseError> {
        match self.peek() {
            Token::UpperId(_) => {
                let (tok, span) = self.advance();
                let name = match tok {
                    Token::UpperId(n) => n,
                    _ => unreachable!(),
                };
                let mut args = vec![];
                loop {
                    let next_span = self.peek_span();
                    if next_span.line != span.line
                        && matches!(self.peek(), Token::LowerId(_))
                    {
                        break;
                    }
                    match self.peek() {
                        Token::UpperId(_) | Token::LowerId(_) => {
                            args.push(self.parse_type_app()?);
                        }
                        Token::LParen => {
                            args.push(self.parse_type_app()?);
                        }
                        _ => break,
                    }
                }
                Ok(Type::TCon { name, args })
            }
            Token::LowerId(_) => {
                let (tok, _) = self.advance();
                let name = match tok {
                    Token::LowerId(n) => n,
                    _ => unreachable!(),
                };
                Ok(Type::TVar { name, id: 0 })
            }
            Token::LParen => {
                self.advance();
                if self.check(")") {
                    self.advance();
                    return Ok(Type::unit());
                }
                let first = self.parse_type()?;
                if self.check(",") {
                    let mut tup = vec![first];
                    while self.check(",") {
                        self.advance();
                        tup.push(self.parse_type()?);
                    }
                    self.expect(")")?;
                    Ok(Type::TTuple { types: tup })
                } else {
                    self.expect(")")?;
                    Ok(first)
                }
            }
            _ => Err(ParseError::Parse {
                span: self.peek_span(),
                msg: "expected type".into(),
            }),
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_binop(0)?;
        while let Token::Op(s) = self.peek() {
            if s != "|>" { break; }
            self.advance();
            let span = self.peek_span();
            let right = self.parse_pipe()?;
            left = Expr::App {
                func: Box::new(right),
                args: vec![left],
                span,
            };
        }
        Ok(left)
    }

    fn parse_binop(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Token::Op(s) => BinOp::from_str(s),
                _ => None,
            };

            match op {
                Some(op) if op.precedence() >= min_prec && op != BinOp::Pipe => {
                    self.advance();
                    let right = self.parse_binop(op.precedence() + 1)?;
                    let span = Span::new(
                        left.span().line,
                        left.span().col,
                    );
                    left = Expr::BinOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                        span,
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek_span();
        match self.peek() {
            Token::Op(s) if s == "-" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr), span })
            }
            Token::Op(s) if s == "!" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr), span })
            }
            _ => self.parse_app(),
        }
    }

    fn parse_app(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_atom()?;
        let current_line = expr.span().line;
        while self.is_atom_start() {
            let next_span = self.peek_span();
            if next_span.line != current_line {
                break;
            }
            let arg = self.parse_atom()?;
            let span = expr.span();
            expr = Expr::App {
                func: Box::new(expr),
                args: vec![arg],
                span,
            };
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek_span();
        match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Int(n), span })
            }
            Token::Float(n) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Float(n), span })
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::String(s), span })
            }
            Token::Char(c) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Char(c), span })
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Bool(true), span })
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Bool(false), span })
            }
            Token::QuestionQuestionQuestion => {
                self.advance();
                Ok(Expr::Hole { span })
            }
            Token::LowerId(name) => {
                self.advance();
                Ok(Expr::Var { name, span })
            }
            Token::UpperId(name) => {
                self.advance();
                Ok(Expr::Var { name, span })
            }
            Token::LParen => {
                self.advance();
                if self.check(")") {
                    self.advance();
                    return Ok(Expr::Literal { value: Literal::Unit, span });
                }
                let first = self.parse_expr()?;
                if self.check(",") {
                    let mut exprs = vec![first];
                    while self.check(",") {
                        self.advance();
                        exprs.push(self.parse_expr()?);
                    }
                    self.expect(")")?;
                    Ok(Expr::Tuple { exprs, span })
                } else {
                    self.expect(")")?;
                    Ok(first)
                }
            }
            Token::LBracket => {
                self.advance();
                let mut exprs = vec![];
                if !self.check("]") {
                    loop {
                        exprs.push(self.parse_expr()?);
                        if !self.check(",") {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect("]")?;
                Ok(Expr::List { exprs, span })
            }
            Token::Lambda => {
                self.advance();
                let params = self.parse_params()?;
                self.expect("->")?;
                let body = self.parse_expr()?;
                Ok(Expr::Lambda { params, body: Box::new(body), span })
            }
            Token::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect("then")?;
                let then_b = self.parse_expr()?;
                self.expect("else")?;
                let else_b = self.parse_expr()?;
                Ok(Expr::If {
                    cond: Box::new(cond),
                    then_b: Box::new(then_b),
                    else_b: Box::new(else_b),
                    span,
                })
            }
            Token::Match => {
                self.advance();
                let match_span = span;
                let expr = self.parse_expr()?;
                self.expect("with")?;
                let mut arms = vec![];
                loop {
                    let next_span = self.peek_span();
                    if next_span.line != match_span.line
                        && next_span.col <= match_span.col
                    {
                        break;
                    }
                    if self.check("|") {
                        self.advance();
                    }
                    let pat = self.parse_pattern()?;
                    self.expect("->")?;
                    let body = self.parse_expr()?;
                    arms.push((pat, body));
                    if !self.check("|") && !self.is_atom_start() && !matches!(self.peek(), Token::LowerId(_)) {
                        break;
                    }
                    if matches!(self.peek(), Token::RParen | Token::RBracket | Token::Eof)
                        || self.check("in") || self.check("then") || self.check("else")
                        || self.check("with") || self.check("=")
                    {
                        break;
                    }
                }
                Ok(Expr::Match { expr: Box::new(expr), arms, span })
            }
            Token::Do => {
                self.advance();
                let mut stmts = vec![];
                while self.is_atom_start()
                    || matches!(self.peek(), Token::LowerId(_))
                    || matches!(self.peek(), Token::If)
                    || matches!(self.peek(), Token::Match)
                    || matches!(self.peek(), Token::Do)
                    || matches!(self.peek(), Token::Let)
                {
                    if self.check("let") {
                        return Err(ParseError::Parse {
                            span: self.peek_span(),
                            msg: "let-in inside do block not yet supported".into(),
                        });
                    } else if self.check("lower") {
                        let save_pos = self.pos;
                        let (tok, _) = self.advance();
                        let name = match tok {
                            Token::LowerId(n) => n,
                            _ => unreachable!(),
                        };
                        if matches!(self.peek(), Token::LeftArrow) {
                            self.advance();
                            let expr = self.parse_expr()?;
                            stmts.push(Stmt::Bind { name, expr });
                        } else {
                            self.pos = save_pos;
                            let expr = self.parse_expr()?;
                            stmts.push(Stmt::Expr(expr));
                        }
                    } else {
                        let expr = self.parse_expr()?;
                        stmts.push(Stmt::Expr(expr));
                    }

                    if matches!(self.peek(), Token::RParen | Token::RBracket | Token::Eof)
                        || self.check("in") || self.check("then") || self.check("else")
                    {
                        break;
                    }
                }
                Ok(Expr::Do { stmts, span })
            }
            Token::Let => {
                self.advance();
                let mut bindings = vec![];
                loop {
                    let (name_tok, _) = self.advance();
                    let name = match name_tok {
                        Token::LowerId(n) => n,
                        _ => return Err(ParseError::UnexpectedToken {
                            span: self.peek_span(),
                            expected: "binding name".into(),
                            got: format!("{:?}", name_tok),
                        }),
                    };

                    let mut params = vec![];
                    while self.check("lower") {
                        let (p_tok, _) = self.advance();
                        match p_tok {
                            Token::LowerId(p) => params.push(p),
                            _ => unreachable!(),
                        }
                    }

                    self.expect("=")?;
                    let body = self.parse_expr()?;
                    bindings.push(Binding { name, params, body });

                    if !self.check("lower") {
                        break;
                    }
                }
                self.expect("in")?;
                let body = self.parse_expr()?;
                Ok(Expr::Let { bindings, body: Box::new(body), span })
            }
            Token::Return => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Return { expr: Box::new(expr), span })
            }
            t => Err(ParseError::UnexpectedToken {
                span,
                expected: "expression".into(),
                got: format!("{:?}", t),
            }),
        }
    }

    fn parse_params(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut params = vec![];
        loop {
            match self.peek() {
                Token::LowerId(_) => {
                    let (tok, span) = self.advance();
                    match tok {
                        Token::LowerId(n) => params.push(Pattern::PVar { name: n, span }),
                        _ => unreachable!(),
                    }
                }
                Token::LParen => {
                    self.advance();
                    let mut pats = vec![];
                    loop {
                        if self.check(")") {
                            break;
                        }
                        pats.push(self.parse_pattern()?);
                        if !self.check(",") {
                            break;
                        }
                        self.advance();
                    }
                    self.expect(")")?;
                    if pats.len() == 1 {
                        params.push(pats.into_iter().next().unwrap());
                    } else {
                        params.push(Pattern::PTuple {
                            pats,
                            span: self.peek_span(),
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(params)
    }

    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let span = self.peek_span();
        match self.peek() {
            Token::LowerId(_) => {
                let (tok, span) = self.advance();
                match tok {
                    Token::LowerId(ref n) if n == "_" => Ok(Pattern::Wildcard { span }),
                    Token::LowerId(n) => Ok(Pattern::PVar { name: n, span }),
                    _ => unreachable!(),
                }
            }
            Token::UpperId(_) => {
                let (tok, span) = self.advance();
                let name = match tok {
                    Token::UpperId(n) => n,
                    _ => unreachable!(),
                };
                let mut args = vec![];
                while self.is_pattern_start() {
                    args.push(self.parse_pattern()?);
                }
                Ok(Pattern::PCtor { name, args, span })
            }
            Token::Int(_) | Token::String(_) | Token::Char(_) | Token::True | Token::False => {
                let (tok, span) = self.advance();
                match tok {
                    Token::Int(n) => Ok(Pattern::PLit { value: Literal::Int(n), span }),
                    Token::String(s) => Ok(Pattern::PLit { value: Literal::String(s), span }),
                    Token::Char(c) => Ok(Pattern::PLit { value: Literal::Char(c), span }),
                    Token::True => Ok(Pattern::PLit { value: Literal::Bool(true), span }),
                    Token::False => Ok(Pattern::PLit { value: Literal::Bool(false), span }),
                    _ => unreachable!(),
                }
            }
            Token::LParen => {
                self.advance();
                if self.check(")") {
                    self.advance();
                    return Ok(Pattern::PLit { value: Literal::Unit, span });
                }
                let mut pats = vec![];
                loop {
                    pats.push(self.parse_pattern()?);
                    if !self.check(",") {
                        break;
                    }
                    self.advance();
                }
                self.expect(")")?;
                Ok(Pattern::PTuple { pats, span })
            }
            Token::LBracket => {
                self.advance();
                if self.check("]") {
                    self.advance();
                    return Ok(Pattern::PNil { span });
                }
                let head = self.parse_pattern()?;
                if self.check("|") {
                    // [x | xs] — cons pattern
                    return Err(ParseError::Parse {
                        span: self.peek_span(),
                        msg: "list cons pattern [x|xs] not yet supported".into(),
                    });
                }
                Err(ParseError::Parse {
                    span: self.peek_span(),
                    msg: "list literal pattern not supported".into(),
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                span,
                expected: "pattern".into(),
                got: format!("{:?}", self.peek()),
            }),
        }
    }

    fn is_pattern_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::LowerId(_)
                | Token::UpperId(_)
                | Token::Int(_)
                | Token::String(_)
                | Token::Char(_)
                | Token::True
                | Token::False
                | Token::LParen
                | Token::LBracket
        )
    }
}
