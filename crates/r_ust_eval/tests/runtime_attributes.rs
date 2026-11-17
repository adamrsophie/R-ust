use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{
    AggregateVector, AssignmentOperator, AtomicVector, Environment, ListElement, Sexp, SexpType,
    ValueImport, ValueObject,
};
#[path = "runtime_attributes/corrections.rs"]
mod corrections;
#[path = "runtime_attributes/ordinary.rs"]
mod ordinary;
#[path = "runtime_attributes/sdk.rs"]
mod sdk;
#[path = "runtime_attributes/validation.rs"]
mod validation;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn truth(frontend: &mut Frontend, source: &str) -> Result {
    assert_eq!(
        frontend.eval_source(source)?.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}"
    );
    Ok(())
}
fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}
fn attr(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn string(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}
