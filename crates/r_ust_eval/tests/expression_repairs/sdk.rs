use super::*;

#[test]
fn raw_sdk_call_heads_are_captured_without_normalization_or_execution(
) -> Result<(), Box<dyn std::error::Error>> {
    for head in [
        vec![Some("f".into())],
        vec![Some("f".into()), Some("g".into())],
        vec![],
        vec![None],
    ] {
        let payload = Sexp::Call {
            function: Box::new(Sexp::String(head.clone())),
            arguments: vec![CallArgument::positional(Sexp::Symbol("unbound".into()))],
        };
        let mut runtime = RuntimeState::default();
        let captured = runtime.eval(&Sexp::Call {
            function: Box::new(Sexp::Primitive(PrimitiveId::Expression)),
            arguments: vec![CallArgument::positional(payload.clone())],
        })?;
        assert_eq!(
            captured,
            Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: payload.clone()
                }],
                names_present: false,
            }
        );
        assert_eq!(canonical_actual_call(&payload)?, payload);
        assert!(runtime.take_output().is_empty());
        if head.len() == 1 && head[0].is_some() {
            assert_eq!(
                canonical_language(&payload)?,
                Sexp::Call {
                    function: Box::new(Sexp::Symbol("f".into())),
                    arguments: vec![CallArgument::positional(Sexp::Symbol("unbound".into()))],
                }
            );
        } else {
            assert_eq!(
                canonical_language(&payload),
                Err(LanguageError::RuntimeValue)
            );
        }
    }
    Ok(())
}

#[test]
fn capture_rejects_internal_dispatch_and_strict_language_rejects_runtime_values() {
    let internal = Sexp::InternalCall {
        operation: r_ust_sexp::InternalOperation::IsVector,
        arguments: vec![],
    };
    assert_eq!(
        canonical_actual_call(&internal),
        Err(LanguageError::InternalCall)
    );
    assert_eq!(
        canonical_language(&internal),
        Err(LanguageError::InternalCall)
    );
    let value = Sexp::Environment(r_ust_sexp::Environment::global());
    assert_eq!(canonical_language(&value), Err(LanguageError::RuntimeValue));
    assert_eq!(canonical_actual_call(&value), Ok(value));
}

#[test]
fn parser_normalizes_only_direct_literal_string_heads() -> Result<(), Box<dyn std::error::Error>> {
    for (source, expected) in [
        ("'f'(x)", Sexp::Symbol("f".into())),
        (
            "('f')(x)",
            Sexp::Parenthesized(Box::new(Sexp::String(vec![Some("f".into())]))),
        ),
    ] {
        let expressions = r_ust_parse::Parser
            .parse_all(source)
            .map_err(|error| error.message)?
            .expressions;
        assert_eq!(
            expressions,
            vec![Sexp::Call {
                function: Box::new(expected),
                arguments: vec![CallArgument::positional(Sexp::Symbol("x".into()))],
            }]
        );
    }
    Ok(())
}
