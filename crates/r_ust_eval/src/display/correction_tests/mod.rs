mod dimensions;
mod labels;
mod summaries;
mod wrappers;

use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
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
    let transport = object.transport()?;
    let descriptor = object.borrow()?;
    drop(owner);
    for value in [&raw, &transport, descriptor.value()] {
        assert_eq!(super::format_sexp(value)?, expected(name), "direct {name}");
        assert_eq!(
            super::character::format(value, true)?,
            expected(name),
            "print.default {name}"
        );
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
