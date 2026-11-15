use crate::{EvalError, ListElement, Sexp};

pub(super) fn read(elements: &[ListElement], index: &Sexp) -> Result<Option<usize>, EvalError> {
    if let Sexp::Matrix { data, .. } = index {
        return read(elements, data);
    }
    let length = crate::value_len(index)?;
    if length == 0 {
        return Err(EvalError::Domain(
            "attempt to select less than one element in get1index".into(),
        ));
    }
    if length > 1 {
        return Err(EvalError::NotImplemented("recursive language extraction"));
    }
    if let Some(names) = index.character_values() {
        return Ok(names[0]
            .as_deref()
            .filter(|name| !name.is_empty())
            .and_then(|name| named(elements, name)));
    }
    match index {
        Sexp::Symbol(name) => Ok(named(elements, name)),
        Sexp::MissingValue | Sexp::MissingArg(_) => {
            Err(EvalError::Domain("subscript is missing".into()))
        }
        Sexp::Logical(values) => integer(values[0].map(i32::from), elements.len()),
        Sexp::Integer(values) => integer(values[0], elements.len()),
        Sexp::Factor { codes, .. } => integer(
            codes[0]
                .map(i32::try_from)
                .transpose()
                .map_err(|_| EvalError::Domain("subscript out of bounds".into()))?,
            elements.len(),
        ),
        Sexp::Real(values) => real(values[0], elements.len()),
        _ => Err(invalid_type(index)),
    }
}

fn named(elements: &[ListElement], name: &str) -> Option<usize> {
    elements
        .iter()
        .position(|slot| slot.name.as_deref() == Some(name))
}

fn integer(value: Option<i32>, length: usize) -> Result<Option<usize>, EvalError> {
    let Some(value) = value else { return Ok(None) };
    if value > 0 {
        return Ok(usize::try_from(value - 1).ok());
    }
    if value == 0 || length < 2 {
        return Err(EvalError::Domain(
            "attempt to select less than one element in integerOneIndex".into(),
        ));
    }
    if length == 2 && value > -3 {
        return Ok(usize::try_from(2 + value).ok());
    }
    Err(EvalError::Domain(
        "attempt to select more than one element in integerOneIndex".into(),
    ))
}

fn real(value: f64, length: usize) -> Result<Option<usize>, EvalError> {
    if value.is_nan() {
        return Ok(None);
    }
    if value >= 1.0 {
        if !value.is_finite() {
            return Ok(None);
        }
        return crate::extraction_position(&Sexp::Real(vec![value])).map(Some);
    }
    if value > -1.0 || length < 2 {
        return Err(real_error(if value <= -1.0 {
            "invalid negative subscript"
        } else {
            "attempt to select less than one element"
        }));
    }
    if length == 2 && value > -3.0 {
        let values = crate::numeric_index_values(&Sexp::Real(vec![value]))?;
        return Ok(values[0].and_then(|value| usize::try_from(2 + value).ok()));
    }
    Err(real_error(if value <= -1.0 {
        "invalid negative subscript"
    } else {
        "attempt to select more than one element"
    }))
}

fn real_error(message: &str) -> EvalError {
    EvalError::Domain(format!("{message} in get1index <real>"))
}

pub(super) fn invalid_type(index: &Sexp) -> EvalError {
    EvalError::Type(format!(
        "invalid subscript type '{}'",
        crate::metadata::typeof_value(index)
    ))
}
