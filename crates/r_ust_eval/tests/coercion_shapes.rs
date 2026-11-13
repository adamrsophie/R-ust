use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{is_na_real, CallArgument, ListElement, Sexp};

fn text(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}

fn list(values: Vec<Sexp>) -> Sexp {
    Sexp::List(
        values
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
    )
}

fn convert(frontend: &mut Frontend, name: &str, value: Sexp) -> Result<Sexp, EvalError> {
    frontend.runtime_mut().eval(&Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: vec![CallArgument::positional(value)],
    })
}

#[test]
fn numeric_shapes_drop_structure_and_preserve_element_order() {
    let atomic = Sexp::Integer(vec![Some(2), Some(1)]);
    for input in [
        atomic.clone(),
        Sexp::Real(vec![2.0, 1.0]),
        Sexp::String(vec![Some("2".into()), Some("1".into())]),
        Sexp::Matrix {
            data: Box::new(atomic.clone()),
            nrow: 1,
            ncol: 2,
        },
        Sexp::Factor {
            codes: vec![Some(2), Some(1)],
            levels: vec!["a".into(), "b".into()],
            ordered: false,
        },
        list(vec![text("2"), text("1")]),
        Sexp::Vector(vec![text("2"), text("1")]),
        Sexp::DataFrame {
            columns: vec![
                ListElement {
                    name: Some("a".into()),
                    value: text("2"),
                },
                ListElement {
                    name: Some("b".into()),
                    value: text("1"),
                },
            ],
            nrow: 1,
        },
    ] {
        for (name, expected) in [
            ("as.double", Sexp::Real(vec![2.0, 1.0])),
            ("as.integer", atomic.clone()),
        ] {
            let mut frontend = Frontend::default();
            assert_eq!(
                convert(&mut frontend, name, input.clone()).unwrap(),
                expected,
                "{name}: {input:?}"
            );
            assert!(frontend.runtime_mut().take_output().is_empty());
        }
    }
    for name in ["as.double", "as.integer"] {
        let mut frontend = Frontend::default();
        let expected = if name == "as.integer" {
            Sexp::Integer(vec![Some(1), Some(0), None])
        } else {
            Sexp::Real(vec![1.0, 0.0, r_ust_sexp::NA_REAL])
        };
        let actual = convert(
            &mut frontend,
            name,
            Sexp::Logical(vec![Some(true), Some(false), None]),
        )
        .unwrap();
        if let (Sexp::Real(values), Sexp::Real(expected)) = (&actual, &expected) {
            assert_eq!(
                values.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
        } else {
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn null_and_omitted_arguments_return_typed_empty_vectors() {
    // as.real is an existing project alias; R 4.6.0 only supplies the other names.
    for name in ["as.numeric", "as.double", "as.real", "as.integer"] {
        let expected = if name == "as.integer" {
            Sexp::Integer(vec![])
        } else {
            Sexp::Real(vec![])
        };
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(&format!("{name}()")).unwrap(),
            vec![expected.clone()]
        );
        assert_eq!(convert(&mut frontend, name, Sexp::Nil).unwrap(), expected);
    }
}

#[test]
fn explicit_empty_arguments_are_not_omitted_defaults() {
    for name in ["as.numeric", "as.double", "as.real", "as.integer"] {
        for (arguments, index, printed) in [
            ("x=", 1, false),
            (",", 1, false),
            ("x=,print(9)", 1, false),
            ("print(9),", 2, true),
        ] {
            let mut frontend = Frontend::default();
            let error = frontend
                .eval_source(&format!("{name}({arguments})"))
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains(&format!("argument {index} is empty")),
                "{error}"
            );
            let expected = if printed {
                vec![OutputEvent::Stdout("[1] 9\n".into())]
            } else {
                vec![]
            };
            assert_eq!(frontend.runtime_mut().take_output(), expected);
        }
    }
}

#[test]
fn empty_atomic_and_short_nested_list_elements_are_missing_without_warnings() {
    let input = list(vec![
        Sexp::Integer(vec![]),
        Sexp::Real(vec![]),
        Sexp::String(vec![]),
        Sexp::Logical(vec![]),
        list(vec![]),
        list(vec![text("bad")]),
        list(vec![list(vec![text("bad")])]),
    ]);
    for name in ["as.double", "as.integer"] {
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=2)").unwrap();
        let result = convert(&mut frontend, name, input.clone()).unwrap();
        if name == "as.integer" {
            assert_eq!(result, Sexp::Integer(vec![None; 7]));
        } else {
            let Sexp::Real(values) = result else {
                panic!("expected real vector")
            };
            assert_eq!(values.len(), 7);
            assert!(values.into_iter().all(is_na_real));
        }
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}

#[test]
fn invalid_lists_fail_before_any_conversion_warning() {
    for invalid in [
        Sexp::Nil,
        Sexp::Integer(vec![Some(1), Some(2)]),
        list(vec![text("1"), text("2")]),
    ] {
        for name in ["as.double", "as.integer"] {
            for warn in [-1, 0, 1, 2, 3] {
                for input in [
                    list(vec![text("bad"), invalid.clone()]),
                    Sexp::Vector(vec![text("bad"), invalid.clone()]),
                    Sexp::DataFrame {
                        columns: vec![
                            ListElement {
                                name: Some("bad".into()),
                                value: text("bad"),
                            },
                            ListElement {
                                name: Some("invalid".into()),
                                value: invalid.clone(),
                            },
                        ],
                        nrow: 1,
                    },
                ] {
                    let mut frontend = Frontend::default();
                    frontend
                        .runtime_mut()
                        .set_option("warn", Sexp::Integer(vec![Some(warn)]))
                        .unwrap();
                    let result = convert(&mut frontend, name, input);
                    let error = frontend
                        .runtime_mut()
                        .finish_parse_warnings(result)
                        .unwrap_err();
                    let target = if name == "as.integer" {
                        "integer"
                    } else {
                        "double"
                    };
                    assert!(
                        error.to_string().contains(&format!(
                            "'list' object cannot be coerced to type '{target}'"
                        )),
                        "{name}; warn={warn}: {error}"
                    );
                    assert!(frontend.runtime_mut().take_output().is_empty());
                }
            }
        }
    }
}

#[test]
fn numeric_arguments_require_x_and_eagerly_evaluate_ignored_trailing_values() {
    for name in ["as.numeric", "as.double", "as.real", "as.integer"] {
        let mut frontend = Frontend::default();
        let error = frontend
            .eval_source(&format!("{name}(y=1,print(9))"))
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("supplied argument name 'y' does not match 'x'"));
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("[1] 9\n".into())]
        );
        let expected = if name == "as.integer" {
            Sexp::Integer(vec![Some(1)])
        } else {
            Sexp::Real(vec![1.0])
        };
        assert_eq!(
            frontend
                .eval_source(&format!("{name}(x=1,print(9),ignored=list(1,2))"))
                .unwrap(),
            vec![expected]
        );
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("[1] 9\n".into())]
        );
        assert_eq!(
            frontend
                .eval_source(&format!("{name}(1,missingName)"))
                .unwrap_err(),
            EvalError::UndefinedSymbol("missingName".into())
        );
    }
}
