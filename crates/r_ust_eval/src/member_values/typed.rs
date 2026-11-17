use crate::{list_attributes::engine, EvalError, ListElement, Sexp};

pub(super) fn extract(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    let Sexp::AttributedAggregate { values, .. } = value else {
        unreachable!("typed aggregate extraction receives a descriptor")
    };
    let names = engine::payload(&engine::get(value, "names")?)?;
    let names = match &names {
        Sexp::Nil => None,
        names => Some(
            names
                .character_values()
                .ok_or_else(|| EvalError::Type("invalid 'names' attribute".into()))?,
        ),
    };
    let elements = values
        .values()
        .iter()
        .enumerate()
        .map(|(i, value)| ListElement {
            name: names.and_then(|names| names.get(i)).cloned().flatten(),
            value: value.clone(),
        })
        .collect::<Vec<_>>();
    let index = engine::payload(index)?;
    crate::expression_values::extract((&elements, names.is_some()), &index)
}
