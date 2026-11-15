use r_ust_eval::RuntimeState;
use r_ust_sexp::{CallArgument, ListElement, Sexp};

#[test]
fn canonical_function_source_is_stored_unevaluated() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = RuntimeState::default();
    let source = Sexp::Call {
        function: Box::new(Sexp::Symbol("stop".into())),
        arguments: vec![CallArgument::positional(Sexp::String(vec![Some(
            "source forced".into(),
        )]))],
    };
    let function = runtime.eval(&Sexp::Call {
        function: Box::new(Sexp::Symbol("function".into())),
        arguments: vec![
            CallArgument::positional(Sexp::Nil),
            CallArgument::positional(Sexp::Integer(vec![Some(7)])),
            CallArgument::positional(source.clone()),
        ],
    })?;
    let Sexp::Closure { attributes, .. } = &function else {
        panic!("closure")
    };
    assert_eq!(
        attributes,
        &[ListElement {
            name: Some("srcref".into()),
            value: source
        }]
    );
    assert_eq!(
        runtime.eval(&Sexp::Call {
            function: Box::new(function),
            arguments: vec![]
        })?,
        Sexp::Integer(vec![Some(7)])
    );
    Ok(())
}
