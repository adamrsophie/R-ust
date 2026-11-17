use super::managed;
use crate::coercion::{atomic, strings::BlankPolicy, warnings::Warnings};
use crate::{EvalError, Sexp};
use r_ust_sexp::{is_na_real, AtomicVector, Environment, NA_REAL};

#[test]
fn managed_scalar_first_element_and_string_warning_categories() -> Result<(), EvalError> {
    let owner = Environment::global();
    for (value, expected, generic, range) in [
        (
            Sexp::String(vec![Some("7".into()), Some("bad".into())]),
            Some(7),
            false,
            false,
        ),
        (Sexp::String(vec![Some("bad".into())]), None, true, false),
        (
            Sexp::String(vec![Some("2147483648".into())]),
            None,
            false,
            true,
        ),
        (Sexp::Char(Some("8".into())), Some(8), false, false),
        (Sexp::Real(vec![7.9, f64::INFINITY]), Some(7), false, false),
        (Sexp::Real(vec![NA_REAL]), None, false, false),
        (Sexp::Real(vec![f64::NAN]), None, false, false),
        (Sexp::Integer(vec![]), None, false, false),
    ] {
        for input in [value.clone(), managed(&owner, &value)?] {
            let (actual, warnings) = atomic::scalar_integer(&input, BlankPolicy::C);
            assert_eq!(actual, expected);
            assert_eq!((warnings.generic, warnings.range), (generic, range));
        }
    }
    Ok(())
}

#[test]
fn attributed_atomic_storage_preserves_real_bits_and_logical_first_element() -> Result<(), EvalError>
{
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Real(vec![NA_REAL, f64::NAN, -0.0, 2.5]),
        attributes: vec![],
    };
    for input in [raw.clone(), managed(&owner, &raw)?] {
        let (values, warnings) = atomic::reals(&input, BlankPolicy::C)?;
        assert!(is_na_real(values[0]));
        assert!(values[1].is_nan() && !is_na_real(values[1]));
        assert_eq!(values[2].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(values[3], 2.5);
        assert_eq!(warnings, Warnings::default());
        assert_eq!(crate::internal::as_logical(&input), None);
    }
    for (raw, expected) in [
        (Sexp::Logical(vec![Some(true), None]), Some(true)),
        (Sexp::Real(vec![-0.0, 1.0]), Some(false)),
        (Sexp::String(vec![Some("True".into()), None]), Some(true)),
        (Sexp::String(vec![Some("yes".into())]), None),
    ] {
        assert_eq!(
            crate::internal::as_logical(&managed(&owner, &raw)?),
            expected
        );
    }
    Ok(())
}

#[test]
fn managed_factor_numeric_coercion_uses_codes() -> Result<(), EvalError> {
    let owner = Environment::global();
    let factor = Sexp::Factor {
        codes: vec![Some(2), None, Some(1)],
        levels: vec!["low".into(), "high".into()],
        ordered: false,
    };
    let input = managed(&owner, &factor)?;
    assert_eq!(
        atomic::integers(&input, BlankPolicy::C)?.0,
        [Some(2), None, Some(1)]
    );
    assert_eq!(atomic::scalar_integer(&input, BlankPolicy::C).0, Some(2));
    assert_eq!(crate::internal::as_logical(&input), Some(true));
    Ok(())
}
