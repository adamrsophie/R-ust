use r_ust_sexp::{
    canonical_actual_call, canonical_language, AssignmentOperator, CallArgument,
    ClosureEnvironment, Environment, EvaluationOrigin, InternalOperation, LanguageError,
    PrimitiveId, PromiseState, Sexp,
};

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}

#[test]
fn actual_calls_reuse_source_block_missing_and_assignment_conversion(
) -> Result<(), Box<dyn std::error::Error>> {
    let block = Sexp::Block(vec![Sexp::Integer(vec![Some(1)])]);
    let source = call("n", vec![block.clone(), Sexp::MissingArg("x".into())]);
    let expected = call(
        "n",
        vec![
            call("{", vec![Sexp::Integer(vec![Some(1)])]),
            Sexp::MissingValue,
        ],
    );
    assert_eq!(canonical_actual_call(&source)?, expected);
    assert_eq!(
        canonical_actual_call(&source)?,
        canonical_language(&source)?
    );
    let assignment = Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "x".into(),
        value: Box::new(block),
    };
    assert_eq!(
        canonical_actual_call(&assignment)?,
        canonical_language(&assignment)?
    );
    Ok(())
}

#[test]
fn runtime_leaves_keep_identity_without_relaxing_strict_language(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Environment::global();
    let state = PromiseState::new(
        &Sexp::Symbol("unforced".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    let closure = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Integer(vec![Some(7)])),
        environment: ClosureEnvironment::Captured(frame.clone()),
    };
    for value in [
        Sexp::Environment(frame.clone()),
        closure,
        Sexp::Primitive(PrimitiveId::Quote),
        Sexp::Promise {
            state: state.clone(),
        },
    ] {
        let source = call("nargs", vec![value]);
        assert_eq!(
            canonical_language(&source),
            Err(LanguageError::RuntimeValue)
        );
        assert_eq!(canonical_actual_call(&source)?, source);
    }
    assert_eq!(state.environment()?, Some(frame));
    assert_eq!(state.expression()?, Sexp::Symbol("unforced".into()));
    Ok(())
}

#[test]
fn actual_assignment_recursion_preserves_runtime_leaf_policy(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Environment::global();
    let source = Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "x".into(),
        value: Box::new(Sexp::Environment(frame.clone())),
    };
    assert_eq!(
        canonical_language(&source),
        Err(LanguageError::RuntimeValue)
    );
    assert_eq!(
        canonical_actual_call(&source)?,
        call(
            "<-",
            vec![Sexp::Symbol("x".into()), Sexp::Environment(frame)]
        )
    );
    Ok(())
}

#[test]
fn actual_calls_do_not_publish_internal_dispatch_language() {
    let internal = Sexp::InternalCall {
        operation: InternalOperation::Vector,
        arguments: vec![],
    };
    for source in [internal.clone(), call("f", vec![internal])] {
        assert_eq!(
            canonical_language(&source),
            Err(LanguageError::InternalCall)
        );
        assert_eq!(
            canonical_actual_call(&source),
            Err(LanguageError::InternalCall)
        );
    }
}

#[test]
fn runtime_closure_is_an_opaque_value_not_a_public_body_conversion(
) -> Result<(), Box<dyn std::error::Error>> {
    let closure = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::InternalCall {
            operation: InternalOperation::Vector,
            arguments: vec![],
        }),
        environment: ClosureEnvironment::Captured(Environment::global()),
    };
    let source = call("nargs", vec![closure]);
    assert_eq!(canonical_actual_call(&source)?, source);
    assert_eq!(
        canonical_language(&source),
        Err(LanguageError::RuntimeValue)
    );
    Ok(())
}
