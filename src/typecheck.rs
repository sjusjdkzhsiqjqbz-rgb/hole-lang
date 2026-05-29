use crate::ast::*;
use crate::builtins::BuiltinCtx;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("{span}: type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        span: Span,
        expected: Type,
        got: Type,
    },
    #[error("{span}: unbound variable '{name}'")]
    UnboundVariable { span: Span, name: String },
    #[error("{span}: unbound type '{name}'")]
    UnboundType { span: Span, name: String },
    #[error("{span}: not a function (type is {ty})")]
    NotAFunction { span: Span, ty: Type },
    #[error("{span}: not a sum type (type is {ty})")]
    NotASumType { span: Span, ty: Type },
    #[error("{span}: unknown constructor '{name}'")]
    UnknownConstructor { span: Span, name: String },
    #[error("{span}: inexhaustive pattern match")]
    InexhaustiveMatch { span: Span },
    #[error("{span}: duplicate constructor '{name}' in type '{ty}'")]
    DuplicateConstructor { span: Span, name: String, ty: String },
    #[error("{span}: recursive alias '{name}'")]
    RecursiveAlias { span: Span, name: String },
    #[error("{span}: expected IO type, got {ty}")]
    ExpectedIO { span: Span, ty: Type },
    #[error("{span}: hole found (type: {hole_ty})")]
    HoleFound { span: Span, hole_ty: Type },
    #[error("{span}: {msg}")]
    Other { span: Span, msg: String },
}

pub struct TypeEnv {
    subst: HashMap<u64, Type>,
    tvar_counter: u64,
    value_env: Vec<HashMap<String, Scheme>>,
    type_ctors: HashMap<String, Scheme>,
    adts: HashMap<String, (Vec<String>, Vec<(String, Vec<Type>)>)>,
    pub holes: Vec<(Span, Type)>,
}

impl TypeEnv {
    pub fn new(builtins: &BuiltinCtx) -> Self {
        TypeEnv {
            subst: HashMap::new(),
            tvar_counter: 0,
            value_env: vec![builtins.values.clone()],
            type_ctors: builtins.types.clone(),
            adts: builtins.adts.clone(),
            holes: vec![],
        }
    }

    fn fresh_id(&mut self) -> u64 {
        let id = self.tvar_counter;
        self.tvar_counter += 1;
        id
    }

    fn fresh_tvar(&mut self) -> Type {
        let id = self.fresh_id();
        Type::TVar {
            name: format!("t{}", id),
            id,
        }
    }

    pub fn prune(&self, ty: &Type) -> Type {
        match ty {
            Type::TVar { id, .. } => {
                if let Some(t) = self.subst.get(id) {
                    self.prune(t)
                } else {
                    ty.clone()
                }
            }
            Type::TFun { from, to } => Type::TFun {
                from: Box::new(self.prune(from)),
                to: Box::new(self.prune(to)),
            },
            Type::TCon { name, args } => Type::TCon {
                name: name.clone(),
                args: args.iter().map(|a| self.prune(a)).collect(),
            },
            Type::TTuple { types } => Type::TTuple {
                types: types.iter().map(|t| self.prune(t)).collect(),
            },
        }
    }

