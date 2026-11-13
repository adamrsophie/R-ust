use super::{empty_like_atomic, scalar_elements_to_vector, sequence_elements, EvalError, Sexp};

pub(super) fn matrix_data(
    data: &Sexp,
    nrow: usize,
    ncol: usize,
    byrow: bool,
) -> Result<Sexp, EvalError> {
    let total = nrow.saturating_mul(ncol);
    if total == 0 {
        return match data {
            Sexp::Nil => Err(EvalError::Type(
                "'data' must be of a vector type, was 'NULL'".into(),
            )),
            Sexp::Factor { .. } => Ok(Sexp::String(Vec::new())),
            Sexp::List(_) | Sexp::Vector(_) | Sexp::Dots(_) => Ok(Sexp::List(Vec::new())),
            _ => Ok(empty_like_atomic(data)),
        };
    }
    let mut elements = sequence_elements(data)?;
    if elements.is_empty() {
        elements.push(Sexp::Logical(vec![None]));
    }
    let mut out = vec![Sexp::Nil; total];
    for row in 0..nrow {
        for col in 0..ncol {
            let storage_index = row + col * nrow;
            let source_index = if byrow {
                row * ncol + col
            } else {
                storage_index
            };
            out[storage_index] = elements[source_index % elements.len()].clone();
        }
    }
    scalar_elements_to_vector(&out)
}

pub(super) fn matrix_column_values(
    data: &Sexp,
    nrow: usize,
    col: usize,
) -> Result<Vec<Sexp>, EvalError> {
    let values = sequence_elements(data)?;
    Ok((0..nrow)
        .map(|row| values.get(row + col * nrow).cloned().unwrap_or(Sexp::Nil))
        .collect())
}

pub(super) fn matrix_row_values(
    data: &Sexp,
    nrow: usize,
    ncol: usize,
    row: usize,
) -> Result<Vec<Sexp>, EvalError> {
    let values = sequence_elements(data)?;
    Ok((0..ncol)
        .map(|col| values.get(row + col * nrow).cloned().unwrap_or(Sexp::Nil))
        .collect())
}
