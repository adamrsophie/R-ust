use crate::{EvalError, Sexp};

pub(crate) fn convert(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    match value {
        Sexp::Vector(values) => list_values(values.iter()),
        Sexp::List(elements)
        | Sexp::DataFrame {
            columns: elements, ..
        } => list_values(elements.iter().map(|element| &element.value)),
        Sexp::Matrix { data, .. } => convert(data),
        other => crate::coerce_logical_values(other),
    }
}

fn list_values<'a>(
    values: impl ExactSizeIterator<Item = &'a Sexp> + Clone,
) -> Result<Vec<Option<bool>>, EvalError> {
    if !values.clone().all(super::lists::vectorizable) {
        return Err(EvalError::Type(
            "'list' object cannot be coerced to type 'logical'".into(),
        ));
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|error| EvalError::Domain(format!("cannot allocate vector: {error}")))?;
    result.extend(values.map(crate::internal::as_logical));
    Ok(result)
}
