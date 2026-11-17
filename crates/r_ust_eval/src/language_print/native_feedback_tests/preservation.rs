use super::{call, expected, symbol};
use crate::language_print::format_coercion;
use crate::{CallArgument, EvalError, Sexp};
use r_ust_sexp::{Environment, EvaluationOrigin, PromiseState, ValueImport, ValueObject};

#[test]
fn retained_managed_operator_head_preserves_identity_after_origins_drop() -> Result<(), EvalError> {
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &symbol("+"), ValueImport::Evaluated)?;
    let head = object.transport()?;
    let raw = Sexp::Call {
        function: Box::new(head.clone()),
        arguments: vec![
            CallArgument::positional(symbol("x")),
            CallArgument::positional(Sexp::Real(vec![2.0])),
        ],
    };
    let before = object.view()?;
    let count = object.semantic_reference_count()?;
    drop(owner);
    assert_eq!(format_coercion(&raw)?, expected("sum"));
    assert_eq!(object.view()?.header, before.header);
    assert_eq!(object.semantic_reference_count()?, count);
    assert_eq!(
        format_coercion(&call("list", vec![raw.clone(), raw]))?,
        "list(x + 2, x + 2)"
    );
    assert_eq!(object.semantic_reference_count()?, count);
    Ok(())
}

#[test]
fn managed_promise_head_inspection_does_not_force() -> Result<(), EvalError> {
    let owner = Environment::default();
    let expression = symbol("UNFORCED");
    let state = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
    let promise = Sexp::Promise {
        state: state.clone(),
    };
    let object = ValueObject::import(&owner, &promise, ValueImport::Evaluated)?;
    let raw = Sexp::Call {
        function: Box::new(Sexp::Managed(object)),
        arguments: vec![],
    };
    assert_eq!(
        format_coercion(&raw),
        Err(EvalError::NotImplemented("language printer node"))
    );
    assert_eq!(state.expression()?, expression);
    assert!(state.environment()?.is_some());
    Ok(())
}
