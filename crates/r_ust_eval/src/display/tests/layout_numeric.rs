use super::{support::layout, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_rounding_and_subnormal_layout() -> Result<(), EvalError> {
    for (name, values) in [
        (
            "rounding_widens",
            vec![9999999.6, 0.00099999996, 9.9999996, 9.999999e99],
        ),
        ("subnormal", vec![5e-324, 1e-300, 1e300]),
    ] {
        layout(
            name,
            Sexp::AttributedAtomic {
                values: AtomicVector::Real(values),
                attributes: vec![a("note", Sexp::Logical(vec![Some(true)]))],
            },
        )?;
    }
    Ok(())
}
