use super::*;

#[test]
fn syntax_primitive_type_and_print_bytes_match_pinned_r() {
    let expected =
        include_str!("../../../tests/fixtures/executable_language/syntax-primitives.stdout");
    let mut runtime = RuntimeState::new(Default::default());
    let mut actual = String::new();
    for header in expected
        .lines()
        .filter(|line| line.starts_with("PRIMITIVE "))
    {
        let name = header.split_whitespace().nth(1).unwrap();
        let value = runtime
            .eval(&call(
                ".Primitive",
                vec![Sexp::String(vec![Some(name.into())])],
            ))
            .unwrap();
        actual.push_str(&format!(
            "PRIMITIVE {name} {} \n{}\n",
            crate::typeof_value(&value),
            crate::format_sexp(&value).unwrap()
        ));
    }
    assert_eq!(actual, expected);
}

#[test]
fn syntax_catalogue_matches_pinned_special_and_builtin_flags() {
    let specials = [
        "if", "while", "for", "repeat", "break", "next", "function", "<-", "=", "<<-", "{", "[",
        "[[", "$", "[<-", "[[<-", "$<-", "&&", "||",
    ];
    let builtins = [
        "(",
        "names<-",
        "class<-",
        "attributes<-",
        "attr<-",
        "&",
        "|",
        "!",
        "==",
        "!=",
        "<",
        "<=",
        ">",
        ">=",
    ];
    let mut runtime = RuntimeState::new(Default::default());
    for (names, kind) in [
        (&specials[..], r_ust_sexp::SexpType::Special),
        (&builtins[..], r_ust_sexp::SexpType::Builtin),
    ] {
        for name in names {
            let id = PrimitiveId::from_name(name).unwrap();
            assert_eq!(id.name(), *name);
            assert_eq!(id.sexptype(), kind, "{name}");
            assert_eq!(
                runtime
                    .eval(&call(
                        ".Primitive",
                        vec![Sexp::String(vec![Some((*name).into())])]
                    ))
                    .unwrap(),
                Sexp::Primitive(id)
            );
        }
    }
    assert!(PrimitiveId::from_name("->").is_none());
    assert!(PrimitiveId::from_name("->>").is_none());
}

#[test]
fn canonical_functions_keep_runtime_body_and_default_roots_for_inspection() {
    let mut runtime = RuntimeState::new(Default::default());
    let environment = Sexp::Environment(runtime.current.child().unwrap());
    let formals = Sexp::pairlist(vec![ListElement {
        name: Some("x".into()),
        value: environment.clone(),
    }]);
    let function = runtime
        .eval(&call(
            "function",
            vec![formals.clone(), environment.clone()],
        ))
        .unwrap();
    assert_eq!(runtime.body(&function, &Sexp::Nil).unwrap(), environment);
    assert_eq!(runtime.formals(&function, &Sexp::Nil).unwrap(), formals);
    assert!(canonical_language(&environment).is_err());
    let invocation = Sexp::Call {
        function: Box::new(function),
        arguments: vec![],
    };
    assert_eq!(runtime.eval(&invocation).unwrap(), environment);
    let closure = source(&mut runtime, "function()1L", false);
    for leaf in [closure, Sexp::Primitive(PrimitiveId::Sum)] {
        let formals = Sexp::pairlist(vec![ListElement {
            name: Some("x".into()),
            value: leaf.clone(),
        }]);
        let function = runtime
            .eval(&call("function", vec![formals.clone(), symbol("x")]))
            .unwrap();
        assert_eq!(runtime.formals(&function, &Sexp::Nil).unwrap(), formals);
        let invocation = Sexp::Call {
            function: Box::new(function),
            arguments: vec![],
        };
        assert_eq!(runtime.eval(&invocation).unwrap(), leaf);
        let bad_body = call("function", vec![Sexp::Nil, leaf]);
        assert!(runtime
            .eval(&bad_body)
            .unwrap_err()
            .to_string()
            .contains("invalid body argument for 'function'"));
    }
}

