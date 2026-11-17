use crate::{AtomicVector, GraphError, Sexp};

/// Public getAttrib(row.names) projection; the stored compact description is unchanged.
pub fn expanded_row_names(value: &Sexp) -> Result<Sexp, GraphError> {
    let value = value.snapshot()?;
    let values = match &value {
        Sexp::Integer(values)
        | Sexp::AttributedAtomic {
            values: AtomicVector::Integer(values),
            ..
        } => values,
        _ => return Ok(value),
    };
    let [None, Some(length)] = values.as_slice() else {
        return Ok(value);
    };
    let count = length.checked_abs().ok_or(GraphError::WrongKind)?;
    Ok(Sexp::Integer((1..=count).map(Some).collect()))
}
