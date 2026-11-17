mod replacements;
use replacements::{attribute_list, list, ordered};
mod dimensions;
mod normalize;
mod restore;
mod rows;
mod runtime;
mod storage;

use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

pub(crate) fn description(value: &Sexp) -> Result<Option<Sexp>, EvalError> {
    Ok(value.attribute_description()?)
}

pub(crate) fn stored(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    if let Some(frame) = super::environment::frame(value)? {
        return Ok(frame.attribute_values()?);
    }
    let Some(description) = description(value)? else {
        return Ok(Vec::new());
    };
    Ok(attributes(&description).to_vec())
}

pub(crate) fn payload(value: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::Managed(object) = value {
        return payload(object.borrow()?.value());
    }
    let Some(value) = description(value)? else {
        return Ok(value.clone());
    };
    Ok(match value {
        Sexp::AttributedAtomic { values, .. } => values.into_value(),
        Sexp::AttributedCharacter { values, .. } => Sexp::String(values),
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            ..
        } => Sexp::Vector(values),
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(values),
            ..
        } => Sexp::Expression {
            elements: values
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
            names_present: false,
        },
        other => other,
    })
}

pub(crate) fn length(value: &Sexp) -> Result<Option<usize>, EvalError> {
    let Some(value) = description(value)? else {
        return Ok(None);
    };
    Ok(match value {
        Sexp::AttributedAtomic { values, .. } => Some(values.len()),
        Sexp::AttributedAggregate { values, .. } => Some(values.len()),
        Sexp::AttributedCharacter { values, .. } => Some(values.len()),
        _ => None,
    })
}

pub(crate) fn get(value: &Sexp, name: &str) -> Result<Sexp, EvalError> {
    let attrs = stored(value)?;
    if name == "names" && dimensions::one_dimensional(&attrs)? {
        if let Some(dn) = find(&attrs, "dimnames") {
            let list = list(dn)?;
            return Ok(list
                .first()
                .map(|slot| slot.value.clone())
                .unwrap_or(Sexp::Nil));
        }
    }
    let result = find(&attrs, name).cloned().unwrap_or(Sexp::Nil);
    if name == "row.names" {
        return Ok(r_ust_sexp::expanded_row_names(&result)?);
    }
    Ok(result)
}

pub(crate) fn all(value: &Sexp) -> Result<Sexp, EvalError> {
    let mut attrs = stored(value)?;
    for attr in &mut attrs {
        if attr.name.as_deref() == Some("row.names") {
            attr.value = r_ust_sexp::expanded_row_names(&attr.value)?;
        }
    }
    Ok(if attrs.is_empty() {
        Sexp::Nil
    } else {
        Sexp::List(attrs)
    })
}

pub(crate) fn set(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    let replacement = normalize::value(value, (name, replacement))?;
    set_prepared(value, name, &replacement)
}

fn set_prepared(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    if matches!(value, Sexp::Nil) {
        return if matches!(replacement, Sexp::Nil) {
            Ok(Sexp::Nil)
        } else {
            Err(EvalError::Domain(
                "attempt to set an attribute on NULL".into(),
            ))
        };
    }
    let mut result = description(value)?.ok_or(EvalError::NotImplemented(
        "attributes on unrepresented value role",
    ))?;
    let attrs = attributes_mut(&mut result);
    if matches!(replacement, Sexp::Nil)
        && find(attrs, name).is_none()
        && !(name == "dim" && find(attrs, "dimnames").is_some())
    {
        return Ok(value.clone());
    }
    if name == "names" && !matches!(replacement, Sexp::Nil) && dimensions::one_dimensional(attrs)? {
        let dimnames = Sexp::List(vec![ListElement {
            name: None,
            value: replacement.clone(),
        }]);
        install(attrs, "dimnames", dimnames);
    } else {
        if name == "dim" {
            install(attrs, "dimnames", Sexp::Nil);
        }
        install(attrs, name, replacement.clone());
    }
    restore::value(value, result)
}

pub(crate) fn replace(value: &Sexp, replacements: &Sexp) -> Result<Sexp, EvalError> {
    let attrs = attribute_list(replacements)?;
    let value = if matches!(value, Sexp::Nil) && !matches!(replacements, Sexp::Nil) {
        Sexp::List(vec![])
    } else {
        value.clone()
    };
    let mut result = clear(&value)?;
    for attr in ordered(&attrs) {
        result = set(&result, attr.name.as_deref().unwrap_or("NA"), &attr.value)?;
    }
    Ok(result)
}

fn clear(value: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Nil) {
        return Ok(Sexp::Nil);
    }
    let mut result = description(value)?.ok_or(EvalError::NotImplemented(
        "attribute replacement on unrepresented role",
    ))?;
    attributes_mut(&mut result).clear();
    restore::value(value, result)
}

fn attributes(value: &Sexp) -> &[ListElement] {
    match value {
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => attributes,
        _ => &[],
    }
}
fn attributes_mut(value: &mut Sexp) -> &mut Vec<ListElement> {
    match value {
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => attributes,
        _ => unreachable!("attribute_description guarantees an attribute container"),
    }
}
fn find<'a>(attrs: &'a [ListElement], name: &str) -> Option<&'a Sexp> {
    attrs
        .iter()
        .find(|attr| attr.name.as_deref() == Some(name))
        .map(|attr| &attr.value)
}
fn install(attrs: &mut Vec<ListElement>, name: &str, value: Sexp) {
    if let Some(index) = attrs
        .iter()
        .position(|attr| attr.name.as_deref() == Some(name))
    {
        if matches!(value, Sexp::Nil) {
            attrs.remove(index);
        } else {
            attrs[index].value = value;
        }
    } else if !matches!(value, Sexp::Nil) {
        attrs.push(ListElement {
            name: Some(name.into()),
            value,
        });
    }
}
