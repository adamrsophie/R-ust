use crate::{ordinary_values, EvalError, Sexp};

pub(crate) fn validate(value: &Sexp) -> Result<(), EvalError> {
    Validator {
        visited: Vec::new(),
    }
    .visit(value)
}

pub(super) fn check_class(class: &Sexp) -> Result<(), EvalError> {
    let Some(values) = class.character_values() else {
        return Err(EvalError::Type("class storage must be character".into()));
    };
    if values.is_empty() {
        return Err(EvalError::Type(
            "empty class storage must be normalized to a list".into(),
        ));
    }
    Ok(())
}

struct Validator {
    visited: Vec<Sexp>,
}

impl Validator {
    fn visit(&mut self, value: &Sexp) -> Result<(), EvalError> {
        if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
            if self.visited.contains(value) {
                return Ok(());
            }
            self.visited.push(value.clone());
        }
        let projection = ordinary_values::read(value)?;
        let value = projection.as_ref();
        if let Sexp::ClassedList { class, .. } = value {
            check_class(class)?;
            self.visit(class)?;
        }
        if let Sexp::AttributedCharacter { attributes, .. } = value {
            for (index, attribute) in attributes.iter().enumerate() {
                let name = attribute
                    .name
                    .as_deref()
                    .ok_or_else(|| EvalError::Type("character attributes must be named".into()))?;
                super::validate_name(name)?;
                if attributes[..index]
                    .iter()
                    .any(|other| other.name == attribute.name)
                {
                    return Err(EvalError::Type("duplicate character attribute".into()));
                }
                if matches!(attribute.value, Sexp::Nil) {
                    return Err(EvalError::Type(
                        "NULL character attribute must be removed".into(),
                    ));
                }
                self.visit(&attribute.value)?;
            }
        }
        match value {
            Sexp::Char(value) if value.as_ref().is_some_and(|value| value.contains('\0')) => {
                return Err(EvalError::NotImplemented("CHARSXP with embedded NUL"));
            }
            Sexp::List(elements)
            | Sexp::Dots(elements)
            | Sexp::ClassedList { elements, .. }
            | Sexp::DataFrame {
                columns: elements, ..
            } => {
                for element in elements {
                    self.visit(&element.value)?;
                }
            }
            Sexp::PairList(elements) => {
                for element in elements.as_slice() {
                    self.visit(&element.value)?;
                }
            }
            Sexp::Vector(values) => {
                for value in values {
                    self.visit(value)?;
                }
            }
            Sexp::Matrix { data, .. } => self.visit(data)?,
            _ => {}
        }
        Ok(())
    }
}
