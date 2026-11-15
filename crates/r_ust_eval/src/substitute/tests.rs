use super::*;

#[test]
fn source_fault_retains_actual_alias_call_and_current_frame(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = RuntimeState::default();
    runtime.current = runtime.current.child()?;
    let current = runtime.current.clone();
    let arguments = vec![
        CallArgument::positional(Sexp::Symbol("x".into())),
        CallArgument::positional(Sexp::Integer(vec![Some(1)])),
    ];
    let actual = Sexp::Call {
        function: Box::new(Sexp::Symbol("alias".into())),
        arguments: arguments.clone(),
    };
    let Err(EvalError::Fresh(fault)) = runtime.eval_substitute(&arguments, &actual) else {
        return Err("fresh source fault".into());
    };
    assert_eq!(fault.message, "invalid environment specified");
    assert_eq!(fault.call, Ok(actual));
    assert_eq!(
        *fault.original,
        EvalError::Type("invalid environment specified".into())
    );
    assert_eq!(runtime.current, current);
    assert!(runtime.take_output().is_empty());
    Ok(())
}
