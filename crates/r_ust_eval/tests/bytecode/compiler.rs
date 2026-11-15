use super::*;
use r_ust_sexp::{ClosureEnvironment, PrimitiveId};

#[test]
fn real_base_eval_is_compiled_by_shape_with_binding_guard_and_getvar() -> Result {
    let runtime = RuntimeState::default();
    assert_eq!(
        runtime.get(".Internal")?,
        Some(Sexp::Primitive(PrimitiveId::Internal))
    );
    let eval = runtime
        .get("eval")?
        .ok_or("incoming genuine D3 eval closure")?;
    let parts = body_code(&eval).parts()?;
    assert_eq!(parts.code[0], 12);
    assert_eq!(parts.code[1], 123);
    let names = parts
        .code
        .windows(2)
        .filter(|pair| pair[0] == 20)
        .map(|pair| parts.constants[pair[1] as usize].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            Sexp::Symbol("expr".into()),
            Sexp::Symbol("envir".into()),
            Sexp::Symbol("enclos".into())
        ]
    );
    assert_eq!(parts.code.last(), Some(&1));
    assert_eq!(parts.expression, Sexp::Nil);
    let Sexp::Call { function, .. } = &parts.constants[0] else {
        panic!("original source")
    };
    assert_eq!(function.as_ref(), &Sexp::Symbol(".Internal".into()));
    for name in ["identity", "force"] {
        let function = runtime.get(name)?.ok_or("base closure")?;
        assert_eq!(body_code(&function).parts()?.code, vec![12, 20, 1, 1]);
        assert_eq!(body_code(&function).source()?, Sexp::Symbol("x".into()));
    }
    Ok(())
}

#[test]
fn renamed_closures_compile_and_unsupported_grammar_stays_explicit() -> Result {
    let mut frontend = Frontend::default();
    let function = compile(&mut frontend, "function(unrelated) unrelated")?;
    bind(&frontend, "renamed", &function)?;
    assert_eq!(
        frontend.eval_source("renamed(41L)")?.pop(),
        Some(Sexp::Integer(vec![Some(41)]))
    );
    for source in [
        "function(x)x+1L",
        "function(...)..1",
        "function(x)list(x)",
        "function(x){x}",
    ] {
        let original = frontend.eval_source(source)?.pop().ok_or("closure")?;
        assert!(
            frontend.runtime_mut().compile_closure(&original)?.is_none(),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn actual_internal_rebinding_uses_ast_without_duplicate_argument_effects() -> Result {
    let mut frontend = Frontend::default();
    let mut function = compile(&mut frontend, "function(x) .Internal(body(x))")?;
    let environment = owner(frontend.runtime())?.child()?;
    let override_function = frontend
        .eval_source("function(call) {cat('GUARD|'); 77L}")?
        .pop()
        .ok_or("override")?;
    environment.insert(".Internal".into(), &override_function)?;
    let Sexp::Closure {
        environment: capture,
        ..
    } = &mut function
    else {
        panic!("closure")
    };
    *capture = ClosureEnvironment::Captured(environment.clone());
    bind(&frontend, "guarded", &function)?;
    assert_eq!(
        frontend
            .eval_source("guarded({cat('ARG|');function()1L})")?
            .pop(),
        Some(Sexp::Integer(vec![Some(77)]))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![r_ust_eval::OutputEvent::Stdout("GUARD|".into())]
    );
    environment.insert(".Internal".into(), &Sexp::Integer(vec![Some(5)]))?;
    assert_eq!(
        frontend
            .eval_source("guarded({cat('ARG|');function()1L})")?
            .pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![r_ust_eval::OutputEvent::Stdout("ARG|".into())]
    );
    Ok(())
}

#[test]
fn do_call_rejects_both_internal_name_and_actual_primitive() -> Result {
    let mut frontend = Frontend::default();
    for source in [
        "do.call('.Internal',list())",
        "do.call(.Primitive('.Internal'),list())",
    ] {
        assert_eq!(
            frontend.eval_source(source),
            Err(r_ust_eval::EvalError::Domain("illegal usage".into()))
        );
    }
    assert_eq!(
        frontend
            .eval_source("typeof(.Primitive('.Internal'))")?
            .pop(),
        Some(Sexp::String(vec![Some("special".into())]))
    );
    Ok(())
}
