use crate::{
    compare_sort_keys, named_bool_arg, positional, sort_keys, EvalError, EvaluatedArgument, Sexp,
    SortKey,
};

pub(crate) fn call_is_unsorted(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let strictly = named_bool_arg(args, &["strictly"], false)?;
    let mut keys = sort_keys(value)?;
    if keys.len() < 2 {
        return Ok(Sexp::Logical(vec![Some(false)]));
    }
    if na_rm {
        keys.retain(|key| !matches!(key, SortKey::Missing));
    } else if keys.iter().any(|key| matches!(key, SortKey::Missing)) {
        return Ok(Sexp::Logical(vec![None]));
    }

    for pair in keys.windows(2) {
        let ordering = compare_sort_keys(&pair[0], &pair[1]);
        if ordering == std::cmp::Ordering::Greater
            || (strictly && ordering == std::cmp::Ordering::Equal)
        {
            return Ok(Sexp::Logical(vec![Some(true)]));
        }
    }
    Ok(Sexp::Logical(vec![Some(false)]))
}
