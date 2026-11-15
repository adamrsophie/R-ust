use super::*;
use r_ust_sexp::{canonical_actual_call, canonical_language, ClosureEnvironment, LanguageError};

fn wrap(value: Sexp) -> Sexp {
    Sexp::Expression {
        elements: vec![ListElement { name: None, value }],
        names_present: false,
    }
}

#[test]
fn expression_self_evaluation_does_not_sequence_sdk_payloads(
) -> Result<(), Box<dyn std::error::Error>> {
    let expression = wrap(Sexp::Call {
        function: Box::new(Sexp::Symbol("stop".into())),
        arguments: vec![CallArgument::positional(Sexp::String(vec![Some(
            "BAD".into(),
        )]))],
    });
    let mut runtime = RuntimeState::default();
    assert_eq!(runtime.eval(&expression)?, expression);
    assert!(runtime.take_output().is_empty());
    Ok(())
}

#[test]
fn expression_capture_preserves_runtime_leaves_and_source_boundary(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Environment::global();
    frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
    let payload = wrap(Sexp::Call {
        function: Box::new(Sexp::Closure {
            attributes: Vec::new(),
            parameters: vec![],
            body: Box::new(Sexp::Symbol("marker".into())),
            environment: ClosureEnvironment::Captured(frame.clone()),
        }),
        arguments: vec![CallArgument::positional(Sexp::Environment(frame.clone()))],
    });
    assert_eq!(
        canonical_language(&payload),
        Err(LanguageError::RuntimeValue)
    );
    assert_eq!(canonical_actual_call(&payload)?, payload);
    let mut runtime = RuntimeState::default();
    let result = runtime.eval(&Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Expression)),
        arguments: vec![CallArgument::positional(payload.clone())],
    })?;
    assert_eq!(result, wrap(payload));
    assert!(runtime.take_output().is_empty());
    drop(runtime);
    drop(frame);
    let Sexp::Expression {
        elements: outer, ..
    } = result
    else {
        return Err("outer expression".into());
    };
    let Sexp::Expression { elements, .. } = &outer[0].value else {
        return Err("inner expression".into());
    };
    let Sexp::Call {
        function,
        arguments,
    } = &elements[0].value
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
    let mut destination = RuntimeState::default();
    assert_eq!(
        destination.eval(&Sexp::Call {
            function: function.clone(),
            arguments: vec![],
        })?,
        Sexp::Integer(vec![Some(42)])
    );
    Ok(())
}
