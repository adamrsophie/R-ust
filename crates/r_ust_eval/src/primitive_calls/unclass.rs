use crate::{EvalError, Sexp};

// R objects.c do_unclass removes the explicit class, not implicit type classes.
pub(super) fn remove(value: &Sexp) -> Result<Sexp, EvalError> {
    if value.sexptype() == r_ust_sexp::SexpType::Environment {
        let projection = crate::ordinary_values::read(value)?;
        if let Sexp::Environment(frame) = projection.as_ref() {
            return if frame.is_object()? {
                Err(EvalError::Domain("cannot unclass an environment".into()))
            } else {
                Ok(value.clone())
            };
        }
    }
    if let Sexp::Managed(object) = value {
        let transport = object.transport()?;
        return if matches!(transport, Sexp::Managed(_)) {
            Ok(object.without_class()?.transport()?)
        } else {
            remove(&transport)
        };
    }
    let projection = crate::ordinary_values::read(value)?;
    let projected = projection.as_ref();
    if let Some(view) = projected.ordinary_list() {
        if view.class.is_none() {
            return Ok(value.clone());
        }
        let result = crate::list_attributes::set(projected, "class", &Sexp::Nil)?;
        return crate::ordinary_values::replaced(value, result);
    }
    match projected {
        Sexp::Closure { .. } if crate::closure_attributes::explicit_class(projected).is_some() => {
            crate::closure_attributes::set(projected, "class", &Sexp::Nil)
        }
        Sexp::AttributedCharacter { values, attributes }
            if attributes
                .iter()
                .any(|attribute| attribute.name.as_deref() == Some("class")) =>
        {
            let attributes = attributes
                .iter()
                .filter(|attribute| attribute.name.as_deref() != Some("class"))
                .cloned()
                .collect::<Vec<_>>();
            Ok(if attributes.is_empty() {
                Sexp::String(values.clone())
            } else {
                Sexp::AttributedCharacter {
                    values: values.clone(),
                    attributes,
                }
            })
        }
        Sexp::AttributedAtomic { values, attributes } => {
            if !has_class(attributes) {
                return Ok(value.clone());
            }
            Ok(Sexp::AttributedAtomic {
                values: values.clone(),
                attributes: without_class(attributes),
            })
        }
        Sexp::AttributedAggregate { values, attributes } => {
            if !has_class(attributes) {
                return Ok(value.clone());
            }
            Ok(Sexp::AttributedAggregate {
                values: values.clone(),
                attributes: without_class(attributes),
            })
        }
        Sexp::Factor { .. } | Sexp::DataFrame { .. } => {
            let description =
                projected
                    .attribute_description()?
                    .ok_or(EvalError::NotImplemented(
                        "unclass requires a persistent attribute description",
                    ))?;
            remove(&description)
        }
        // Plain environments have no represented OBJECT bit or class attribute.
        _ => Ok(value.clone()),
    }
}

fn has_class(attributes: &[crate::ListElement]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.name.as_deref() == Some("class"))
}

fn without_class(attributes: &[crate::ListElement]) -> Vec<crate::ListElement> {
    attributes
        .iter()
        .filter(|attribute| attribute.name.as_deref() != Some("class"))
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "unclass_tests.rs"]
mod tests;
