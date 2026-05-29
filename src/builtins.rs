use crate::ast::{Scheme, Type};
use std::collections::HashMap;

pub struct BuiltinCtx {
    pub types: HashMap<String, Scheme>,
    pub values: HashMap<String, Scheme>,
    pub adts: HashMap<String, (Vec<String>, Vec<(String, Vec<Type>)>)>,
}

impl Default for BuiltinCtx {
    fn default() -> Self {
        let mut types = HashMap::new();
        let mut values = HashMap::new();
        let mut adts = HashMap::new();

        types.insert("Int".into(), Scheme::mono(Type::tcon("Int".into(), vec![])));
        types.insert("Float".into(), Scheme::mono(Type::tcon("Float".into(), vec![])));
        types.insert("String".into(), Scheme::mono(Type::tcon("String".into(), vec![])));
        types.insert("Bool".into(), Scheme::mono(Type::tcon("Bool".into(), vec![])));
        types.insert("Char".into(), Scheme::mono(Type::tcon("Char".into(), vec![])));
        types.insert("Unit".into(), Scheme::mono(Type::tcon("Unit".into(), vec![])));
        types.insert(
            "IO".into(),
            Scheme {
                vars: vec!["a".into()],
                ty: Type::tcon("IO".into(), vec![Type::tvar("a".into())]),
            },
        );

        add_binop(&mut values, "iadd", Type::int(), Type::int(), Type::int());
        add_binop(&mut values, "isub", Type::int(), Type::int(), Type::int());
        add_binop(&mut values, "imul", Type::int(), Type::int(), Type::int());
        add_binop(&mut values, "idiv", Type::int(), Type::int(), Type::int());
        add_binop(&mut values, "imod", Type::int(), Type::int(), Type::int());
        add_binop(&mut values, "ieq", Type::int(), Type::int(), Type::bool());
        add_binop(&mut values, "ineq", Type::int(), Type::int(), Type::bool());
        add_binop(&mut values, "ilt", Type::int(), Type::int(), Type::bool());
        add_binop(&mut values, "igt", Type::int(), Type::int(), Type::bool());
        add_binop(&mut values, "ilte", Type::int(), Type::int(), Type::bool());
        add_binop(&mut values, "igte", Type::int(), Type::int(), Type::bool());

        add_binop(&mut values, "fadd", Type::float(), Type::float(), Type::float());
        add_binop(&mut values, "fsub", Type::float(), Type::float(), Type::float());
        add_binop(&mut values, "fmul", Type::float(), Type::float(), Type::float());
        add_binop(&mut values, "fdiv", Type::float(), Type::float(), Type::float());
        add_binop(&mut values, "feq", Type::float(), Type::float(), Type::bool());
        add_binop(&mut values, "flt", Type::float(), Type::float(), Type::bool());
        add_binop(&mut values, "fgt", Type::float(), Type::float(), Type::bool());

        add_binop(&mut values, "strConcat", Type::string(), Type::string(), Type::string());
        add_binop(&mut values, "boolAnd", Type::bool(), Type::bool(), Type::bool());
        add_binop(&mut values, "boolOr", Type::bool(), Type::bool(), Type::bool());

        values.insert("boolNot".into(), Scheme::mono(Type::func(Type::bool(), Type::bool())));

        values.insert("show".into(), Scheme {
            vars: vec!["a".into()],
            ty: Type::func(Type::tvar("a".into()), Type::string()),
        });

        values.insert("print".into(), Scheme::mono(Type::func(Type::string(), Type::io(Type::unit()))));
        values.insert("println".into(), Scheme::mono(Type::func(Type::string(), Type::io(Type::unit()))));
        values.insert("readLine".into(), Scheme::mono(Type::io(Type::string())));

        let char_string_pair = Type::TTuple {
            types: vec![Type::char(), Type::string()],
        };
        values.insert(
            "strUncons".into(),
            Scheme::mono(Type::func(
                Type::string(),
                Type::tcon("Option".into(), vec![char_string_pair]),
            )),
        );

        values.insert("charEq".into(), Scheme::mono(Type::func(
            Type::char(), Type::func(Type::char(), Type::bool()),
        )));
        values.insert("charIsDigit".into(), Scheme::mono(Type::func(
            Type::char(), Type::bool(),
        )));
        values.insert("charIsSpace".into(), Scheme::mono(Type::func(
            Type::char(), Type::bool(),
        )));
        values.insert("charToInt".into(), Scheme::mono(Type::func(
            Type::char(), Type::int(),
        )));
        values.insert("intToChar".into(), Scheme::mono(Type::func(
            Type::int(), Type::char(),
        )));
        values.insert("strFromList".into(), Scheme::mono(Type::func(
            Type::tcon("List".into(), vec![Type::char()]),
            Type::string(),
        )));

        BuiltinCtx { types, values, adts }
    }
}

fn add_binop(values: &mut HashMap<String, Scheme>, name: &str, a: Type, b: Type, ret: Type) {
    values.insert(name.into(), Scheme::mono(Type::func(a, Type::func(b, ret))));
}

impl Type {
    pub fn tvar(name: String) -> Self {
        Type::TVar { name, id: 0 }
    }

    pub fn tcon(name: String, args: Vec<Type>) -> Self {
        Type::TCon { name, args }
    }
}
