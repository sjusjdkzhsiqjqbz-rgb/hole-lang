use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Unit,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => {
                let s = format!("{}", n);
                if s.contains('.') {
                    write!(f, "{}", s)
                } else {
                    write!(f, "{}.0", s)
                }
            }
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Concat,
    And,
    Or,
    Pipe,
}

impl BinOp {
    pub fn from_str(s: &str) -> Option<BinOp> {
        match s {
            "+" => Some(BinOp::Add),
            "-" => Some(BinOp::Sub),
            "*" => Some(BinOp::Mul),
            "/" => Some(BinOp::Div),
            "%" => Some(BinOp::Mod),
            "==" => Some(BinOp::Eq),
            "!=" => Some(BinOp::Neq),
            "<" => Some(BinOp::Lt),
            ">" => Some(BinOp::Gt),
            "<=" => Some(BinOp::Lte),
            ">=" => Some(BinOp::Gte),
            "++" => Some(BinOp::Concat),
            "&&" => Some(BinOp::And),
            "||" => Some(BinOp::Or),
            "|>" => Some(BinOp::Pipe),
            _ => None,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Pipe => 0,
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => 3,
            BinOp::Add | BinOp::Sub | BinOp::Concat => 4,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 5,
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Lte => write!(f, "<="),
            BinOp::Gte => write!(f, ">="),
            BinOp::Concat => write!(f, "++"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Pipe => write!(f, "|>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: Literal,
        span: Span,
    },
    Var {
        name: String,
        span: Span,
    },
    Lambda {
        params: Vec<Pattern>,
        body: Box<Expr>,
        span: Span,
    },
    App {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        then_b: Box<Expr>,
        else_b: Box<Expr>,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<(Pattern, Expr)>,
        span: Span,
    },
    Do {
        stmts: Vec<Stmt>,
        span: Span,
    },
    Let {
        bindings: Vec<Binding>,
        body: Box<Expr>,
        span: Span,
    },
    Hole {
        span: Span,
    },
    Tuple {
        exprs: Vec<Expr>,
        span: Span,
    },
    List {
        exprs: Vec<Expr>,
        span: Span,
    },
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Return {
        expr: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. }
            | Expr::Var { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::App { span, .. }
            | Expr::If { span, .. }
            | Expr::Match { span, .. }
            | Expr::Do { span, .. }
            | Expr::Let { span, .. }
            | Expr::Hole { span, .. }
            | Expr::Tuple { span, .. }
            | Expr::List { span, .. }
            | Expr::BinOp { span, .. }
            | Expr::UnaryOp { span, .. }
            | Expr::Return { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Bind { name: String, expr: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard { span: Span },
    PVar { name: String, span: Span },
    PCtor { name: String, args: Vec<Pattern>, span: Span },
    PLit { value: Literal, span: Span },
    PTuple { pats: Vec<Pattern>, span: Span },
    PNil { span: Span },
    PCons { head: Box<Pattern>, tail: Box<Pattern>, span: Span },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span }
            | Pattern::PVar { span, .. }
            | Pattern::PCtor { span, .. }
            | Pattern::PLit { span, .. }
            | Pattern::PTuple { span, .. }
            | Pattern::PNil { span }
            | Pattern::PCons { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    TVar {
        name: String,
        id: u64,
    },
    TCon {
        name: String,
        args: Vec<Type>,
    },
    TFun {
        from: Box<Type>,
        to: Box<Type>,
    },
    TTuple {
        types: Vec<Type>,
    },
}

impl Type {
    pub fn unit() -> Self {
        Type::TCon {
            name: "Unit".into(),
            args: vec![],
        }
    }

    pub fn int() -> Self {
        Type::TCon {
            name: "Int".into(),
            args: vec![],
        }
    }

    pub fn float() -> Self {
        Type::TCon {
            name: "Float".into(),
            args: vec![],
        }
    }

    pub fn string() -> Self {
        Type::TCon {
            name: "String".into(),
            args: vec![],
        }
    }

    pub fn bool() -> Self {
        Type::TCon {
            name: "Bool".into(),
            args: vec![],
        }
    }

    pub fn char() -> Self {
        Type::TCon {
            name: "Char".into(),
            args: vec![],
        }
    }

    pub fn func(from: Type, to: Type) -> Self {
        Type::TFun {
            from: Box::new(from),
            to: Box::new(to),
        }
    }

    pub fn io(inner: Type) -> Self {
        Type::TCon {
            name: "IO".into(),
            args: vec![inner],
        }
    }

    pub fn list(inner: Type) -> Self {
        Type::TCon {
            name: "List".into(),
            args: vec![inner],
        }
    }

    pub fn option(inner: Type) -> Self {
        Type::TCon {
            name: "Option".into(),
            args: vec![inner],
        }
    }

    pub fn result(err: Type, ok: Type) -> Self {
        Type::TCon {
            name: "Result".into(),
            args: vec![err, ok],
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::TVar { name, .. } => write!(f, "{}", name),
            Type::TCon { name, args } => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}", name)?;
                    for arg in args {
                        write!(f, " {}", arg)?;
                    }
                    Ok(())
                }
            }
            Type::TFun { from, to } => match from.as_ref() {
                Type::TFun { .. } => write!(f, "({}) -> {}", from, to),
                _ => write!(f, "{} -> {}", from, to),
            },
            Type::TTuple { types } => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scheme {
    pub vars: Vec<String>,
    pub ty: Type,
}

impl Scheme {
    pub fn mono(ty: Type) -> Self {
        Scheme {
            vars: vec![],
            ty,
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "∀{}. {}", self.vars.join(" "), self.ty)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ADTDef {
    pub name: String,
    pub params: Vec<String>,
    pub variants: Vec<(String, Vec<Type>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AliasDef {
    pub name: String,
    pub params: Vec<String>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunDef {
    pub name: String,
    pub type_ann: Type,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopDecl {
    Type(ADTDef),
    Alias(AliasDef),
    Fun(FunDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<TopDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub module: Module,
}
