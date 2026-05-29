use crate::ast::*;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("{span}: unbound variable '{name}'")]
    UnboundVariable { span: Span, name: String },
    #[error("{span}: not a function (value is {val})")]
    NotAFunction { span: Span, val: Value },
    #[error("{span}: pattern match failure (value is {val})")]
    MatchFailure { span: Span, val: Value },
    #[error("{span}: type error: {msg}")]
    Type { span: Span, msg: String },
    #[error("{span}: division by zero")]
    DivisionByZero { span: Span },
    #[error("{span}: hole encountered at runtime")]
    HoleEncountered { span: Span },
}

type BuiltinFn = Arc<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Unit,
    Tag(String, Vec<Value>),
    Closure { params: Vec<String>, body: Expr, env: Env },
    Builtin(String, BuiltinFn),
    IOSuspended(Box<Value>),
}

fn make_builtin(name: &str, f: impl Fn(Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync + 'static) -> Value {
    Value::Builtin(name.into(), Arc::new(f))
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Tag(name, args) => {
                if args.is_empty() { write!(f, "{}", name) }
                else { write!(f, "{}", name)?; for a in args { write!(f, " {}", a)?; } Ok(()) }
            }
            Value::Closure { .. } => write!(f, "<function>"),
            Value::Builtin(name, _) => write!(f, "<builtin {name}>"),
            Value::IOSuspended(_) => write!(f, "<IO action>"),
        }
    }
}

