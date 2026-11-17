mod control;
mod operators;
mod preservation;

use super::format_coercion;
use crate::{CallArgument, EvalError, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};

fn symbol(name: &str) -> Sexp {
    Sexp::Symbol(name.into())
}

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(symbol(name)),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}

fn expected(name: &str) -> &str {
    include_str!("reference.txt")
        .split(&format!("CASE {name}\n"))
        .nth(1)
        .expect("reference case")
        .split("\nEND\n")
        .next()
        .expect("reference end")
}

fn check(name: &str, raw: Sexp) -> Result<(), EvalError> {
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = object.view()?;
    let references = object.semantic_reference_count()?;
    let descriptor = object.borrow()?;
    let transport = object.transport()?;
    drop(owner);
    for value in [&raw, &transport, descriptor.value()] {
        assert_eq!(format_coercion(value)?, expected(name), "{name}");
    }
    let after = object.view()?;
    assert_eq!(before.header, after.header);
    assert_eq!(before.children.len(), after.children.len());
    for (a, b) in before.children.iter().zip(&after.children) {
        assert!(a.same_object(b));
    }
    assert_eq!(object.semantic_reference_count()?, references);
    Ok(())
}
