use super::*;

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
