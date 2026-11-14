use super::logical::as_logical;
use crate::{value_len, EvalError, EvaluatedArgument, Sexp};

pub(super) fn find_interval(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let (Sexp::Real(breaks), Sexp::Real(values)) = (&args[0].value, &args[1].value) else {
        return Err(EvalError::Domain("invalid input".into()));
    };
    let n = i32::try_from(breaks.len())
        .map_err(|_| EvalError::Domain("long vector 'vec' is not supported".into()))?;
    let mut flags = [false; 3];
    for (flag, arg) in flags.iter_mut().zip(&args[2..5]) {
        let len = value_len(&arg.value)?;
        if len > 1 {
            return Err(EvalError::Domain(format!(
                "'length = {len}' in coercion to 'logical(1)'"
            )));
        }
        *flag = as_logical(&arg.value)
            .ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
    }
    let [closed, inside, left_open] = flags;
    // shortcut: reject invalid unchecked breaks; support only sorted, nonmissing inputs.
    if breaks.len() > 1
        && (breaks.iter().any(|x| x.is_nan()) || breaks.windows(2).any(|pair| pair[0] > pair[1]))
    {
        return Err(EvalError::Domain(
            "'vec' must be sorted non-decreasingly and not contain NAs".into(),
        ));
    }
    // R 4.6.0 tests the checkNA SEXP pointer, so even FALSE keeps the NaN check.
    let result = values
        .iter()
        .map(|x| {
            if x.is_nan() {
                return Ok(None);
            }
            let (Some(first), Some(last)) = (breaks.first(), breaks.last()) else {
                return Ok(Some(0));
            };
            let index = breaks.partition_point(|b| if left_open { b < x } else { b <= x });
            let index = if index == 0 {
                i32::from(inside || (closed && x == first))
            } else if index == breaks.len() {
                if inside || (closed && x == last) {
                    n - 1
                } else {
                    n
                }
            } else {
                i32::try_from(index)
                    .map_err(|_| EvalError::Domain("interval index overflow".into()))?
            };
            Ok(Some(index))
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    Ok(Sexp::Integer(result))
}
