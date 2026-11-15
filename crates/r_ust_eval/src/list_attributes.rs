mod class_assignment;
mod names;
pub(super) use class_assignment::replace as replace_class;
pub(super) use names::assign_names;

use crate::{string_scalar, EvalError, EvaluatedArgument, ListElement, Sexp};

pub(super) fn preserve_class(original: &Sexp, elements: Vec<ListElement>) -> Sexp {
    match original.ordinary_list() {
        Some(view) => view.rebuild(elements, view.names_present),
        None => Sexp::List(elements),
    }
}

pub(super) fn attributes(value: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Closure { .. }) {
        let attrs = crate::closure_attributes::attributes(value)?;
        return Ok(if attrs.is_empty() {
            Sexp::Nil
        } else {
            Sexp::List(attrs.to_vec())
        });
    }
    if value.character_values().is_some() {
        let attrs = crate::character_attributes::attributes(value)?;
        return Ok(if attrs.is_empty() {
            Sexp::Nil
        } else {
            Sexp::List(attrs.to_vec())
        });
    }
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    let mut result = Vec::new();
    if let Some(view) = value.ordinary_list() {
        if let Some(class) = view.class.filter(|_| view.class_first) {
            result.push(attribute("class", class.clone()));
        }
        if view.names_present {
            result.push(attribute("names", crate::names_value(value)?));
        }
        if let Some(class) = view.class.filter(|_| !view.class_first) {
            result.push(attribute("class", class.clone()));
        }
        return Ok(if result.is_empty() {
            Sexp::Nil
        } else {
            Sexp::List(result)
        });
    }
    match value {
        Sexp::Expression { names_present, .. } => {
            if *names_present {
                result.push(attribute("names", crate::names_value(value)?));
            }
        }
        Sexp::PairList(_) => {
            let names = crate::names_value(value)?;
            if !matches!(names, Sexp::Nil) {
                result.push(attribute("names", names));
            }
        }
        Sexp::Nil
        | Sexp::Char(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Vector(_) => {}
        _ => return Err(EvalError::NotImplemented("attributes of this object type")),
    }
    Ok(if result.is_empty() {
        Sexp::Nil
    } else {
        Sexp::List(result)
    })
}

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

pub(super) fn replace(value: &Sexp, attrs: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        let projection = crate::ordinary_values::read(value)?;
        let result = replace(projection.as_ref(), attrs)?;
        return crate::ordinary_values::replaced(value, result);
    }
    let attrs_projection = crate::ordinary_values::read(attrs)?;
    let attrs = match attrs_projection.as_ref() {
        Sexp::Nil => &[][..],
        Sexp::List(elements)
        | Sexp::ClassedList { elements, .. }
        | Sexp::NamedList { elements, .. } => elements.as_slice(),
        _ => return Err(EvalError::Type("attributes must be a list or NULL".into())),
    };
    if !attrs.is_empty() && attrs.iter().all(|attr| attr.name.is_none()) {
        return Err(EvalError::Type("attributes must be named".into()));
    }
    for (index, attr) in attrs.iter().enumerate() {
        if index > 0 && attr.name.as_deref().is_none_or(str::is_empty) {
            return Err(EvalError::Type(format!(
                "all attributes must have names [{} does not]",
                index + 1
            )));
        }
    }
    if value.character_values().is_some() {
        return crate::character_attributes::replace(value, attrs);
    }
    if matches!(value, Sexp::Closure { .. }) {
        return crate::closure_attributes::replace(value, attrs);
    }
    if matches!(value, Sexp::Char(_)) {
        return if attrs.is_empty() {
            Ok(value.clone())
        } else {
            Err(EvalError::Domain(
                "cannot set an attribute on a CHARSXP".into(),
            ))
        };
    }
    let mut result = match value {
        Sexp::PairList(values) => {
            crate::pairlist_values::assign_names(values.as_slice(), &Sexp::Nil)?
        }
        Sexp::List(elements)
        | Sexp::ClassedList { elements, .. }
        | Sexp::NamedList { elements, .. } => Sexp::List(
            elements
                .iter()
                .map(|element| ListElement {
                    name: None,
                    value: element.value.clone(),
                })
                .collect(),
        ),
        Sexp::Nil if attrs.is_empty() => return Ok(Sexp::Nil),
        Sexp::Nil => Sexp::List(Vec::new()),
        _ => {
            return Err(EvalError::NotImplemented(
                "attribute replacement on non-list values",
            ))
        }
    };
    for attr in attrs {
        result = set(&result, attr.name.as_deref().unwrap_or(""), &attr.value)?;
    }
    Ok(result)
}

