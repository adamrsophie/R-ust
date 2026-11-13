use super::{
    coerce_logical_values, data_args, is_na_real, numeric_result, numeric_values, preserve_na,
    real_to_integer, typeof_value, value_len, EvalError, EvaluatedArgument, Sexp,
};

fn arguments(args: &[EvaluatedArgument], operation: &str) -> Result<bool, EvalError> {
    let mut flags = args
        .iter()
        .filter(|arg| arg.name.as_deref() == Some("na.rm"));
    let flag = flags.next();
    if flags.next().is_some() {
        return Err(EvalError::Arity(
            "formal argument \"na.rm\" matched by multiple actual arguments".into(),
        ));
    }
    let first = args
        .iter()
        .find(|arg| arg.name.as_deref() != Some("na.rm"))
        .or(flag);
    if let Some(EvaluatedArgument {
        value: Sexp::Factor { ordered, .. },
        ..
    }) = first
    {
        let message = if *ordered {
            format!("'{operation}' not defined for ordered factors")
        } else {
            // shortcut: use straight quotes, locale-aware condition rendering remains separate.
            format!("'{operation}' not meaningful for factors")
        };
        return Err(EvalError::Type(message));
    }
    let Some(flag) = flag else {
        return Ok(false);
    };
    let value = storage(&flag.value);
    if matches!(
        typeof_value(value),
        "logical" | "integer" | "double" | "character"
    ) && value_len(value) > 1
    {
        return Err(EvalError::Domain(format!(
            "'length = {}' in coercion to 'logical(1)'",
            value_len(value)
        )));
    }
    let logical = match value {
        Sexp::Factor { codes, .. } => codes.first().copied().flatten().map(|code| code != 0),
        _ => coerce_logical_values(value)
            .map_err(|_| EvalError::Domain("NA in coercion to boolean".into()))?
            .first()
            .copied()
            .flatten(),
    };
    logical.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))
}

fn storage(value: &Sexp) -> &Sexp {
    match value {
        Sexp::Matrix { data, .. } => storage(data),
        other => other,
    }
}

fn values(args: &[EvaluatedArgument], remove_missing: bool) -> Result<Vec<f64>, EvalError> {
    let mut values = Vec::new();
    for arg in data_args(args, &["na.rm"]) {
        let value = storage(&arg.value);
        if !matches!(
            typeof_value(value),
            "NULL" | "logical" | "integer" | "double"
        ) {
            return Err(EvalError::Type(format!(
                "invalid 'type' ({}) of argument",
                typeof_value(value)
            )));
        }
        values.extend(
            numeric_values(value)?
                .into_iter()
                .filter(|value| !(remove_missing && value.is_nan())),
        );
    }
    Ok(values)
}

pub(super) fn sum(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let remove_missing = arguments(args, "sum")?;
    let values = values(args, remove_missing)?;
    let value = preserve_na(values.iter().fold(0.0, |sum, value| sum + value), &values);
    let integer = data_args(args, &["na.rm"])
        .iter()
        .all(|arg| matches!(typeof_value(&arg.value), "NULL" | "logical" | "integer"))
        && (is_na_real(value) || real_to_integer(value).is_some());
    Ok(numeric_result(vec![value], integer))
}

pub(super) fn product(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let remove_missing = arguments(args, "prod")?;
    let values = values(args, remove_missing)?;
    Ok(Sexp::Real(vec![preserve_na(
        values.iter().product(),
        &values,
    )]))
}
