use crate::{EvalError, Frontend, Sexp};
use r_ust_sexp::{EnvironmentRole, InternalOperation};

#[test]
fn quit_sdk_registry_alias_uses_the_same_template() -> Result<(), EvalError> {
    let templates: Vec<_> = super::iter()?.collect();
    let quit = templates
        .iter()
        .find(|(name, _)| *name == "quit")
        .expect("quit template")
        .1;
    let q = templates
        .iter()
        .find(|(name, _)| *name == "q")
        .expect("q alias")
        .1;
    assert!(std::ptr::eq(quit, q));
    assert!(matches!(quit, Sexp::Closure { .. }));
    assert_eq!(
        crate::internal::resolve("quit"),
        Some(InternalOperation::Quit)
    );
    assert_eq!(crate::internal::resolve("q"), None);
    Ok(())
}

#[test]
fn quit_sdk_alias_preserves_compiled_handle_and_session_environment() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    frontend.eval_source("NULL")?;
    let quit = frontend.runtime().get("quit")?.expect("installed quit");
    let q = frontend.runtime().get("q")?.expect("installed q");
    assert_eq!(quit, q);
    let Sexp::Closure {
        body, environment, ..
    } = &quit
    else {
        panic!("quit closure")
    };
    let Sexp::Closure {
        body: alias_body,
        environment: alias_environment,
        ..
    } = &q
    else {
        panic!("q closure")
    };
    assert_eq!(environment, alias_environment);
    let Sexp::Bytecode(code) = body.as_ref() else {
        panic!("compiled quit body")
    };
    let Sexp::Bytecode(alias) = alias_body.as_ref() else {
        panic!("alias bytecode")
    };
    assert_eq!(code, alias);
    let namespace = frontend
        .runtime()
        .global
        .session_root(EnvironmentRole::BaseNamespace)?;
    assert_eq!(namespace.get_local("quit")?, Some(quit.clone()));
    assert_eq!(namespace.get_local("q")?, Some(q.clone()));
    super::initialize(&namespace)?;
    assert_eq!(namespace.get_local("quit")?, Some(quit));
    assert_eq!(namespace.get_local("q")?, Some(q));
    Ok(())
}
