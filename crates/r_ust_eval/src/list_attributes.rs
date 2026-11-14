mod class_assignment;
mod names;
pub(super) use class_assignment::replace as replace_class;
pub(super) use names::assign_names;

use crate::{string_scalar, EvalError, EvaluatedArgument, ListElement, Sexp};

pub(super) fn preserve_class(original: &Sexp, elements: Vec<ListElement>) -> Sexp {
    match original {
        Sexp::ClassedList {
            class, class_first, ..
        } => Sexp::ClassedList {
            elements,
            class: class.clone(),
            class_first: *class_first,
        },
        _ => Sexp::List(elements),
    }
}

pub(super) fn attributes(value: &Sexp) -> Result<Sexp, EvalError> {
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
    match value {
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => {
            if let Sexp::ClassedList {
                class,
                class_first: true,
                ..
            } = value
            {
                result.push(attribute("class", class.as_ref().clone()));
            }
            if elements.iter().any(|element| element.name.is_some()) {
                result.push(attribute("names", crate::names_value(value)?));
            }
            if let Sexp::ClassedList {
                class,
                class_first: false,
                ..
            } = value
            {
                result.push(attribute("class", class.as_ref().clone()));
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
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => elements.as_slice(),
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
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => Sexp::List(
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
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => elements.clone(),
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
    let class_first = match value {
        Sexp::ClassedList { class_first, .. } => *class_first,
        _ => elements.iter().all(|element| element.name.is_none()),
    };
    Ok(if class.is_empty() {
        Sexp::List(elements)
    } else {
        Sexp::ClassedList {
            elements,
            class: Box::new(replacement.clone()),
            class_first,
        }
    })
}

pub(super) fn as_vector(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let [value, mode] = args else {
        return Err(EvalError::Arity("as.vector requires two arguments".into()));
    };
    if string_scalar(&mode.value, "mode")? != "pairlist" {
        return Err(EvalError::NotImplemented(
            "as.vector modes other than pairlist",
        ));
    }
    match &value.value {
        Sexp::Nil | Sexp::PairList(_) | Sexp::PairListObject(_) => Ok(value.value.clone()),
        Sexp::ListObject(value) => match value.value()? {
            Sexp::List(elements) | Sexp::ClassedList { elements, .. } => {
                Ok(Sexp::pairlist(elements))
            }
            _ => Err(EvalError::Graph(r_ust_sexp::GraphError::WrongKind)),
        },
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => {
            Ok(Sexp::pairlist(elements.clone()))
        }
        _ => Err(EvalError::NotImplemented("as.pairlist of non-list values")),
    }
}
