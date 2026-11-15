use super::*;
use r_ust_sexp::{EvaluationOrigin, Force, PromiseState};

#[test]
fn substitute_constructed_expression_and_pairlist_are_opaque(
) -> Result<(), Box<dyn std::error::Error>> {
    let addition = Sexp::Call {
        function: Box::new(Sexp::Symbol("+".into())),
        arguments: vec![
            CallArgument::positional(Sexp::Symbol("x".into())),
            CallArgument::positional(Sexp::Integer(vec![Some(1)])),
        ],
    };
    let expression = Sexp::Expression {
        elements: vec![
            ListElement {
                name: Some("a".into()),
                value: Sexp::Symbol("x".into()),
            },
            ListElement {
                name: Some(String::new()),
                value: addition.clone(),
            },
        ],
        names_present: true,
    };
    let pairlist = Sexp::pairlist(vec![
        ListElement {
            name: Some("a".into()),
            value: Sexp::Symbol("x".into()),
        },
        ListElement {
            name: Some("b".into()),
            value: addition,
        },
    ]);
    let env = Sexp::List(vec![ListElement {
        name: Some("x".into()),
        value: Sexp::Integer(vec![Some(1)]),
    }]);
    for value in [expression, pairlist] {
        assert_eq!(
            RuntimeState::default().eval(&call(value.clone(), env.clone()))?,
            value
        );
    }
    Ok(())
}

#[test]
fn substitute_uses_prexpr_even_when_cached_or_under_evaluation(
) -> Result<(), Box<dyn std::error::Error>> {
    for cached in [false, true] {
        let frame = Environment::global().child()?;
        let original = Sexp::Call {
            function: Box::new(Sexp::Symbol("stop".into())),
            arguments: vec![CallArgument::positional(Sexp::String(vec![Some(
                "BAD".into(),
            )]))],
        };
        let inner = PromiseState::new(&original, &frame, EvaluationOrigin::Global, true)?;
        let outer = PromiseState::new(
            &Sexp::Promise {
                state: inner.clone(),
            },
            &frame,
            EvaluationOrigin::Global,
            true,
        )?;
        let Force::Evaluate(lease) = inner.begin_force()? else {
            return Err("new promise".into());
        };
        let active = if cached {
            lease.complete(&Sexp::Integer(vec![Some(99)]))?;
            None
        } else {
            Some(lease)
        };
        frame.insert(
            "x".into(),
            &Sexp::Promise {
                state: outer.clone(),
            },
        )?;
        frame.insert("stop".into(), &Sexp::Integer(vec![Some(17)]))?;
        let mut runtime = RuntimeState::default();
        assert_eq!(
            runtime.eval(&call(Sexp::Symbol("x".into()), Sexp::Environment(frame)))?,
            original
        );
        assert!(runtime.take_output().is_empty());
        assert!(matches!(outer.begin_force()?, Force::Evaluate(_)));
        if cached {
            assert!(
                matches!(inner.begin_force()?, Force::Cached(Sexp::Integer(values)) if values==vec![Some(99)])
            );
        } else {
            assert!(matches!(
                inner.begin_force(),
                Err(r_ust_sexp::GraphError::PromiseUnderEvaluation)
            ));
        }
        drop(active);
    }
    Ok(())
}

#[test]
fn embedded_promise_is_walked_but_bound_promise_result_is_not(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Environment::global().child()?;
    frame.insert("y".into(), &Sexp::Integer(vec![Some(3)]))?;
    let promise = PromiseState::new(
        &Sexp::Symbol("y".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    frame.insert(
        "x".into(),
        &Sexp::Promise {
            state: promise.clone(),
        },
    )?;
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&call(
            Sexp::Promise { state: promise },
            Sexp::Environment(frame.clone())
        ))?,
        Sexp::Integer(vec![Some(3)])
    );
    assert_eq!(
        runtime.eval(&call(Sexp::Symbol("x".into()), Sexp::Environment(frame)))?,
        Sexp::Symbol("y".into())
    );
    Ok(())
}

#[test]
fn dots_binding_and_unused_nested_promises_do_not_force() -> Result<(), Box<dyn std::error::Error>>
{
    let frame = Environment::global().child()?;
    let original = Sexp::Symbol("unbound".into());
    let inner = PromiseState::new(&original, &frame, EvaluationOrigin::Global, true)?;
    let outer = PromiseState::new(
        &Sexp::Promise {
            state: inner.clone(),
        },
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    frame.insert("x".into(), &Sexp::Dots(vec![]))?;
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&call(Sexp::Symbol("x".into()), Sexp::Environment(frame))),
        Err(EvalError::Domain(
            "'...' used in an incorrect context".into()
        ))
    );
    let request = Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Substitute)),
        arguments: vec![
            CallArgument::positional(Sexp::Symbol("x".into())),
            CallArgument::positional(Sexp::Nil),
            CallArgument::named(
                "unused",
                Sexp::Promise {
                    state: outer.clone(),
                },
            ),
        ],
    };
    assert_eq!(
        runtime.eval(&request),
        Err(EvalError::Arity(
            "unused argument (unused = unbound)".into()
        ))
    );
    assert!(matches!(inner.begin_force()?, Force::Evaluate(_)));
    assert!(matches!(outer.begin_force()?, Force::Evaluate(_)));
    assert!(runtime.take_output().is_empty());
    Ok(())
}

#[test]
fn dots_splice_extracts_original_expression_with_lookup_disabled(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Environment::global().child()?;
    frame.insert("x".into(), &Sexp::Integer(vec![Some(7)]))?;
    frame.insert("g".into(), &Sexp::Symbol("h".into()))?;
    let promise = PromiseState::new(
        &Sexp::Symbol("x".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    frame.insert(
        "...".into(),
        &Sexp::Dots(vec![ListElement {
            name: Some("a".into()),
            value: Sexp::Promise {
                state: promise.clone(),
            },
        }]),
    )?;
    let input = Sexp::Call {
        function: Box::new(Sexp::Symbol("g".into())),
        arguments: vec![CallArgument::positional(Sexp::Symbol("...".into()))],
    };
    let expected = Sexp::Call {
        function: Box::new(Sexp::Symbol("h".into())),
        arguments: vec![CallArgument::named("a", Sexp::Symbol("x".into()))],
    };
    assert_eq!(
        RuntimeState::default().eval(&call(input, Sexp::Environment(frame)))?,
        expected
    );
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
