use super::{fields, select, storage};
use crate::{real_to_integer, EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn inverse_rle(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        if args
            .iter()
            .filter(|arg| arg.name.as_deref() == Some("x"))
            .count()
            > 1
        {
            return Err(EvalError::Arity(
                "formal argument \"x\" matched by multiple actual arguments".into(),
            ));
        }
        let x = args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("x"))
            .or_else(|| args.iter().find(|arg| arg.name.is_none()))
            .ok_or_else(|| EvalError::MissingArgument("x".into()))?;
        let invalid = || EvalError::Domain("invalid 'rle' structure".into());
        let lengths = fields::member(&x.value, "lengths")?;
        if matches!(storage::payload(&lengths)?, Sexp::Nil) {
            return Err(invalid());
        }
        let values = fields::member(&x.value, "values")?;
        if matches!(storage::payload(&values)?, Sexp::Nil)
            || fields::length(&lengths)? != fields::length(&values)?
        {
            return Err(invalid());
        }
        let raw_counts = storage::payload(&lengths)?;
        if !matches!(
            raw_counts,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::Vector(_)
                | Sexp::Expression { .. }
        ) {
            return Err(EvalError::Type(format!(
                "invalid type ({}) for 'times' (must be a vector)",
                crate::metadata::typeof_value(&raw_counts)
            )));
        }
        let raw_values = storage::payload(&values)?;
        if !matches!(
            raw_values,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::Vector(_)
                | Sexp::Expression { .. }
        ) {
            return Err(EvalError::Type(format!(
                "attempt to replicate an object of type '{}'",
                crate::metadata::typeof_value(&raw_values)
            )));
        }
        let counts = self.call_numeric_coercion(
            "as.double",
            &[EvaluatedArgument {
                name: None,
                value: raw_counts,
            }],
        )?;
        let Sexp::Real(counts) = counts else {
            return Err(EvalError::Type("invalid 'times' value".into()));
        };
        let positions = positions(counts)?;
        select::positions(&values, &positions, false)
    }
}

fn positions(counts: Vec<f64>) -> Result<Vec<Option<usize>>, EvalError> {
    if counts
        .iter()
        .any(|count| !count.is_finite() || *count <= -1.0 || *count >= 4503599627370497.0)
    {
        return Err(EvalError::Domain("invalid 'times' value".into()));
    }
    if counts.iter().any(|count| *count >= 2147483648.0) {
        return Err(EvalError::NotImplemented("long-vector run lengths"));
    }
    let counts = counts
        .into_iter()
        .map(|count| {
            real_to_integer(count)
                .and_then(|count| usize::try_from(count).ok())
                .ok_or_else(|| EvalError::Domain("invalid 'times' value".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let total = counts
        .iter()
        .try_fold(0usize, |total, count| total.checked_add(*count))
        .ok_or(EvalError::NotImplemented("long-vector run lengths"))?;
    i32::try_from(total).map_err(|_| EvalError::NotImplemented("long-vector run lengths"))?;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(total)
        .map_err(|_| EvalError::Domain("cannot allocate run-length result".into()))?;
    for (index, count) in counts.into_iter().enumerate() {
        positions.extend(std::iter::repeat_n(Some(index), count));
    }
    Ok(positions)
}
