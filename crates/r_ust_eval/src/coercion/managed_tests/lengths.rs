use super::managed;
use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{AtomicVector, Environment};

#[test]
fn managed_vector_length_requires_one_supported_element() -> Result<(), EvalError> {
    let owner = Environment::global();
    let mut runtime = RuntimeState::default();
    for value in [
        Sexp::Integer(vec![]),
        Sexp::Integer(vec![Some(2), Some(3)]),
        Sexp::Real(vec![]),
        Sexp::Real(vec![2.0, 3.0]),
        Sexp::Logical(vec![Some(true)]),
        Sexp::Vector(vec![]),
    ] {
        let input = managed(&owner, &value)?;
        assert_eq!(
            runtime.vector_length(&input).unwrap_err().to_string(),
            "domain error: invalid 'length' argument"
        );
    }
    for (value, expected) in [
        (Sexp::Integer(vec![Some(3)]), 3),
        (Sexp::Real(vec![3.8]), 3),
        (Sexp::Real(vec![-0.5]), 0),
        (Sexp::String(vec![Some("4".into())]), 4),
        (
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(vec![Some(5)]),
                attributes: vec![],
            },
            5,
        ),
    ] {
        assert_eq!(runtime.vector_length(&managed(&owner, &value)?)?, expected);
    }
    Ok(())
}

#[test]
fn managed_vector_length_reports_nonfinite_negative_and_missing_errors() -> Result<(), EvalError> {
    let owner = Environment::global();
    let mut runtime = RuntimeState::default();
    for (value, message) in [
        (Sexp::Integer(vec![None]), "vector size cannot be NA"),
        (Sexp::Real(vec![f64::NAN]), "vector size cannot be NA/NaN"),
        (
            Sexp::Real(vec![f64::INFINITY]),
            "vector size cannot be infinite",
        ),
        (Sexp::Real(vec![-1.0]), "invalid 'length' argument"),
        (
            Sexp::Real(vec![4_503_599_627_370_497.0]),
            "vector size specified is too large",
        ),
    ] {
        assert_eq!(
            runtime
                .vector_length(&managed(&owner, &value)?)
                .unwrap_err()
                .to_string(),
            format!("domain error: {message}")
        );
    }
    Ok(())
}