    fn occurs_check(&self, id: u64, ty: &Type) -> Result<(), TypeError> {
        match ty {
            Type::TVar { id: id2, .. } => {
                if *id2 == id {
                    return Err(TypeError::Other {
                        span: Span::new(0, 0),
                        msg: "infinite type".into(),
                    });
                }
                if let Some(t) = self.subst.get(id2) {
                    return self.occurs_check(id, t);
                }
            }
            Type::TFun { from, to } => {
                self.occurs_check(id, from)?;
                self.occurs_check(id, to)?;
            }
            Type::TCon { args, .. } => {
                for a in args {
                    self.occurs_check(id, a)?;
                }
            }
            Type::TTuple { types } => {
                for t in types {
                    self.occurs_check(id, t)?;
                }
            }
        }
        Ok(())
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), TypeError> {
        let t1 = self.prune(t1);
        let t2 = self.prune(t2);

        match (&t1, &t2) {
            (Type::TVar { id: id1, .. }, Type::TVar { id: id2, .. }) if id1 == id2 => {
                return Ok(());
            }
            (Type::TVar { id, .. }, _) => {
                self.occurs_check(*id, &t2)?;
                self.subst.insert(*id, t2.clone());
                return Ok(());
            }
            (_, Type::TVar { id, .. }) => {
                self.occurs_check(*id, &t1)?;
                self.subst.insert(*id, t1.clone());
                return Ok(());
            }
            (Type::TFun { from: f1, to: t1 }, Type::TFun { from: f2, to: t2 }) => {
                self.unify(f1, f2)?;
                self.unify(t1, t2)?;
                return Ok(());
            }
            (Type::TCon { name: n1, args: a1 }, Type::TCon { name: n2, args: a2 }) => {
                if n1 != n2 {
                    return Err(TypeError::TypeMismatch {
                        span: Span::new(0, 0),
                        expected: t1,
                        got: t2,
                    });
                }
                if a1.len() != a2.len() {
                    return Err(TypeError::TypeMismatch {
                        span: Span::new(0, 0),
                        expected: t1,
                        got: t2,
                    });
                }
                for (a, b) in a1.iter().zip(a2.iter()) {
                    self.unify(a, b)?;
                }
                return Ok(());
            }
            (Type::TTuple { types: ts1 }, Type::TTuple { types: ts2 }) => {
                if ts1.len() != ts2.len() {
                    return Err(TypeError::TypeMismatch {
                        span: Span::new(0, 0),
                        expected: t1,
                        got: t2,
                    });
                }
                for (a, b) in ts1.iter().zip(ts2.iter()) {
                    self.unify(a, b)?;
                }
                return Ok(());
            }
            _ => {}
        }

        Err(TypeError::TypeMismatch {
            span: Span::new(0, 0),
            expected: t1.clone(),
            got: t2.clone(),
        })
    }

    fn generalize(&self, ty: &Type) -> Scheme {
        let ty = self.prune(ty);
        let mut free_vars = HashMap::new();
        self.collect_free_tvars(&ty, &mut free_vars);

        let mut bound_vars = HashMap::new();
        for scope in self.value_env.iter().rev() {
            for scheme in scope.values() {
                self.collect_bound_tvars(scheme, &mut bound_vars);
            }
        }
        for scheme in self.type_ctors.values() {
            self.collect_bound_tvars(scheme, &mut bound_vars);
        }

        let mut vars: Vec<String> = free_vars
            .keys()
            .filter(|k| !bound_vars.contains_key(*k))
            .cloned()
            .collect();
        vars.sort();

        if vars.is_empty() {
            Scheme::mono(ty)
        } else {
            Scheme { vars, ty }
        }
    }

    fn collect_free_tvars(&self, ty: &Type, vars: &mut HashMap<String, ()>) {
        match ty {
            Type::TVar { name, id } => {
                if !self.subst.contains_key(id) {
                    vars.insert(name.clone(), ());
                }
            }
            Type::TFun { from, to } => {
                self.collect_free_tvars(from, vars);
                self.collect_free_tvars(to, vars);
            }
            Type::TCon { args, .. } => {
                for a in args {
                    self.collect_free_tvars(a, vars);
                }
            }
            Type::TTuple { types } => {
                for t in types {
                    self.collect_free_tvars(t, vars);
                }
            }
        }
    }

    fn collect_bound_tvars(&self, scheme: &Scheme, vars: &mut HashMap<String, ()>) {
        for v in &scheme.vars {
            vars.insert(v.clone(), ());
        }
    }

