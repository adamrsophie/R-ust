use r_ust_eval::{EvalError, Frontend, RuntimeState};
use r_ust_sexp::{
    AggregateVector, AssignmentOperator, AtomicVector, Environment, ListElement, Sexp, ValueImport,
    ValueObject,
};
#[path = "runtime_native_feedback/dataframe.rs"]
mod dataframe;
#[path = "runtime_native_feedback/extraction.rs"]
mod extraction;
#[path = "runtime_native_feedback/retention.rs"]
mod retention;
#[path = "runtime_native_feedback/validation.rs"]
mod validation;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn attr(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

fn string(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}

fn bind(f: &mut Frontend, name: &str, value: Sexp) -> Result {
    f.runtime_mut().eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}

fn truth(f: &mut Frontend, source: &str) -> Result {
    assert_eq!(
        f.eval_source(source)?.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}"
    );
    Ok(())
}

fn managed(owner: &Environment, value: &Sexp) -> Result<Sexp> {
    Ok(ValueObject::import(owner, value, ValueImport::Evaluated)?.transport()?)
}

fn get(f: &Frontend, name: &str) -> Result<Sexp> {
    Ok(f.runtime().get(name)?.ok_or("missing test binding")?)
}

fn global_environment(f: &mut Frontend) -> Result<Environment> {
    let values = f.eval_source("globalenv()")?;
    match values.last() {
        Some(Sexp::Environment(environment)) => Ok(environment.clone()),
        _ => Err("globalenv() did not return an environment".into()),
    }
}