impl fmt::Debug for Value { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) } }

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Tag(n1, a1), Value::Tag(n2, a2)) => n1 == n2 && a1 == a2,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self { Env { bindings: HashMap::new() } }

    pub fn with_builtins() -> Self {
        let mut env = Env::new();

        macro_rules! binop_int {
            ($name:expr, $op:expr) => {
                env.insert($name.into(), make_builtin($name, {
                    let name = $name.to_string();
                    move |args| match (&args[0], &args[1]) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int($op(*a, *b))),
                        _ => Err(RuntimeError::Type { span: Span::new(0,0), msg: format!("{name}: expected Int Int") }),
                    }
                }));
            };
        }
        macro_rules! binop_cmp {
            ($name:expr, $op:expr) => {
                env.insert($name.into(), make_builtin($name, {
                    let name = $name.to_string();
                    move |args| match (&args[0], &args[1]) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool($op(*a, *b))),
                        _ => Err(RuntimeError::Type { span: Span::new(0,0), msg: format!("{name}: expected Int Int") }),
                    }
                }));
            };
        }
        macro_rules! binop_float {
            ($name:expr, $op:expr) => {
                env.insert($name.into(), make_builtin($name, {
                    let name = $name.to_string();
                    move |args| match (&args[0], &args[1]) {
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float($op(*a, *b))),
                        _ => Err(RuntimeError::Type { span: Span::new(0,0), msg: format!("{name}: expected Float Float") }),
                    }
                }));
            };
        }

        binop_int!("iadd", |a, b| a + b);
        binop_int!("isub", |a, b| a - b);
        binop_int!("imul", |a, b| a * b);

        env.insert("idiv".into(), make_builtin("idiv", move |args| match (&args[0], &args[1]) {
            (Value::Int(_a), Value::Int(0)) => Err(RuntimeError::DivisionByZero{span:Span::new(0,0)}),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"idiv: expected Int Int".into()}),
        }));
        env.insert("imod".into(), make_builtin("imod", move |args| match (&args[0], &args[1]) {
            (Value::Int(_a), Value::Int(0)) => Err(RuntimeError::DivisionByZero{span:Span::new(0,0)}),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"imod: expected Int Int".into()}),
        }));
        binop_cmp!("ieq", |a, b| a == b);
        binop_cmp!("ineq", |a, b| a != b);
        binop_cmp!("ilt", |a, b| a < b);
        binop_cmp!("igt", |a, b| a > b);
        binop_cmp!("ilte", |a, b| a <= b);
        binop_cmp!("igte", |a, b| a >= b);

        binop_float!("fadd", |a, b| a + b);
        binop_float!("fsub", |a, b| a - b);
        binop_float!("fmul", |a, b| a * b);
        binop_float!("fdiv", |a, b| a / b);
        binop_float!("feq", |a, b| if a == b { 1.0 } else { 0.0 });
        binop_float!("flt", |a, b| if a < b { 1.0 } else { 0.0 });
        binop_float!("fgt", |a, b| if a > b { 1.0 } else { 0.0 });

        env.insert("strConcat".into(), make_builtin("strConcat", |args| match (&args[0], &args[1]) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"strConcat: expected String String".into()}),
        }));
        env.insert("boolAnd".into(), make_builtin("boolAnd", |args| match (&args[0], &args[1]) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"boolAnd: expected Bool Bool".into()}),
        }));
        env.insert("boolOr".into(), make_builtin("boolOr", |args| match (&args[0], &args[1]) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"boolOr: expected Bool Bool".into()}),
        }));
        env.insert("boolNot".into(), make_builtin("boolNot", |args| match &args[0] {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"boolNot: expected Bool".into()}),
        }));
        env.insert("show".into(), make_builtin("show", |args| Ok(Value::String(format!("{}", args[0])))));
        env.insert("print".into(), make_builtin("print", |args| match &args[0] {
            Value::String(s) => { print!("{}", s); io::stdout().flush().ok(); Ok(Value::IOSuspended(Box::new(Value::Unit))) }
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"print: expected String".into()}),
        }));
        env.insert("println".into(), make_builtin("println", |args| match &args[0] {
            Value::String(s) => { println!("{}", s); Ok(Value::IOSuspended(Box::new(Value::Unit))) }
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"println: expected String".into()}),
        }));
        env.insert("readLine".into(), make_builtin("readLine", |_args| {
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let t = input.trim_end_matches('\n').trim_end_matches('\r');
            Ok(Value::IOSuspended(Box::new(Value::String(t.to_string()))))
        }));

        env.insert("strUncons".into(), make_builtin("strUncons", |args| {
            match &args[0] {
                Value::String(s) => {
                    let mut chars = s.chars();
                    match chars.next() {
                        Some(c) => {
                            let rest: String = chars.collect();
                            Ok(Value::Tag("Some".into(), vec![
                                Value::Tag("Tuple".into(), vec![
                                    Value::Char(c),
                                    Value::String(rest),
                                ]),
                            ]))
                        }
                        None => Ok(Value::Tag("None".into(), vec![])),
                    }
                }
                _ => Err(RuntimeError::Type {
                    span: Span::new(0, 0),
                    msg: "strUncons: expected String".into(),
                }),
            }
        }));

        env.insert("charEq".into(), make_builtin("charEq", |args| match (&args[0], &args[1]) {
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a == b)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"charEq: expected Char Char".into()}),
        }));
        env.insert("charIsDigit".into(), make_builtin("charIsDigit", |args| match &args[0] {
            Value::Char(c) => Ok(Value::Bool(c.is_ascii_digit())),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"charIsDigit: expected Char".into()}),
        }));
        env.insert("charIsSpace".into(), make_builtin("charIsSpace", |args| match &args[0] {
            Value::Char(c) => Ok(Value::Bool(c.is_whitespace())),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"charIsSpace: expected Char".into()}),
        }));
        env.insert("charToInt".into(), make_builtin("charToInt", |args| match &args[0] {
            Value::Char(c) if c.is_ascii_digit() => Ok(Value::Int((*c as u8 - b'0') as i64)),
            Value::Char(c) => Ok(Value::Int(*c as i64)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"charToInt: expected Char".into()}),
        }));
        env.insert("intToChar".into(), make_builtin("intToChar", |args| match &args[0] {
            Value::Int(n) if (0..=9).contains(n) => Ok(Value::Char((*n as u8 + b'0') as char)),
            Value::Int(n) if (0..=127).contains(n) => Ok(Value::Char(*n as u8 as char)),
            _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"intToChar: expected Int".into()}),
        }));
        env.insert("strFromList".into(), make_builtin("strFromList", |args| {
            fn collect_chars(v: &Value) -> Result<String, RuntimeError> {
                match v {
                    Value::Tag(name, vals) if name == "Nil" && vals.is_empty() => Ok(String::new()),
                    Value::Tag(name, vals) if name == "Cons" && vals.len() == 2 => {
                        let head = match &vals[0] {
                            Value::Char(c) => *c,
                            _ => return Err(RuntimeError::Type{span:Span::new(0,0),msg:"strFromList: expected List Char".into()}),
                        };
                        let tail = collect_chars(&vals[1])?;
                        let mut s = String::new();
                        s.push(head);
                        s.push_str(&tail);
                        Ok(s)
                    }
                    _ => Err(RuntimeError::Type{span:Span::new(0,0),msg:"strFromList: expected List Char".into()}),
                }
            }
            Ok(Value::String(collect_chars(&args[0])?))
        }));

        env.insert("Nil".into(), Value::Tag("Nil".into(), vec![]));
        env.insert("None".into(), Value::Tag("None".into(), vec![]));
        env
    }

    pub fn insert(&mut self, name: String, value: Value) { self.bindings.insert(name, value); }
    pub fn lookup(&self, name: &str) -> Option<&Value> { self.bindings.get(name) }
}