    pub fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let mut subst_map: HashMap<String, Type> = HashMap::new();
        for v in &scheme.vars {
            subst_map.insert(v.clone(), self.fresh_tvar());
        }
        self.apply_var_subst(&scheme.ty, &subst_map)
    }

    fn apply_var_subst(&self, ty: &Type, subst_map: &HashMap<String, Type>) -> Type {
        match ty {
            Type::TVar { name, .. } => {
                if let Some(t) = subst_map.get(name) {
                    t.clone()
                } else {
                    ty.clone()
                }
            }
            Type::TFun { from, to } => Type::TFun {
                from: Box::new(self.apply_var_subst(from, subst_map)),
                to: Box::new(self.apply_var_subst(to, subst_map)),
            },
            Type::TCon { name, args } => Type::TCon {
                name: name.clone(),
                args: args.iter().map(|a| self.apply_var_subst(a, subst_map)).collect(),
            },
            Type::TTuple { types } => Type::TTuple {
                types: types.iter().map(|t| self.apply_var_subst(t, subst_map)).collect(),
            },
        }
    }

    fn lookup_value(&self, name: &str) -> Result<Scheme, TypeError> {
        for scope in self.value_env.iter().rev() {
            if let Some(scheme) = scope.get(name) {
                return Ok(scheme.clone());
            }
        }
        Err(TypeError::UnboundVariable {
            span: Span::new(0, 0),
            name: name.into(),
        })
    }

    fn lookup_type(&self, name: &str) -> Result<Scheme, TypeError> {
        self.type_ctors
            .get(name)
            .cloned()
            .ok_or_else(|| TypeError::UnboundType {
                span: Span::new(0, 0),
                name: name.into(),
            })
    }

    fn push_scope(&mut self) {
        self.value_env.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.value_env.pop();
    }

    fn insert_value(&mut self, name: String, scheme: Scheme) {
        if let Some(scope) = self.value_env.last_mut() {
            scope.insert(name, scheme);
        }
    }

    pub fn register_types(&mut self, decls: &[TopDecl]) -> Result<(), TypeError> {
        for decl in decls {
            match decl {
                TopDecl::Type(adt) => {
                    if self.adts.contains_key(&adt.name) {
                        continue;
                    }
                    let mut freshened_variants = vec![];
                    for (ctor_name, arg_types) in &adt.variants {
                        freshened_variants.push((
                            ctor_name.clone(),
                            arg_types.iter().map(|t| self.freshen_type(t)).collect(),
                        ));
                    }
                    self.adts.insert(
                        adt.name.clone(),
                        (adt.params.clone(), freshened_variants.clone()),
                    );
                    let scheme = if adt.params.is_empty() {
                        Scheme::mono(Type::TCon {
                            name: adt.name.clone(),
                            args: vec![],
                        })
                    } else {
                        Scheme {
                            vars: adt.params.clone(),
                            ty: Type::TCon {
                                name: adt.name.clone(),
                                args: adt.params.iter().map(|p| {
                                    Type::TVar { name: p.clone(), id: self.fresh_id() }
                                }).collect(),
                            },
                        }
                    };
                    self.type_ctors.insert(adt.name.clone(), scheme);

                    for (ctor_name, _) in &freshened_variants {
                        let ctor_type = self.build_constructor_type(&adt.name, &adt.params, ctor_name, &freshened_variants);
                        self.insert_value(ctor_name.clone(), ctor_type);
                    }
                }
                TopDecl::Alias(alias) => {
                    let scheme = if alias.params.is_empty() {
                        Scheme::mono(alias.ty.clone())
                    } else {
                        Scheme {
                            vars: alias.params.clone(),
                            ty: alias.ty.clone(),
                        }
                    };
                    self.type_ctors.insert(alias.name.clone(), scheme);
                }
                TopDecl::Fun(_) => {}
            }
        }
        Ok(())
    }

    fn build_constructor_type(
        &self,
        type_name: &str,
        params: &[String],
        ctor_name: &str,
        variants: &[(String, Vec<Type>)],
    ) -> Scheme {
        let ctor_args = variants
            .iter()
            .find(|(n, _)| n == ctor_name)
            .map(|(_, args)| args.clone())
            .unwrap_or_default();

        let result_type = if params.is_empty() {
            Type::TCon {
                name: type_name.into(),
                args: vec![],
            }
        } else {
            Type::TCon {
                name: type_name.into(),
                args: params.iter().map(|p| Type::TVar { name: p.clone(), id: 0 }).collect(),
            }
        };

        let mut full_type = result_type;
        for arg in ctor_args.iter().rev() {
            full_type = Type::TFun {
                from: Box::new(arg.clone()),
                to: Box::new(full_type),
            };
        }

        if params.is_empty() {
            Scheme::mono(full_type)
        } else {
            Scheme {
                vars: params.to_vec(),
                ty: full_type,
            }
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        self.register_types(&program.module.declarations)?;

        for decl in &program.module.declarations.clone() {
            match decl {
                TopDecl::Fun(fun) => {
                    let mut fun = fun.clone();
                    fun.type_ann = self.freshen_type(&fun.type_ann);
                    let scheme = self.check_fun_def(&fun)?;
                    self.insert_value(fun.name.clone(), scheme);
                }
                _ => {}
            }
        }

        let main_scheme = self.value_env.last().and_then(|s| s.get("main")).cloned();
        if let Some(main_scheme) = main_scheme {
            let main_type = self.instantiate(&main_scheme);
            let expected = self.prune(&Type::io(Type::unit()));
            let main_type = self.prune(&main_type);
            self.unify(&main_type, &expected).map_err(|_| TypeError::TypeMismatch {
                span: Span::new(0, 0),
                expected,
                got: main_type,
            })?;
        }

        Ok(())
    }

    fn freshen_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::TVar { name, .. } => {
                let id = self.fresh_id();
                Type::TVar { name: name.clone(), id }
            }
            Type::TFun { from, to } => Type::TFun {
                from: Box::new(self.freshen_type(from)),
                to: Box::new(self.freshen_type(to)),
            },
            Type::TCon { name, args } => Type::TCon {
                name: name.clone(),
                args: args.iter().map(|a| self.freshen_type(a)).collect(),
            },
            Type::TTuple { types } => Type::TTuple {
                types: types.iter().map(|t| self.freshen_type(t)).collect(),
            },
        }
    }

    fn check_fun_def(&mut self, fun: &FunDef) -> Result<Scheme, TypeError> {
        let mut current_type = fun.type_ann.clone();

        for _ in &fun.params {
            let arg_tvar = self.fresh_tvar();
            let ret_tvar = self.fresh_tvar();
            self.unify(
                &current_type,
                &Type::TFun {
                    from: Box::new(arg_tvar),
                    to: Box::new(ret_tvar.clone()),
                },
            ).map_err(|e| TypeError::TypeMismatch {
                span: Span::new(0, 0),
                expected: fun.type_ann.clone(),
                got: current_type.clone(),
            })?;
            current_type = ret_tvar;
        }

        let return_type = current_type.clone();

        self.push_scope();

        self.insert_value(fun.name.clone(), Scheme::mono(fun.type_ann.clone()));

        let mut fn_type = fun.type_ann.clone();
        for param in fun.params.iter() {
            let (param_type, rest_type) = self.destructure_function_type(&fn_type)?;
            self.insert_value(param.clone(), Scheme::mono(param_type));
            fn_type = rest_type;
        }

        let body_type = self.infer(&fun.body)?;

        self.pop_scope();

        self.unify(&body_type, &return_type).map_err(|e| TypeError::TypeMismatch {
            span: fun.body.span(),
            expected: return_type.clone(),
            got: body_type,
        })?;

        let full_type = self.prune(&fun.type_ann);
        Ok(self.generalize(&full_type))
    }

    fn destructure_function_type(&mut self, ty: &Type) -> Result<(Type, Type), TypeError> {
        match self.prune(ty) {
            Type::TFun { from, to } => Ok((*from.clone(), *to.clone())),
            _ => {
                let a = self.fresh_tvar_silent();
                let b = self.fresh_tvar_silent();
                Ok((a, b))
            }
        }
    }

    fn fresh_tvar_silent(&mut self) -> Type {
        let id = self.fresh_id();
        Type::TVar { name: "?".into(), id }
    }

    pub fn infer(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal { value, span } => Ok(self.literal_type(value, *span)?),
            Expr::Var { name, span } => {
                let scheme = self.lookup_value(name).map_err(|_| TypeError::UnboundVariable {
                    span: *span,
                    name: name.clone(),
                })?;
                Ok(self.instantiate(&scheme))
            }
            Expr::Lambda { params, body, span } => {
                let param_types: Vec<Type> = (0..params.len()).map(|_| self.fresh_tvar()).collect();
                self.push_scope();
                for (pat, pt) in params.iter().zip(param_types.iter()) {
                    let bindings = self.infer_pattern(pat, pt)?;
                    for (name, ty) in bindings {
                        self.insert_value(name, Scheme::mono(ty));
                    }
                }
                let body_type = self.infer(body)?;
                self.pop_scope();

                let mut result_type = body_type;
                for pt in param_types.iter().rev() {
                    result_type = Type::TFun {
                        from: Box::new(pt.clone()),
                        to: Box::new(result_type),
                    };
                }
                Ok(result_type)
            }
            Expr::App { func, args, span } => {
                let mut func_type = self.infer(func)?;

                for arg in args {
                    let arg_type = self.infer(arg)?;
                    let ret_tvar = self.fresh_tvar();
                    self.unify(
                        &func_type,
                        &Type::TFun {
                            from: Box::new(arg_type),
                            to: Box::new(ret_tvar.clone()),
                        },
                    ).map_err(|_| TypeError::NotAFunction {
                        span: func.span(),
                        ty: self.prune(&func_type),
                    })?;
                    func_type = ret_tvar;
                }

                Ok(self.prune(&func_type))
            }
            Expr::If { cond, then_b, else_b, span } => {
                let cond_type = self.infer(cond)?;
                self.unify(&cond_type, &Type::bool()).map_err(|_| TypeError::TypeMismatch {
                    span: cond.span(),
                    expected: Type::bool(),
                    got: cond_type,
                })?;

                let then_type = self.infer(then_b)?;
                let else_type = self.infer(else_b)?;
                self.unify(&then_type, &else_type).map_err(|_| TypeError::TypeMismatch {
                    span: else_b.span(),
                    expected: self.prune(&then_type),
                    got: self.prune(&else_type),
                })?;

                Ok(self.prune(&then_type))
            }
            Expr::Match { expr, arms, span } => {
                let matched_type = self.infer(expr)?;
                let result_type = self.fresh_tvar();

                for (pat, body) in arms {
                    self.push_scope();
                    let bindings = self.infer_pattern(pat, &matched_type)?;
                    for (name, ty) in bindings {
                        self.insert_value(name, Scheme::mono(ty));
                    }
                    let body_type = self.infer(body)?;
                    self.unify(&body_type, &result_type).map_err(|_| TypeError::TypeMismatch {
                        span: body.span(),
                        expected: self.prune(&result_type),
                        got: body_type,
                    })?;
                    self.pop_scope();
                }

                self.check_exhaustiveness(*span, &matched_type, arms)?;

                Ok(self.prune(&result_type))
            }
            Expr::Do { stmts, span } => {
                if stmts.is_empty() {
                    return Ok(Type::io(Type::unit()));
                }

                let last_idx = stmts.len() - 1;
                for (i, stmt) in stmts.iter().enumerate() {
                    match stmt {
                        Stmt::Expr(e) => {
                            let ty = self.infer(e)?;
                            if i == last_idx {
                                return Ok(self.prune(&ty));
                            } else {
                                let pruned = self.prune(&ty);
                                match pruned {
                                    Type::TCon { ref name, .. } if name == "IO" => {}
                                    _ => {
                                        return Err(TypeError::ExpectedIO {
                                            span: e.span(),
                                            ty: pruned,
                                        });
                                    }
                                }
                            }
                        }
                        Stmt::Bind { name, expr: e } => {
                            let ty = self.infer(e)?;
                            let pruned = self.prune(&ty);
                            match pruned {
                                Type::TCon { args, .. } => {
                                    let inner = args.first().cloned().unwrap_or(Type::unit());
                                    self.insert_value(name.clone(), Scheme::mono(inner));
                                }
                                _ => {
                                    return Err(TypeError::ExpectedIO {
                                        span: e.span(),
                                        ty: pruned,
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(Type::io(Type::unit()))
            }
            Expr::Let { bindings, body, span } => {
                self.push_scope();

                let mut binding_schemes = Vec::new();
                for binding in bindings {
                    if binding.params.is_empty() {
                        let value_type = self.infer(&binding.body)?;
                        let scheme = self.generalize(&value_type);
                        self.insert_value(binding.name.clone(), scheme.clone());
                        binding_schemes.push((binding.name.clone(), scheme));
                    } else {
                        let mut fun_type: Type = self.fresh_tvar();
                        let param_types: Vec<Type> = (0..binding.params.len())
                            .map(|_| self.fresh_tvar())
                            .collect();

                        self.push_scope();
                        for (p, pt) in binding.params.iter().zip(param_types.iter()) {
                            self.insert_value(p.clone(), Scheme::mono(pt.clone()));
                        }
                        let body_type = self.infer(&binding.body)?;
                        self.pop_scope();

                        let mut result = body_type;
                        for pt in param_types.iter().rev() {
                            result = Type::TFun {
                                from: Box::new(pt.clone()),
                                to: Box::new(result),
                            };
                        }
                        let scheme = self.generalize(&result);
                        self.insert_value(binding.name.clone(), scheme.clone());
                        binding_schemes.push((binding.name.clone(), scheme));
                    }
                }

                let body_type = self.infer(body)?;
                self.pop_scope();
                Ok(body_type)
            }
            Expr::Hole { span } => {
                let ty = self.fresh_tvar();
                self.holes.push((*span, ty.clone()));
                Ok(ty)
            }
            Expr::Tuple { exprs, .. } => {
                let types: Result<Vec<Type>, TypeError> = exprs.iter().map(|e| self.infer(e)).collect();
                Ok(Type::TTuple { types: types? })
            }
            Expr::List { exprs, span } => {
                let elem_type = self.fresh_tvar();
                for e in exprs {
                    let t = self.infer(e)?;
                    self.unify(&t, &elem_type).map_err(|_| TypeError::TypeMismatch {
                        span: e.span(),
                        expected: self.prune(&elem_type),
                        got: t,
                    })?;
                }
                Ok(Type::list(self.prune(&elem_type)))
            }
            Expr::BinOp { op, left, right, span } => {
                let builtin_name = match op {
                    BinOp::Add => "iadd",
                    BinOp::Sub => "isub",
                    BinOp::Mul => "imul",
                    BinOp::Div => "idiv",
                    BinOp::Mod => "imod",
                    BinOp::Eq => "ieq",
                    BinOp::Neq => "ineq",
                    BinOp::Lt => "ilt",
                    BinOp::Gt => "igt",
                    BinOp::Lte => "ilte",
                    BinOp::Gte => "igte",
                    BinOp::Concat => "strConcat",
                    BinOp::And => "boolAnd",
                    BinOp::Or => "boolOr",
                    BinOp::Pipe => unreachable!("pipe handled in parser"),
                };

                let scheme = self.lookup_value(builtin_name).map_err(|_| TypeError::UnboundVariable {
                    span: *span,
                    name: builtin_name.into(),
                })?;
                let op_type = self.instantiate(&scheme);

                let left_type = self.infer(left)?;
                let right_type = self.infer(right)?;
                let result_type = self.fresh_tvar();

                self.unify(
                    &op_type,
                    &Type::TFun {
                        from: Box::new(left_type),
                        to: Box::new(Type::TFun {
                            from: Box::new(right_type),
                            to: Box::new(result_type.clone()),
                        }),
                    },
                ).map_err(|_| TypeError::TypeMismatch {
                    span: *span,
                    expected: Type::int(),
                    got: Type::float(),
                })?;

                Ok(self.prune(&result_type))
            }
            Expr::UnaryOp { op, expr: e, span } => {
                let ty = self.infer(e)?;
                match op {
                    UnaryOp::Neg => {
                        self.unify(&ty, &Type::int()).map_err(|_| TypeError::TypeMismatch {
                            span: e.span(),
                            expected: Type::int(),
                            got: ty,
                        })?;
                        Ok(Type::int())
                    }
                    UnaryOp::Not => {
                        self.unify(&ty, &Type::bool()).map_err(|_| TypeError::TypeMismatch {
                            span: e.span(),
                            expected: Type::bool(),
                            got: ty,
                        })?;
                        Ok(Type::bool())
                    }
                }
            }
            Expr::Return { expr, .. } => {
                let inner = self.infer(expr)?;
                Ok(Type::io(inner))
            }
        }
    }

    fn literal_type(&self, lit: &Literal, span: Span) -> Result<Type, TypeError> {
        match lit {
            Literal::Int(_) => Ok(Type::int()),
            Literal::Float(_) => Ok(Type::float()),
            Literal::String(_) => Ok(Type::string()),
            Literal::Char(_) => Ok(Type::char()),
            Literal::Bool(_) => Ok(Type::bool()),
            Literal::Unit => Ok(Type::unit()),
        }
    }

    pub fn infer_pattern(&mut self, pat: &Pattern, ty: &Type) -> Result<Vec<(String, Type)>, TypeError> {
        match pat {
            Pattern::Wildcard { .. } => Ok(vec![]),
            Pattern::PVar { name, .. } => Ok(vec![(name.clone(), ty.clone())]),
            Pattern::PLit { value, span } => {
                let lit_ty = self.literal_type(value, *span)?;
                self.unify(ty, &lit_ty).map_err(|_| TypeError::TypeMismatch {
                    span: *span,
                    expected: lit_ty,
                    got: ty.clone(),
                })?;
                Ok(vec![])
            }
            Pattern::PCtor { name, args, span } => {
                let scheme = self.lookup_value(name).map_err(|_| TypeError::UnknownConstructor {
                    span: *span,
                    name: name.clone(),
                })?;
                let ctor_type = self.instantiate(&scheme);

                let mut current = ctor_type;
                let mut arg_types = vec![];
                while let Type::TFun { from, to } = self.prune(&current) {
                    arg_types.push((*from).clone());
                    current = (*to).clone();
                }

                self.unify(ty, &current).map_err(|_| TypeError::TypeMismatch {
                    span: *span,
                    expected: current,
                    got: ty.clone(),
                })?;

                if args.len() != arg_types.len() {
                    return Err(TypeError::TypeMismatch {
                        span: *span,
                        expected: Type::unit(),
                        got: Type::unit(),
                    });
                }

                let mut bindings = vec![];
                for (pat, arg_ty) in args.iter().zip(arg_types.iter()) {
                    bindings.extend(self.infer_pattern(pat, arg_ty)?);
                }
                Ok(bindings)
            }
            Pattern::PTuple { pats, .. } => {
                let mut types = vec![];
                for _ in pats {
                    types.push(self.fresh_tvar());
                }
                self.unify(ty, &Type::TTuple { types: types.clone() }).map_err(|_| {
                    TypeError::TypeMismatch {
                        span: pat.span(),
                        expected: ty.clone(),
                        got: Type::TTuple { types: types.clone() },
                    }
                })?;

                let mut bindings = vec![];
                for (pat, t) in pats.iter().zip(types.iter()) {
                    bindings.extend(self.infer_pattern(pat, t)?);
                }
                Ok(bindings)
            }
            Pattern::PNil { span } => {
                let elem = self.fresh_tvar();
                let list_type = Type::list(elem);
                self.unify(ty, &list_type).map_err(|_| TypeError::TypeMismatch {
                    span: *span,
                    expected: list_type,
                    got: ty.clone(),
                })?;
                Ok(vec![])
            }
            Pattern::PCons { head, tail, span } => {
                let elem = self.fresh_tvar();
                let list_type = Type::list(elem.clone());
                let list_type_clone = list_type.clone();
                self.unify(ty, &list_type).map_err(|_| TypeError::TypeMismatch {
                    span: *span,
                    expected: list_type_clone,
                    got: ty.clone(),
                })?;
                let mut bindings = vec![];
                bindings.extend(self.infer_pattern(head, &elem)?);
                bindings.extend(self.infer_pattern(tail, &list_type)?);
                Ok(bindings)
            }
        }
    }

    fn check_exhaustiveness(
        &self,
        span: Span,
        matched_type: &Type,
        arms: &[(Pattern, Expr)],
    ) -> Result<(), TypeError> {
        let pruned = self.prune(matched_type);
        match &pruned {
            Type::TCon { name, .. } => {
                if let Some((_, variants)) = self.adts.get(name) {
                    let has_wildcard = arms.iter().any(|(p, _)| matches!(p, Pattern::Wildcard { .. }));
                    if has_wildcard {
                        return Ok(());
                    }

                    for (v_name, _) in variants {
                        let covered = arms.iter().any(|(p, _)| match p {
                            Pattern::PCtor { name, .. } => name == v_name,
                            _ => false,
                        });
                        if !covered {
                            return Err(TypeError::InexhaustiveMatch { span });
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