pub(super) fn set(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Closure { .. }) {
        return crate::closure_attributes::set(value, name, replacement);
    }
    if value.character_values().is_some() {
        return crate::character_attributes::set(value, name, replacement);
    }
    if matches!(value, Sexp::Char(_)) {
        return Err(EvalError::Domain(
            "cannot set an attribute on a CHARSXP".into(),
        ));
    }
    match name {
        "" => Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        )),
        "names" => crate::names_assign_value(value, replacement),
        "class" => set_class(value, replacement),
        _ => Err(EvalError::NotImplemented(
            "ordinary list attributes other than class and names",
        )),
    }
}

fn set_class(value: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    crate::character_attributes::validate(replacement)?;
    if let Sexp::ListObject(object) = value {
        let result = set_class(&object.value()?, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    let class = match replacement {
        Sexp::Nil => &[][..],
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values.as_slice(),
        _ => {
            return Err(EvalError::Type(
                "attempt to set invalid 'class' attribute".into(),
            ))
        }
    };
    if class.iter().any(|name| name.as_deref() == Some("factor")) {
        return Err(EvalError::Type(
            "adding class \"factor\" to an invalid object".into(),
        ));
    }
    let elements = match value {
        Sexp::List(elements)
        | Sexp::ClassedList { elements, .. }
        | Sexp::NamedList { elements, .. } => elements.clone(),
        Sexp::Nil if class.is_empty() => return Ok(Sexp::Nil),
        Sexp::Nil => {
            return Err(EvalError::Type(
                "attempt to set an attribute on NULL".into(),
            ))
        }
        _ => {
            return Err(EvalError::NotImplemented(
                "class replacement on non-list values",
            ))
        }
    };
    let view = value.ordinary_list().ok_or(EvalError::NotImplemented(
        "class replacement on non-list values",
    ))?;
    let class_first = if view.class.is_some() {
        view.class_first
    } else {
        !view.names_present
    };
    Ok(Sexp::list_with_metadata(
        elements,
        view.names_present,
        if class.is_empty() {
            None
        } else {
            Some((replacement.clone(), class_first))
        },
    ))
}

pub(super) fn as_vector(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let [value, mode] = args else {
        return Err(EvalError::Arity("as.vector requires two arguments".into()));
    };
    let mode = string_scalar(&mode.value, "mode")?;
    let projection = crate::ordinary_values::read(&value.value)?;
    if let Some(view) = projection.ordinary_list() {
        return match mode.as_str() {
            "any" | "list" => Ok(value.value.clone()),
            "pairlist" => Ok(crate::list_construction::to_pairlist(view)),
            _ => Err(EvalError::NotImplemented(
                "as.vector list coercion to this mode",
            )),
        };
    }
    match (projection.as_ref(), mode.as_str()) {
        (Sexp::Nil | Sexp::PairList(_), "pairlist") => Ok(value.value.clone()),
        (Sexp::Nil, "list") => Ok(Sexp::List(Vec::new())),
        (Sexp::PairList(values), "list") => {
            Ok(crate::list_construction::from_tags(values.as_slice()))
        }
        _ => Err(EvalError::NotImplemented("as.vector of this type and mode")),
    }
}
