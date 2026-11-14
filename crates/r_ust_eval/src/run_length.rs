use super::{
    dollar_value, positional, real_to_integer, subset_linear_positions, value_len, EvalError,
    EvaluatedArgument, ListElement, RuntimeState, Sexp,
};

pub(super) fn encode(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    if args.len() > 1
        || args
            .iter()
            .any(|arg| arg.name.as_deref().is_some_and(|n| n != "x"))
    {
        return Err(EvalError::Arity("unused argument in rle()".into()));
    }
    let value = positional(args, 0, "x")?;
    let (lengths, positions) = match value {
        Sexp::Logical(values) => runs(values, |a, b| a.is_some() && a == b)?,
        Sexp::Integer(values) => runs(values, |a, b| a.is_some() && a == b)?,
        // R compares adjacent doubles exactly; NaN and NA must each start a run.
        Sexp::Real(values) => runs(values, |a, b| a == b)?,
        Sexp::String(values) => runs(values, |a, b| a.is_some() && a == b)?,
        Sexp::List(values) => {
            if values.len() > 1 {
                return Err(EvalError::Type(
                    "comparison of these types is not implemented".into(),
                ));
            }
            runs(values, |_, _| false)?
        }
        Sexp::Vector(values) => {
            if values.len() > 1 {
                return Err(EvalError::Type(
                    "comparison of these types is not implemented".into(),
                ));
            }
            runs(values, |_, _| false)?
        }
        Sexp::DataFrame { .. } => return Err(EvalError::NotImplemented("rle data frames")),
        Sexp::Nil
        | Sexp::Factor { .. }
        | Sexp::Matrix { .. }
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Block(_)
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::InternalCall { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => {
            return Err(EvalError::Type(
                "'x' must be a vector of an atomic type".into(),
            ));
        }
    };
    // shortcut: Sexp has no general class attribute; add class "rle" when attributes are represented.
    Ok(Sexp::List(vec![
        ListElement {
            name: Some("lengths".into()),
            value: lengths,
        },
        ListElement {
            name: Some("values".into()),
            value: subset_linear_positions(value, &positions)?,
        },
    ]))
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

impl RuntimeState {
    pub(super) fn inverse_rle(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
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
        if matches!(x.value, Sexp::Nil) {
            return Err(invalid());
        }
        let lengths = dollar_value(&x.value, "lengths")
            .map_err(|_| EvalError::Type("$ operator is invalid for atomic vectors".into()))?;
        let values = dollar_value(&x.value, "values")?;
        if matches!(lengths, Sexp::Nil)
            || matches!(values, Sexp::Nil)
            || value_len(&lengths) != value_len(&values)
        {
            return Err(invalid());
        }
        let counts = self.call_numeric_coercion(
            "as.double",
            &[EvaluatedArgument {
                name: None,
                value: lengths,
            }],
        )?;
        let Sexp::Real(counts) = counts else {
            return Err(EvalError::Type("invalid 'times' value".into()));
        };
        if counts
            .iter()
            .any(|count| !count.is_finite() || *count <= -1.0 || *count >= 4503599627370497.0)
        {
            return Err(EvalError::Domain("invalid 'times' value".into()));
        }
        if counts
            .iter()
            .any(|count| count.is_finite() && *count >= 2147483648.0)
        {
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
        let mut result = subset_linear_positions(&values, &positions)?;
        if let Sexp::List(elements) = &mut result {
            for element in elements {
                element.name = None;
            }
        }
        Ok(result)
    }
}