#[test]
fn pinned_primitive_closure_and_computed_heads_run_once() {
    let mut runtime = RuntimeState::new(Default::default());
    let primitive = Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Add)),
        arguments: vec![
            CallArgument::positional(integer(2)),
            CallArgument::positional(integer(3)),
        ],
    };
    assert_eq!(
        runtime.eval(&primitive).unwrap(),
        integer(5),
        "call_primitive_head"
    );
    let closure = source(&mut runtime, "function(x)x+1L", false);
    let closure_call = Sexp::Call {
        function: Box::new(closure),
        arguments: vec![CallArgument::positional(integer(8))],
    };
    assert_eq!(
        runtime.eval(&closure_call).unwrap(),
        integer(9),
        "call_closure_head"
    );
    source(&mut runtime, "n<-0L", false);
    let head = canonical_language(&expression("{n<-n+1L;function(x)x}")).unwrap();
    let computed = Sexp::Call {
        function: Box::new(head),
        arguments: vec![CallArgument::positional(integer(4))],
    };
    assert_eq!(
        runtime.eval(&computed).unwrap(),
        integer(4),
        "call_head_once result"
    );
    assert_eq!(
        runtime.lookup("n").unwrap(),
        Some(integer(1)),
        "call_head_once side effect"
    );
}

#[test]
fn selected_primitive_identity_does_not_resolve_its_name_again() {
    let mut runtime = RuntimeState::new(Default::default());
    let replacement = source(&mut runtime, "function(...) stop(\"REBIND\")", false);
    runtime.current.insert("if".into(), &replacement).unwrap();
    let invocation = Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Syntax(SyntaxPrimitive::If))),
        arguments: vec![
            CallArgument::positional(logical(true)),
            CallArgument::positional(integer(9)),
        ],
    };
    assert_eq!(runtime.eval(&invocation).unwrap(), integer(9));
}

#[test]
fn raw_adapter_keeps_internal_bodies_and_unselected_children() {
    let mut runtime = RuntimeState::new(Default::default());
    let body = Sexp::InternalCall {
        operation: r_ust_sexp::InternalOperation::SysNFrame,
        arguments: vec![],
    };
    let function = Sexp::Function {
        parameters: vec![],
        body: Box::new(body.clone()),
    };
    assert!(canonical_language(&function).is_err());
    let closure = runtime.eval(&function).unwrap();
    let Sexp::Closure { body: actual, .. } = &closure else {
        panic!("closure");
    };
    assert_eq!(actual.as_ref(), &body);
    let invocation = Sexp::Call {
        function: Box::new(closure),
        arguments: vec![],
    };
    // Direct internal inspection selects the caller; no public wrapper adds a frame.
    assert_eq!(runtime.eval(&invocation).unwrap(), integer(0));
    let choice = Sexp::If {
        condition: Box::new(logical(true)),
        then_branch: Box::new(integer(4)),
        else_branch: Some(Box::new(body)),
    };
    assert_eq!(runtime.eval(&choice).unwrap(), integer(4));
}

#[test]
fn canonical_parenthesized_legacy_heads_keep_existing_lookup() {
    for canonical in [false, true] {
        let mut runtime = RuntimeState::new(Default::default());
        let result = source(&mut runtime, "((c))(1L,2L)", canonical);
        assert_eq!(result, Sexp::Integer(vec![Some(1), Some(2)]));
        let rebound = source(&mut runtime, "function(x) function(...) 82L", false);
        runtime.current.insert("(".into(), &rebound).unwrap();
        assert_eq!(
            source(&mut runtime, "(stop(\"BAD\"))(1L)", canonical),
            integer(82)
        );
    }
}

#[test]
fn parenthesis_visibility_follows_the_selected_identity() {
    for canonical in [false, true] {
        let mut runtime = RuntimeState::new(Default::default());
        assert_eq!(
            source(&mut runtime, "(invisible(1L))", canonical),
            integer(1)
        );
        assert!(runtime.visible);
        let replacement = source(&mut runtime, "function(x) invisible(8L)", false);
        runtime.current.insert("(".into(), &replacement).unwrap();
        assert_eq!(
            source(&mut runtime, "(stop(\"BAD\"))", canonical),
            integer(8)
        );
        assert!(!runtime.visible);
    }
}

#[test]
fn replacement_heads_and_tagged_arguments_remain_callable() {
    for canonical in [false, true] {
        let mut runtime = RuntimeState::new(Default::default());
        source(
            &mut runtime,
            "x<-list(a=1L);`$<-`<-function(x,name,value) list(done=value)",
            false,
        );
        let result = source(&mut runtime, "x$a<-9L;x$done", canonical);
        assert_eq!(result, integer(9));
    }
    let mut runtime = RuntimeState::new(Default::default());
    let replacement = source(&mut runtime, "function(value,target) value", false);
    runtime.current.insert("<-".into(), &replacement).unwrap();
    let tagged = Sexp::Call {
        function: Box::new(symbol("<-")),
        arguments: vec![
            CallArgument::named("value", integer(8)),
            CallArgument::named("target", expression("stop(\"BAD\")")),
        ],
    };
    assert_eq!(runtime.eval(&tagged).unwrap(), integer(8));
}
