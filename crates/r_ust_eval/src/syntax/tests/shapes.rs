use super::*;

#[test]
fn missing_function_body_and_return_operand_are_not_self_evaluating_data() {
    let mut runtime = RuntimeState::new(Default::default());
    let function = runtime
        .eval(&call("function", vec![Sexp::Nil, Sexp::MissingValue]))
        .unwrap();
    assert_eq!(
        runtime.body(&function, &Sexp::Nil).unwrap(),
        Sexp::MissingValue
    );
    let invocation = Sexp::Call {
        function: Box::new(function),
        arguments: vec![],
    };
    assert!(runtime
        .eval(&invocation)
        .unwrap_err()
        .to_string()
        .contains("argument is missing"));
    assert!(runtime
        .eval(&call("return", vec![Sexp::MissingValue]))
        .unwrap_err()
        .to_string()
        .contains("argument is missing"));
}

#[test]
fn pinned_malformed_calls_report_selected_primitive_errors() {
    let cases = vec![
        (
            "shape_if_empty",
            call("if", vec![]),
            "argument is of length zero",
        ),
        (
            "shape_paren_empty",
            call("(", vec![]),
            "0 arguments passed to '(' which requires 1",
        ),
        (
            "shape_paren_extra",
            call("(", vec![integer(1), integer(2)]),
            "2 arguments passed to '(' which requires 1",
        ),
        (
            "shape_function_bad_formals",
            call("function", vec![integer(1), symbol("x")]),
            "invalid formal argument list for \"function\"",
        ),
        (
            "shape_function_list_formals",
            call(
                "function",
                vec![
                    Sexp::List(vec![ListElement {
                        name: Some("x".into()),
                        value: integer(1),
                    }]),
                    symbol("x"),
                ],
            ),
            "invalid formal argument list for \"function\"",
        ),
        (
            "shape_function_unnamed_formal",
            call(
                "function",
                vec![
                    Sexp::pairlist(vec![ListElement {
                        name: None,
                        value: integer(1),
                    }]),
                    symbol("x"),
                ],
            ),
            "invalid formal argument list for \"function\"",
        ),
        (
            "shape_function_no_body",
            call("function", vec![Sexp::Nil]),
            "incorrect number of arguments to \"function\"",
        ),
        (
            "shape_head_integer",
            Sexp::Call {
                function: Box::new(integer(1)),
                arguments: vec![CallArgument::positional(integer(2))],
            },
            "attempt to apply non-function",
        ),
        (
            "shape_head_string",
            Sexp::Call {
                function: Box::new(Sexp::String(vec![Some("sum".into())])),
                arguments: vec![
                    CallArgument::positional(integer(1)),
                    CallArgument::positional(integer(2)),
                ],
            },
            "attempt to apply non-function",
        ),
        (
            "shape_for_bad_variable",
            call(
                "for",
                vec![integer(1), Sexp::Integer(vec![Some(1), Some(2)]), Sexp::Nil],
            ),
            "non-symbol loop variable",
        ),
        (
            "shape_while_empty",
            call("while", vec![]),
            "0 arguments passed to 'while' which requires 2",
        ),
        (
            "shape_assign_empty",
            call("<-", vec![]),
            "incorrect number of arguments to \"<-\"",
        ),
        (
            "shape_assign_bad_lhs",
            call("<-", vec![integer(1), integer(2)]),
            "invalid (do_set) left-hand side to assignment",
        ),
        (
            "shape_plus_hole",
            call("+", vec![integer(1), Sexp::MissingValue]),
            "argument 2 is empty",
        ),
    ];
    for (id, call, message) in cases {
        let mut runtime = RuntimeState::new(Default::default());
        let error = runtime.eval(&call).unwrap_err();
        assert!(error.to_string().contains(message), "{id}: {error:?}");
    }
}

#[test]
fn pinned_permissive_shapes_are_not_guessed_strict_arities() {
    let mut runtime = RuntimeState::new(Default::default());
    assert_eq!(
        runtime.eval(&call("if", vec![logical(true)])).unwrap(),
        Sexp::Nil,
        "shape_if_one"
    );
    assert!(runtime.visible);
    assert_eq!(
        runtime
            .eval(&call(
                "if",
                vec![
                    logical(true),
                    integer(1),
                    integer(2),
                    expression("stop(\"BAD\")")
                ]
            ))
            .unwrap(),
        integer(1),
        "shape_if_extra"
    );
    let formals = Sexp::pairlist(vec![
        ListElement {
            name: Some("x".into()),
            value: integer(1),
        },
        ListElement {
            name: Some("x".into()),
            value: integer(2),
        },
    ]);
    let closure = runtime
        .eval(&call("function", vec![formals, symbol("x")]))
        .unwrap();
    let Sexp::Closure { parameters, .. } = closure else {
        panic!("shape_function_duplicate");
    };
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].name, "x");
    assert_eq!(parameters[1].name, "x");
    assert_eq!(parameters[0].default, Some(integer(1)));
    assert_eq!(parameters[1].default, Some(integer(2)));
}

#[test]
fn function_source_slots_retain_attributes_without_changing_execution() {
    for source_slot in [None, Some(Sexp::Nil)] {
        let mut runtime = RuntimeState::new(Default::default());
        let mut operands = vec![
            Sexp::pairlist(vec![ListElement {
                name: Some("x".into()),
                value: integer(1),
            }]),
            symbol("x"),
        ];
        operands.extend(source_slot);
        let function = runtime.eval(&call("function", operands)).unwrap();
        let invocation = Sexp::Call {
            function: Box::new(function),
            arguments: vec![],
        };
        assert_eq!(
            runtime.eval(&invocation).unwrap(),
            integer(1),
            "call_function_no_source / call_function_source_null"
        );
    }
    let mut runtime = RuntimeState::new(Default::default());
    let function = runtime
        .eval(&call("function", vec![Sexp::Nil, integer(1), integer(1)]))
        .unwrap();
    let Sexp::Closure { attributes, .. } = &function else {
        panic!("closure");
    };
    assert_eq!(
        attributes,
        &[ListElement {
            name: Some("srcref".into()),
            value: integer(1)
        }]
    );
    assert_eq!(
        runtime
            .eval(&Sexp::Call {
                function: Box::new(function),
                arguments: vec![]
            })
            .unwrap(),
        integer(1)
    );
}

#[test]
fn malformed_shape_fault_keeps_the_actual_tagged_call() {
    let mut runtime = RuntimeState::new(Default::default());
    let malformed = Sexp::Call {
        function: Box::new(symbol("while")),
        arguments: vec![CallArgument::named("condition", logical(false))],
    };
    let Err(EvalError::Fresh(fault)) = runtime.eval_flow_body(&malformed) else {
        panic!("expected source fault");
    };
    assert_eq!(fault.call, Ok(malformed));
    assert_eq!(
        fault.message,
        "1 argument passed to 'while' which requires 2"
    );
}
