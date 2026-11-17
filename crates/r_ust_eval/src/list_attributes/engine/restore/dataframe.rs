use super::super::{attributes, find};
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::AggregateVector;

pub(super) fn restore(
    original: &Sexp,
    values: &AggregateVector,
    metadata: &[ListElement],
) -> Result<Option<Sexp>, EvalError> {
    let Sexp::DataFrame { nrow, .. } = original else {
        return Ok(None);
    };
    let AggregateVector::Vector(values) = values else {
        return Ok(None);
    };
    let Some(Sexp::String(names)) = find(metadata, "names") else {
        return Ok(None);
    };
    if names.len() != values.len() {
        return Ok(None);
    }
    let candidate = Sexp::DataFrame {
        columns: values
            .iter()
            .zip(names)
            .map(|(value, name)| ListElement {
                name: name.clone(),
                value: value.clone(),
            })
            .collect(),
        nrow: *nrow,
    };
    let Some(description) = candidate.attribute_description()? else {
        return Ok(None);
    };
    Ok((attributes(&description) == metadata).then_some(candidate))
}
