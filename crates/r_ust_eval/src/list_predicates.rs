use super::*;

pub(super) fn is_na_values(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Sexp::PairList(values) = value {
        return is_na_values(&Sexp::List(values.as_slice().to_vec()));
    }
    if matches!(value, Sexp::Char(_)) {
        return Ok(vec![Some(false); value_len(value)?]);
    }
    Ok(match value {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::Integer(values) => values.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::Real(values) => values.iter().map(|value| Some(value.is_nan())).collect(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.iter().map(|value| Some(value.is_none())).collect()
        }
        Sexp::Factor { codes, .. } => codes.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::DataFrame { columns, .. } => {
            let mut out = Vec::new();
            for column in columns {
                out.extend(is_na_values(&column.value)?);
            }
            out
        }
        Sexp::Matrix { data, .. } => is_na_values(data)?,
        Sexp::Vector(values) => values
            .iter()
            .map(|value| Some(any_na::scalar_missing(value)))
            .collect(),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => values
            .iter()
            .map(|element| Some(any_na::scalar_missing(&element.value)))
            .collect(),
        _ => vec![Some(false)],
    })
}

pub(super) fn numeric_predicate_values(
    value: &Sexp,
    predicate: NumericPredicate,
) -> Result<Vec<Option<bool>>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    if matches!(value, Sexp::Char(_)) {
        return Err(EvalError::NotImplemented("CHARSXP numeric predicates"));
    }
    Ok(match value {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan | NumericPredicate::Infinite => false,
                    NumericPredicate::Finite => value.is_some(),
                })
            })
            .collect(),
        Sexp::Integer(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan | NumericPredicate::Infinite => false,
                    NumericPredicate::Finite => value.is_some(),
                })
            })
            .collect(),
        Sexp::Real(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan => value.is_nan() && !is_na_real(*value),
                    NumericPredicate::Finite => value.is_finite(),
                    NumericPredicate::Infinite => value.is_infinite(),
                })
            })
            .collect(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.iter().map(|_| Some(false)).collect()
        }
        Sexp::Factor { codes, .. } => codes.iter().map(|_| Some(false)).collect(),
        Sexp::Matrix { data, .. } => numeric_predicate_values(data, predicate)?,
        Sexp::Vector(values) => values.iter().map(|_| Some(false)).collect(),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => values.iter().map(|_| Some(false)).collect(),
        Sexp::DataFrame { columns, .. } => columns.iter().map(|_| Some(false)).collect(),
        _ => vec![Some(false)],
    })
}
