use super::*;

mod arguments;

pub(super) fn call(args: &[EvaluatedArgument], call: &Sexp) -> Result<Sexp, EvalError> {
    let inputs = arguments::read(args, call)?;
    let [num_eq, single_na, attrib_as_set, ignore_bytecode, ignore_environment, ignore_srcref, extptr_as_ref] =
        inputs.flags;
    if !num_eq || !single_na || ignore_environment || extptr_as_ref {
        return Err(EvalError::NotImplemented(
            "nondefault identical comparison option",
        ));
    }
    let mut comparison = crate::shared_identity::Comparison::default();
    comparison.use_srcref = !ignore_srcref;
    comparison.use_bytecode = !ignore_bytecode;
    comparison.ordered_attributes = !attrib_as_set;
    Ok(Sexp::Logical(vec![Some(
        comparison.compare(inputs.left, inputs.right)?,
    )]))
}

pub(super) fn sexp_identical(left: &Sexp, right: &Sexp) -> Result<bool, EvalError> {
    crate::shared_identity::Comparison::default().compare(left, right)
}

pub(super) fn real_values_identical(left: &[f64], right: &[f64]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left == right
                || (left.is_nan() && right.is_nan() && is_na_real(*left) == is_na_real(*right))
        })
}
