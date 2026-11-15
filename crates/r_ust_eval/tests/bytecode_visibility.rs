use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{
    BytecodeObject, BytecodeParts, Environment, EvaluationOrigin, PromiseState, Sexp,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn owner(frontend: &Frontend) -> Result<Environment> {
    let Some(Sexp::Environment(owner)) = frontend.runtime().get(".GlobalEnv")? else {
        panic!("global")
    };
    Ok(owner)
}

#[test]
fn bytecode_getvar_preserves_first_force_invisibility_and_resets_cached_reads() -> Result {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_bytecode_enabled(true);
    let owner = owner(&frontend)?;
    let expression = frontend
        .eval_source("quote({cat('ONCE|');invisible(3L)})")?
        .pop()
        .ok_or("expression")?;
    let state = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
    owner.insert("held".into(), &Sexp::Promise { state })?;
    let code = Sexp::Bytecode(BytecodeObject::new(
        &owner,
        BytecodeParts {
            code: vec![12, 20, 1, 1],
            constants: vec![Sexp::Symbol("held".into()), Sexp::Symbol("held".into())],
            expression: Sexp::Nil,
        },
    )?);
    assert_eq!(
        frontend.runtime_mut().eval(&code)?,
        Sexp::Integer(vec![Some(3)])
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stdout("ONCE|".into())]
    );
    assert_eq!(
        frontend.runtime_mut().eval(&code)?,
        Sexp::Integer(vec![Some(3)])
    );
    assert!(frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
    owner.insert("held".into(), &Sexp::Integer(vec![Some(4)]))?;
    assert_eq!(
        frontend.runtime_mut().eval(&code)?,
        Sexp::Integer(vec![Some(4)])
    );
    assert!(frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
    Ok(())
}

#[test]
fn compiled_symbol_body_preserves_supplied_promise_visibility() -> Result {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_bytecode_enabled(true);
    let plain = frontend
        .eval_source("function(x)x")?
        .pop()
        .ok_or("closure")?;
    let compiled = frontend
        .runtime_mut()
        .compile_closure(&plain)?
        .ok_or("compiled")?;
    owner(&frontend)?.insert("f".into(), &compiled)?;
    assert_eq!(
        frontend.eval_source("f(invisible(1L))")?.pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(
        frontend.eval_source("f(2L)")?.pop(),
        Some(Sexp::Integer(vec![Some(2)]))
    );
    assert!(frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
    Ok(())
}
