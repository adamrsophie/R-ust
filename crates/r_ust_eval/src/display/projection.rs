use crate::{EvalError, Sexp};
use std::borrow::Cow;

pub(super) fn read(value: &Sexp) -> Result<Cow<'_, Sexp>, EvalError> {
    match value {
        Sexp::Managed(owner) => {
            let borrow = owner.borrow()?;
            match borrow.value() {
                Sexp::ListObject(list) => Ok(Cow::Owned(list.value()?)),
                Sexp::PairListObject(list) => Ok(Cow::Owned(Sexp::pairlist(list.elements()?))),
                value => Ok(Cow::Owned(value.clone())),
            }
        }
        _ => crate::ordinary_values::read(value),
    }
}

pub(super) fn check(value: &Sexp, path: &mut Vec<Sexp>) -> Result<(), EvalError> {
    let graph = matches!(
        value,
        Sexp::Managed(_) | Sexp::ListObject(_) | Sexp::PairListObject(_)
    );
    if graph {
        if path.iter().any(|prior| match (prior, value) {
            (Sexp::Managed(a), Sexp::Managed(b)) => a.same_object(b),
            (Sexp::ListObject(a), Sexp::ListObject(b)) => a.same_object(b),
            (Sexp::PairListObject(a), Sexp::PairListObject(b)) => a.same_object(b),
            _ => false,
        }) {
            return Err(EvalError::NotImplemented("cyclic ordinary value printing"));
        }
        path.push(value.clone());
    }
    let value = read(value)?;
    children(value.as_ref(), path)?;
    if graph {
        path.pop();
    }
    Ok(())
}

fn children(value: &Sexp, path: &mut Vec<Sexp>) -> Result<(), EvalError> {
    if let Some(class) = value.ordinary_list().and_then(|v| v.class) {
        check(class, path)?;
    }
    match value {
        Sexp::Managed(_) | Sexp::ListObject(_) | Sexp::PairListObject(_) => check(value, path)?,
        Sexp::AttributedAggregate { values, attributes } => {
            for value in values.values() {
                check(value, path)?;
            }
            for attribute in attributes {
                check(&attribute.value, path)?;
            }
        }
        Sexp::AttributedAtomic {
            attributes: values, ..
        }
        | Sexp::AttributedCharacter {
            attributes: values, ..
        }
        | Sexp::Closure {
            attributes: values, ..
        }
        | Sexp::Expression {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::DataFrame {
            columns: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => {
            for slot in values {
                check(&slot.value, path)?;
            }
        }
        Sexp::PairList(values) => {
            for slot in values.as_slice() {
                check(&slot.value, path)?;
            }
        }
        Sexp::Vector(values) => {
            for value in values {
                check(value, path)?;
            }
        }
        Sexp::Matrix { data, .. } => check(data, path)?,
        _ => {}
    }
    Ok(())
}
