use super::LanguagePrintError;
use crate::{EvalError, ListElement, Sexp};

pub(super) fn names(
    attributes: &[ListElement],
) -> Result<Option<Vec<Option<String>>>, LanguagePrintError> {
    if let Some(value) = get(attributes, "names")? {
        return strings(&value).map(Some);
    }
    let one_dim = matches!(get(attributes, "dim")?, Some(Sexp::Integer(v)) | Some(Sexp::AttributedAtomic { values: r_ust_sexp::AtomicVector::Integer(v), .. }) if v.len() == 1);
    if !one_dim {
        return Ok(None);
    }
    let Some(value) = get(attributes, "dimnames")? else {
        return Ok(None);
    };
    let first = match &value {
        Sexp::AttributedAggregate {
            values: r_ust_sexp::AggregateVector::Vector(values),
            ..
        }
        | Sexp::Vector(values) => values.first(),
        other => other
            .ordinary_list()
            .and_then(|v| v.elements.first().map(|e| &e.value)),
    };
    match first {
        None | Some(Sexp::Nil) => Ok(None),
        Some(value) => {
            let value = match value {
                Sexp::Managed(owner) => owner.borrow().map_err(EvalError::from)?.value().clone(),
                other => other.clone(),
            };
            if matches!(value, Sexp::Nil) {
                Ok(None)
            } else {
                strings(&value).map(Some)
            }
        }
    }
}

fn get(attributes: &[ListElement], name: &str) -> Result<Option<Sexp>, LanguagePrintError> {
    attributes
        .iter()
        .find(|a| a.name.as_deref() == Some(name))
        .map(|a| match &a.value {
            Sexp::Managed(owner) => Ok(owner.borrow().map_err(EvalError::from)?.value().clone()),
            other => Ok(other.clone()),
        })
        .transpose()
}

fn strings(value: &Sexp) -> Result<Vec<Option<String>>, LanguagePrintError> {
    let value = match value {
        Sexp::Managed(owner) => owner.borrow().map_err(EvalError::from)?.value().clone(),
        other => other.clone(),
    };
    value
        .character_values()
        .map(<[_]>::to_vec)
        .ok_or(LanguagePrintError::MalformedCall)
}
