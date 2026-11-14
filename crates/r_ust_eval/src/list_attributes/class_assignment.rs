use crate::{EvalError, Sexp};

// R 4.6.0 src/main/coerce.c R_set_class differs from attribute-list classgets.
pub(crate) fn replace(value: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    crate::character_attributes::validate(replacement)?;
    if matches!(value, Sexp::Char(_)) {
        return Err(EvalError::Domain(
            "cannot set an attribute on a CHARSXP".into(),
        ));
    }
    if let Sexp::ListObject(object) = value {
        let result = replace(&object.value()?, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    let classes = match replacement {
        Sexp::Nil => return super::set(value, "class", replacement),
        Sexp::String(classes)
        | Sexp::AttributedCharacter {
            values: classes, ..
        } => classes,
        _ => return Err(EvalError::NotImplemented("class replacement coercion")),
    };
    let first = classes.first().and_then(Option::as_deref);
    if first == Some("matrix") {
        if !matches!(value, Sexp::Matrix { .. }) {
            return Err(EvalError::Domain("invalid to set the class to matrix unless the dimension attribute is of length 2 (was 0)".into()));
        }
        if classes.len() == 1 || (classes.len() == 2 && classes[1].as_deref() == Some("array")) {
            return Ok(value.clone());
        }
    }
    if classes.len() == 1 {
        match first {
            Some("list") => match value {
                Sexp::List(_) | Sexp::ClassedList { .. } => {
                    return super::set(value, "class", &Sexp::Nil)
                }
                Sexp::Nil => return Ok(Sexp::List(Vec::new())),
                _ => {
                    return Err(EvalError::NotImplemented(
                        "class replacement list conversion",
                    ))
                }
            },
            Some("array") => {
                if matches!(value, Sexp::Matrix { .. }) {
                    return Ok(value.clone());
                }
                return Err(EvalError::Domain(
                    "cannot set class to \"array\" unless the dimension attribute has length > 0"
                        .into(),
                ));
            }
            Some(
                "logical" | "integer" | "double" | "raw" | "complex" | "character" | "expression"
                | "char" | "numeric",
            ) => {
                return Err(EvalError::NotImplemented(
                    "class replacement type conversion",
                ))
            }
            Some(name @ ("environment" | "externalptr" | "weakref" | "name")) => {
                if matches!(value, Sexp::List(_) | Sexp::ClassedList { .. }) {
                    return Err(EvalError::Domain(format!("\"{name}\" can only be set as the class if the object has this type; found \"list\"")));
                }
                return Err(EvalError::NotImplemented(
                    "class replacement on this object type",
                ));
            }
            _ => {}
        }
    }
    super::set(value, "class", replacement)
}
