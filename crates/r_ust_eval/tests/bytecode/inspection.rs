use super::*;
use r_ust_eval::OutputEvent;

#[test]
fn source_identity_and_pointer_printing_observe_actual_body() -> Result {
    let mut frontend = Frontend::default();
    let plain = frontend.eval_source("function(x)x")?.pop().ok_or("plain")?;
    let compiled = frontend
        .runtime_mut()
        .compile_closure(&plain)?
        .ok_or("compiled")?;
    let second = frontend
        .runtime_mut()
        .compile_closure(&plain)?
        .ok_or("second")?;
    assert!(!body_code(&compiled).same_object(body_code(&second)));
    bind(&frontend, "plain", &plain)?;
    bind(&frontend, "compiled", &compiled)?;
    bind(&frontend, "second", &second)?;
    assert_eq!(
        frontend.eval_source("body(compiled)")?.pop(),
        Some(Sexp::Symbol("x".into()))
    );
    for source in [
        "identical(plain,compiled)",
        "identical(compiled,second,ignore.bytecode=FALSE)",
    ] {
        assert_eq!(
            frontend.eval_source(source)?.pop(),
            Some(Sexp::Logical(vec![Some(true)]))
        );
    }
    assert_eq!(
        frontend
            .eval_source("identical(plain,compiled,ignore.bytecode=FALSE)")?
            .pop(),
        Some(Sexp::Logical(vec![Some(false)]))
    );
    frontend.eval_source_printed("compiled")?;
    let expected = format!(
        "function (x) \nx\n<bytecode: {}>\n",
        body_code(&compiled).display_address()?
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stdout(expected)]
    );
    assert!(body_code(&compiled).same_object(body_code(
        &frontend.runtime().get("compiled")?.ok_or("alias")?
    )));
    Ok(())
}

#[test]
fn raw_bytecode_metadata_is_distinct_from_source_and_structural_identity() -> Result {
    let mut frontend = Frontend::default();
    let owner = owner(frontend.runtime())?;
    let make = |metadata| {
        BytecodeObject::new(
            &owner,
            BytecodeParts {
                code: vec![12, 16, 1, 1],
                constants: vec![
                    Sexp::Integer(vec![Some(100)]),
                    Sexp::Integer(vec![Some(42)]),
                ],
                expression: metadata,
            },
        )
    };
    let a = make(Sexp::Symbol("tag-a".into()))?;
    let b = make(Sexp::Symbol("tag-b".into()))?;
    bind(&frontend, "a", &Sexp::Bytecode(a.clone()))?;
    bind(&frontend, "b", &Sexp::Bytecode(b))?;
    assert_eq!(a.source()?, Sexp::Integer(vec![Some(100)]));
    assert_eq!(
        frontend.eval_source("identical(a,b)")?.pop(),
        Some(Sexp::Logical(vec![Some(false)]))
    );
    assert_eq!(
        frontend.eval_source("typeof(a)")?.pop(),
        Some(Sexp::String(vec![Some("bytecode".into())]))
    );
    assert_eq!(
        frontend.eval_source("class(a)")?.pop(),
        Some(Sexp::String(vec![Some("bytecode".into())]))
    );
    assert_eq!(
        frontend.eval_source("length(a)")?.pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    assert_eq!(
        frontend.eval_source("mode(a)")?.pop(),
        Some(Sexp::String(vec![Some("bytecode".into())]))
    );
    assert_eq!(
        frontend.runtime_mut().eval(&Sexp::Bytecode(a))?,
        Sexp::Integer(vec![Some(42)])
    );
    Ok(())
}

#[test]
fn captured_compiled_closure_survives_origin_runtime_and_keeps_lexical_owner() -> Result {
    let function = {
        let mut origin = Frontend::default();
        origin.eval_source("marker<-11L")?;
        compile(&mut origin, "function()marker")?
    };
    let mut receiver = Frontend::default();
    receiver.eval_source("marker<-99L")?;
    bind(&receiver, "saved", &function)?;
    drop(function);
    assert_eq!(
        receiver.eval_source("saved()")?.pop(),
        Some(Sexp::Integer(vec![Some(11)]))
    );
    Ok(())
}