pub struct Interpreter {
    env: Env,
    pub globals: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Env::with_builtins();
        Interpreter {
            env: env.clone(),
            globals: env,
        }
    }

    pub fn eval_program(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        for decl in &program.module.declarations {
            if let TopDecl::Type(adt) = decl {
                for (ctor_name, _) in &adt.variants {
                    let tag = Value::Tag(ctor_name.clone(), vec![]);
                    self.env.insert(ctor_name.clone(), tag.clone());
                    self.globals.insert(ctor_name.clone(), tag);
                }
            }
        }

        for decl in &program.module.declarations {
            if let TopDecl::Fun(fun) = decl {
                let closure = Value::Closure {
                    params: fun.params.clone(),
                    body: fun.body.clone(),
                    env: self.env.clone(),
                };
                self.env.insert(fun.name.clone(), closure.clone());
                self.globals.insert(fun.name.clone(), closure);
            }
        }

        if let Some(main) = self.env.lookup("main") {
            let main_val = main.clone();
            self.eval_app(main_val, vec![])
        } else {
            Ok(Value::Unit)
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value, .. } => Ok(self.eval_literal(value)),
            Expr::Var { name, span } => {
                let val = self.env.lookup(name)
                    .or_else(|| self.globals.lookup(name)).cloned()
                    .ok_or_else(|| RuntimeError::UnboundVariable { span: *span, name: name.clone() })?;
                match val {
                    Value::Closure { ref params, .. } if params.is_empty() => {
                        self.eval_app(val, vec![])
                    }
                    _ => Ok(val),
                }
            }
            Expr::Lambda { params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| match p {
                    Pattern::PVar { name, .. } => name.clone(), _ => "_".into(),
                }).collect();
                Ok(Value::Closure { params: param_names, body: *body.clone(), env: self.env.clone() })
            }
            Expr::App { func, args, .. } => {
                let f = self.eval(func)?;
                let mut evaluated_args = vec![];
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                self.eval_app(f, evaluated_args)
            }
            Expr::If { cond, then_b, else_b, .. } => {
                let cv = self.eval(cond)?;
                let truthy = matches!(&cv, Value::Bool(true)) || matches!(&cv, Value::Int(n) if *n != 0);
                if truthy { self.eval(then_b) } else { self.eval(else_b) }
            }
            Expr::Match { expr: e, arms, .. } => {
                let val = self.eval(e)?;
                for (pat, body) in arms {
                    let saved_env = self.env.clone();
                    if self.match_pattern(pat, &val) {
                        let result = self.eval(body);
                        if result.is_err() { self.env = saved_env; }
                        return result;
                    }
                    self.env = saved_env;
                }
                Err(RuntimeError::MatchFailure{span:expr.span(),val})
            }
            Expr::Do { stmts, .. } => self.eval_do(stmts),
            Expr::Let { bindings, body, .. } => {
                let saved_env = self.env.clone();
                for binding in bindings {
                    if binding.params.is_empty() {
                        let val = self.eval(&binding.body)?;
                        self.env.insert(binding.name.clone(), val);
                    } else {
                        let closure = Value::Closure {
                            params: binding.params.clone(),
                            body: binding.body.clone(),
                            env: self.env.clone(),
                        };
                        self.env.insert(binding.name.clone(), closure);
                    }
                }
                let result = self.eval(body);
                self.env = saved_env;
                result
            }
            Expr::Hole { span } => Err(RuntimeError::HoleEncountered{span:*span}),
            Expr::Tuple { exprs, .. } => {
                let vals: Result<Vec<Value>, RuntimeError> = exprs.iter().map(|e| self.eval(e)).collect();
                Ok(Value::Tag("Tuple".into(), vals?))
            }
            Expr::List { exprs, .. } => {
                let mut list = Value::Tag("Nil".into(), vec![]);
                for e in exprs.iter().rev() {
                    list = Value::Tag("Cons".into(), vec![self.eval(e)?, list]);
                }
                Ok(list)
            }
            Expr::BinOp { op, left, right, .. } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                self.eval_binop(*op, l, r, left.span())
            }
            Expr::UnaryOp { op, expr: e, .. } => {
                let v = self.eval(e)?;
                self.eval_unaryop(*op, v, e.span())
            }
            Expr::Return { expr, .. } => {
                let val = self.eval(expr)?;
                Ok(Value::IOSuspended(Box::new(val)))
            }
        }
    }

    fn eval_do(&mut self, stmts: &[Stmt]) -> Result<Value, RuntimeError> {
        if stmts.is_empty() { return Ok(Value::IOSuspended(Box::new(Value::Unit))); }
        let len = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Expr(e) => {
                    let val = self.eval(e)?;
                    let val = self.run_io(val)?;
                    if i == len - 1 { return Ok(val); }
                }
                Stmt::Bind { name, expr: e } => {
                    let val = self.eval(e)?;
                    let val = self.run_io(val)?;
                    self.env.insert(name.clone(), val);
                }
            }
        }
        Ok(Value::Unit)
    }

    pub fn run_io(&mut self, val: Value) -> Result<Value, RuntimeError> {
        match val {
            Value::IOSuspended(inner) => Ok(*inner),
            other => Ok(other),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(n) => Value::Float(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::Char(*c),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Unit => Value::Unit,
        }
    }

    fn eval_app(&mut self, func: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Closure { params, body, env } => {
                let saved_env = self.env.clone();
                self.env = env;
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.env.insert(param.clone(), arg.clone());
                }
                let result = if args.len() < params.len() {
                    let remaining: Vec<String> = params[args.len()..].to_vec();
                    Ok(Value::Closure { params: remaining, body, env: self.env.clone() })
                } else {
                    self.eval(&body)
                };
                self.env = saved_env;
                result
            }
            Value::Builtin(_, f) => f(args),
            Value::Tag(name, tag_args) => {
                let mut all_args = tag_args.clone();
                all_args.extend(args);
                Ok(Value::Tag(name, all_args))
            }
            Value::IOSuspended(inner) => match *inner {
                Value::Unit => Ok(Value::Unit),
                _ => Ok(Value::IOSuspended(inner)),
            },
            _ => Err(RuntimeError::NotAFunction{span:Span::new(0,0),val:func}),
        }
    }

    fn eval_binop(&self, op: BinOp, l: Value, r: Value, span: Span) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => ik(|a,b| Ok(a+b), &l, &r, span, "+"),
            BinOp::Sub => ik(|a,b| Ok(a-b), &l, &r, span, "-"),
            BinOp::Mul => ik(|a,b| Ok(a*b), &l, &r, span, "*"),
            BinOp::Div => ik(|a,b| if b==0 {Err("div by zero".into())} else {Ok(a/b)}, &l, &r, span, "/"),
            BinOp::Mod => ik(|a,b| if b==0 {Err("div by zero".into())} else {Ok(a%b)}, &l, &r, span, "%"),
            BinOp::Eq => Ok(Value::Bool(l == r)),
            BinOp::Neq => Ok(Value::Bool(l != r)),
            BinOp::Lt => cmp_ok(|a,b|a<b, &l, &r, span, "<"),
            BinOp::Gt => cmp_ok(|a,b|a>b, &l, &r, span, ">"),
            BinOp::Lte => cmp_ok(|a,b|a<=b, &l, &r, span, "<="),
            BinOp::Gte => cmp_ok(|a,b|a>=b, &l, &r, span, ">="),
            BinOp::Concat => match (&l,&r) {
                (Value::String(a),Value::String(b)) => Ok(Value::String(format!("{}{}",a,b))),
                _ => Err(RuntimeError::Type{span,msg:"++: expected String String".into()}),
            },
            BinOp::And => match (&l,&r) {
                (Value::Bool(a),Value::Bool(b)) => Ok(Value::Bool(*a&&*b)),
                _ => Err(RuntimeError::Type{span,msg:"&&: expected Bool Bool".into()}),
            },
            BinOp::Or => match (&l,&r) {
                (Value::Bool(a),Value::Bool(b)) => Ok(Value::Bool(*a||*b)),
                _ => Err(RuntimeError::Type{span,msg:"||: expected Bool Bool".into()}),
            },
            BinOp::Pipe => unreachable!(),
        }
    }

    fn eval_unaryop(&self, op: UnaryOp, v: Value, span: Span) -> Result<Value, RuntimeError> {
        match op {
            UnaryOp::Neg => match v {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(n) => Ok(Value::Float(-n)),
                _ => Err(RuntimeError::Type{span,msg:"-: expected Int or Float".into()}),
            },
            UnaryOp::Not => match v {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                Value::Int(n) => Ok(Value::Bool(n==0)),
                _ => Err(RuntimeError::Type{span,msg:"!: expected Bool".into()}),
            },
        }
    }

    fn match_pattern(&mut self, pat: &Pattern, val: &Value) -> bool {
        match pat {
            Pattern::Wildcard{..} => true,
            Pattern::PVar{name,..} => { if name != "_" { self.env.insert(name.clone(), val.clone()); } true }
            Pattern::PLit{value,..} => val == &self.eval_literal(value),
            Pattern::PCtor{name,args,..} => match val {
                Value::Tag(vn,va) if vn==name && va.len()==args.len() =>
                    args.iter().zip(va.iter()).all(|(p,v)| self.match_pattern(p,v)),
                _ => false,
            },
            Pattern::PTuple{pats,..} => match val {
                Value::Tag(name,vals) if name=="Tuple" && vals.len()==pats.len() =>
                    pats.iter().zip(vals.iter()).all(|(p,v)| self.match_pattern(p,v)),
                _ => false,
            },
            Pattern::PNil{..} => matches!(val, Value::Tag(n,a) if n=="Nil" && a.is_empty()),
            Pattern::PCons{head,tail,..} => match val {
                Value::Tag(name,args) if name=="Cons" && args.len()==2 =>
                    self.match_pattern(head,&args[0]) && self.match_pattern(tail,&args[1]),
                _ => false,
            },
        }
    }
}

fn ik(f: impl FnOnce(i64,i64)->Result<i64,String>, l: &Value, r: &Value, span: Span, op: &str) -> Result<Value,RuntimeError> {
    match (l,r) {
        (Value::Int(a),Value::Int(b)) => match f(*a,*b) {
            Ok(v) => Ok(Value::Int(v)),
            Err(_) => Err(RuntimeError::DivisionByZero{span}),
        },
        _ => Err(RuntimeError::Type{span,msg:format!("{op}: expected Int Int")}),
    }
}

fn cmp_ok(f: impl FnOnce(i64,i64)->bool, l: &Value, r: &Value, span: Span, op: &str) -> Result<Value,RuntimeError> {
    match (l,r) {
        (Value::Int(a),Value::Int(b)) => Ok(Value::Bool(f(*a,*b))),
        _ => Err(RuntimeError::Type{span,msg:format!("{op}: expected Int Int")}),
    }
}
