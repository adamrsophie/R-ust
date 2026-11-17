use super::{
    support::layout,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_zero_dimension_layouts() -> Result<(), EvalError> {
    for (name, dimensions) in [
        ("zero_columns", vec![3, 0]),
        ("zero_both", vec![0, 0]),
        ("zero_array", vec![2, 2, 0]),
        ("zero_array_rows", vec![0, 2, 2]),
    ] {
        layout(
            name,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(vec![]),
                attributes: vec![a("dim", i(&dimensions))],
            },
        )?;
    }
    Ok(())
}
