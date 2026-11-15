use super::*;
use r_ust_sexp::{
    canonical_actual_call, canonical_language, ClosureEnvironment, EvaluationEnvironmentId,
    EvaluationOrigin, Force, LanguageError, PromiseState,
};

fn quote(value: Sexp) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Quote)),
        arguments: vec![CallArgument::positional(value)],
    }
}

#[test]
fn source_and_sdk_string_heads_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    for names in [
        vec![Some("f".into())],
        vec![Some("f".into()), Some("g".into())],
    ] {
        let raw = Sexp::Call {
            function: Box::new(Sexp::String(names)),
            arguments: vec![CallArgument::positional(Sexp::Symbol("x".into()))],
        };
        assert_eq!(canonical_actual_call(&raw)?, raw);
        let mut runtime = RuntimeState::default();
        assert_eq!(runtime.eval(&quote(raw.clone()))?, raw);
        let env = Sexp::List(vec![
            ListElement {
                name: Some("f".into()),
                value: Sexp::Symbol("g".into()),
            },
            ListElement {
                name: Some("x".into()),
                value: Sexp::Integer(vec![Some(1)]),
            },
        ]);
        let Sexp::Call {
            function,
            arguments,
        } = runtime.eval(&call(raw.clone(), env))?
        else {
            return Err("call".into());
        };
        let Sexp::Call {
            function: expected, ..
        } = raw
        else {
            return Err("raw call".into());
        };
        assert_eq!(function, expected);
        assert_eq!(
            arguments,
            vec![CallArgument::positional(Sexp::Integer(vec![Some(1)]))]
        );
    }
    for source in ["('f')(stop('BAD'))", "(identity('f'))(stop('BAD'))"] {
        assert_eq!(
            Frontend::default().eval_source(source),
            Err(EvalError::Type("attempt to apply non-function".into()))
        );
    }
    let scalar = Sexp::Call {
        function: Box::new(Sexp::String(vec![Some("f".into())])),
        arguments: vec![],
    };
    assert_eq!(
        canonical_language(&scalar)?,
        Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![]
        }
    );
    Ok(())
}

#[test]
fn quote_preserves_runtime_identity_lifetime_and_promise_control_origin(
) -> Result<(), Box<dyn std::error::Error>> {
    let origin = EvaluationOrigin::Frame(EvaluationEnvironmentId(37));
    let captured = {
        let frame = Environment::global().child()?;
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        let closure = Sexp::Closure {
            attributes: Vec::new(),
            parameters: vec![],
            body: Box::new(Sexp::Symbol("marker".into())),
            environment: ClosureEnvironment::Captured(frame.clone()),
        };
        let promise = PromiseState::new(&closure, &frame, origin.clone(), true)?;
        let raw = Sexp::Call {
            function: Box::new(closure),
            arguments: vec![
                CallArgument::named("env", Sexp::Environment(frame)),
                CallArgument::named("promise", Sexp::Promise { state: promise }),
            ],
        };
        assert_eq!(canonical_language(&raw), Err(LanguageError::RuntimeValue));
        let mut runtime = RuntimeState::default();
        let result = runtime.eval(&quote(raw.clone()))?;
        assert_eq!(result, raw);
        assert!(runtime.take_output().is_empty());
        result
    };
    let Sexp::Call {
        function,
        arguments,
    } = &captured
    else {
        return Err("call".into());
    };
    let Sexp::Environment(frame) = &arguments[0].value else {
        return Err("environment".into());
    };
    assert_eq!(
        frame.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    let Sexp::Promise { state } = &arguments[1].value else {
        return Err("promise".into());
    };
    let Force::Evaluate(lease) = state.begin_force()? else {
        return Err("forced during quote".into());
    };
    assert!(!lease.interrupted);
    assert_eq!(lease.origin, origin);
    assert_eq!(lease.expression, function.as_ref().clone());
    assert_eq!(&lease.environment, frame);
    let mut consumer = RuntimeState::default();
    assert_eq!(
        consumer.eval(&Sexp::Call {
            function: function.clone(),
            arguments: vec![]
        })?,
        Sexp::Integer(vec![Some(42)])
    );
    drop(consumer);
    assert_eq!(state.expression()?, function.as_ref().clone());
    Ok(())
}
