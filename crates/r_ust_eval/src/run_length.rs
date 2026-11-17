use super::{positional, EvalError, EvaluatedArgument, ListElement, Sexp};

mod fields;
mod inverse;
mod select;
mod storage;
#[cfg(test)]
mod tests;

pub(super) fn encode(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    if args.len() > 1
        || args
            .iter()
            .any(|arg| arg.name.as_deref().is_some_and(|n| n != "x"))
    {
        return Err(EvalError::Arity("unused argument in rle()".into()));
    }
    let value = positional(args, 0, "x")?;
    if matches!(storage::read(value)?, Sexp::Factor { .. }) {
        return Err(EvalError::Type(
            "'x' must be a vector of an atomic type".into(),
        ));
    }
    let plain = storage::payload(value)?;
    let list = matches!(plain, Sexp::Vector(_) | Sexp::PairList(_));
    let attributes = storage::attributes(value)?;
    let class = attributes
        .iter()
        .find(|a| a.name.as_deref() == Some("class"))
        .map(|a| storage::payload(&a.value))
        .transpose()?;
    if list
        && class
            .as_ref()
            .and_then(Sexp::character_values)
            .is_some_and(|names| {
                names
                    .iter()
                    .any(|name| name.as_deref() == Some("data.frame"))
            })
    {
        return Err(EvalError::NotImplemented("rle data frames"));
    }
    if !list
        && attributes
            .iter()
            .any(|a| a.name.as_deref() != Some("names"))
    {
        return Err(EvalError::Type(
            "'x' must be a vector of an atomic type".into(),
        ));
    }
    let (mut lengths, positions) = match &plain {
        Sexp::Logical(values) => runs(values, |a, b| a.is_some() && a == b)?,
        Sexp::Integer(values) => runs(values, |a, b| a.is_some() && a == b)?,
        Sexp::Real(values) => runs(values, |a, b| a == b)?,
        Sexp::String(values) => runs(values, |a, b| a.is_some() && a == b)?,
        Sexp::Vector(values) => aggregate_runs(values)?,
        Sexp::Expression { elements, .. } => aggregate_runs(elements)?,
        Sexp::PairList(values) => aggregate_runs(values.as_slice())?,
        _ => {
            return Err(EvalError::Type(
                "'x' must be a vector of an atomic type".into(),
            ))
        }
    };
    let empty = positions.is_empty();
    if !empty {
        if let Some(names) = select::names(value)? {
            let mut boundaries = positions
                .iter()
                .take(positions.len() - 1)
                .map(|position| position.and_then(|i| names.get(i + 1)).cloned().flatten())
                .collect::<Vec<_>>();
            boundaries.push(Some(String::new()));
            lengths = select::decorate(
                lengths,
                vec![ListElement {
                    name: Some("names".into()),
                    value: Sexp::String(boundaries),
                }],
            )?;
        }
    }
    let values = if empty {
        value.clone()
    } else {
        select::positions(value, &positions, true)?
    };
    Ok(Sexp::list_with_metadata(
        vec![
            ListElement {
                name: Some("lengths".into()),
                value: lengths,
            },
            ListElement {
                name: Some("values".into()),
                value: values,
            },
        ],
        true,
        Some((Sexp::String(vec![Some("rle".into())]), false)),
    ))
}

fn runs<T>(
    values: &[T],
    equal: impl Fn(&T, &T) -> bool,
) -> Result<(Sexp, Vec<Option<usize>>), EvalError> {
    let mut lengths = Vec::new();
    let mut positions = Vec::new();
    let mut length: i32 = 0;
    for (index, value) in values.iter().enumerate() {
        length = length
            .checked_add(1)
            .ok_or(EvalError::NotImplemented("long-vector run lengths"))?;
        if values.get(index + 1).is_none_or(|next| !equal(value, next)) {
            lengths.push(Some(length));
            positions.push(Some(index));
            length = 0;
        }
    }
    Ok((Sexp::Integer(lengths), positions))
}

fn aggregate_runs<T>(values: &[T]) -> Result<(Sexp, Vec<Option<usize>>), EvalError> {
    if values.len() > 1 {
        return Err(EvalError::Type(
            "comparison of these types is not implemented".into(),
        ));
    }
    runs(values, |_, _| false)
}
