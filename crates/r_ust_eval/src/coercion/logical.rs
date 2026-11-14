use crate::{EvalError, Sexp};

pub(crate) fn convert(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Char(_) => Err(EvalError::Type(
            "cannot coerce type 'char' to vector of type 'logical'".into(),
        )),
        Sexp::Vector(values) => list_values(values.iter()),
        Sexp::ClassedList { elements, .. }
        | Sexp::List(elements)
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
    for value in values.clone() {
        if super::lists::vectorizable(value)? {
            continue;
        }
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
