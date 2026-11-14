use r_ust_sexp::Sexp;

use super::atomic;
use super::strings::BlankPolicy;
use super::warnings::Warnings;
use crate::{value_len, EvalError};

pub(super) fn convert(
    value: &Sexp,
    mode: (bool, BlankPolicy),
    mut warn: impl FnMut(Warnings) -> Result<(), EvalError>,
) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    let (integer, policy) = mode;
    let elements: Vec<&Sexp> = match value {
        Sexp::Vector(values) => values.iter().collect(),
        Sexp::ClassedList { elements, .. }
        | Sexp::List(elements)
        | Sexp::DataFrame {
            columns: elements, ..
        } => elements.iter().map(|element| &element.value).collect(),
        Sexp::Matrix { data, .. } => return convert(data, mode, warn),
        other => {
            return if integer {
                let (values, warnings) = atomic::integers(other, policy)?;
                warn(warnings)?;
                Ok(Sexp::Integer(values))
            } else {
                let (values, warnings) = atomic::reals(other, policy)?;
                warn(warnings)?;
                Ok(Sexp::Real(values))
            };
        }
    };
    for element in &elements {
        if vectorizable(element)? {
            continue;
        }
        return Err(EvalError::Type(format!(
            "'list' object cannot be coerced to type '{}'",
            if integer { "integer" } else { "double" }
        )));
    }
    if integer {
        let mut values = Vec::with_capacity(elements.len());
        for element in elements {
            let (value, warnings) = atomic::scalar_integer(element, policy);
            warn(warnings)?;
            values.push(value);
        }
        Ok(Sexp::Integer(values))
    } else {
        let mut values = Vec::with_capacity(elements.len());
        for element in elements {
            let (value, warnings) = atomic::scalar_real(element, policy);
            warn(warnings)?;
            values.push(value);
        }
        Ok(Sexp::Real(values))
    }
}

pub(super) fn vectorizable(value: &Sexp) -> Result<bool, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    Ok(matches!(
        value,
        Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::AttributedCharacter { .. }
            | Sexp::Vector(_)
            | Sexp::ClassedList { .. }
            | Sexp::List(_)
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
    ) && value_len(value)? <= 1)
}
