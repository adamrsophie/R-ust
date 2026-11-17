use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{ClosureEnvironment, FunctionParameter, ValueImport, ValueObject};

#[test]
fn managed_closure_formals_preserve_defaults_without_evaluation() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let closure = Sexp::Closure {
        parameters: vec![
            FunctionParameter::required("x"),
            FunctionParameter::optional("y", Sexp::Symbol("never_evaluate".into())),
        ],
        body: Box::new(Sexp::Symbol("never_run".into())),
        environment: ClosureEnvironment::Captured(runtime.global.clone()),
        attributes: vec![],
    };
    let handle = ValueObject::import(&runtime.global, &closure, ValueImport::Code)?;
    let call = Sexp::Symbol("formals".into());
    let expected = runtime.formals(&closure, &call)?.snapshot()?;
    let actual = runtime.formals(&Sexp::Managed(handle), &call)?.snapshot()?;
    assert_eq!(actual, expected);
    let Sexp::PairList(parameters) = actual else {
        panic!("formal pairlist")
    };
    assert_eq!(parameters.as_slice()[0].value, Sexp::MissingValue);
    assert_eq!(
        parameters.as_slice()[1].value,
        Sexp::Symbol("never_evaluate".into())
    );
    Ok(())
}
