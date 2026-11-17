use super::*;

#[test]
fn inverse_run_length_coerces_counts_and_truncates_negative_fractions_toward_zero(
) -> Result<(), EvalError> {
    for raw in [
        Sexp::Integer(vec![Some(2), Some(0), Some(1)]),
        Sexp::Real(vec![2.9, -0.5, 1.5]),
        Sexp::String(vec![Some("2".into()), Some("0".into()), Some("1".into())]),
        Sexp::Vector(vec![
            Sexp::Integer(vec![Some(2)]),
            Sexp::Integer(vec![Some(0)]),
            Sexp::Integer(vec![Some(1)]),
        ]),
    ] {
        for counts in variants(&raw)? {
            for values in variants(&Sexp::Integer(vec![Some(7), Some(8), Some(9)]))? {
                assert_eq!(
                    decoded(counts.clone(), values)?,
                    Sexp::Integer(vec![Some(7), Some(7), Some(9)])
                );
            }
        }
    }
    Ok(())
}

#[test]
fn inverse_run_length_rejects_bad_counts_and_invalid_structures() -> Result<(), EvalError> {
    for count in [
        -1.0,
        f64::NAN,
        r_ust_sexp::NA_REAL,
        f64::INFINITY,
        4_503_599_627_370_497.0,
    ] {
        for input in variants(&Sexp::Real(vec![count]))? {
            assert!(
                matches!(decoded(input,Sexp::Integer(vec![Some(7)])),Err(EvalError::Domain(message)) if message=="invalid 'times' value")
            );
        }
    }
    for (lengths, values) in [
        (Sexp::Nil, Sexp::Integer(vec![])),
        (Sexp::Integer(vec![]), Sexp::Nil),
        (
            Sexp::Integer(vec![Some(1), Some(2)]),
            Sexp::Integer(vec![Some(7)]),
        ),
    ] {
        assert!(
            matches!(decoded(lengths,values),Err(EvalError::Domain(message)) if message=="invalid 'rle' structure")
        );
    }
    assert!(
        matches!(decoded(Sexp::Integer(vec![Some(1)]),Sexp::pairlist(vec![ListElement{name:None,value:Sexp::Nil}])),Err(EvalError::Type(message)) if message=="attempt to replicate an object of type 'pairlist'")
    );
    Ok(())
}

#[test]
fn inverse_run_length_checks_count_and_total_overflow_before_allocating() {
    for counts in [
        Sexp::Real(vec![2_147_483_648.0, 0.0]),
        Sexp::Integer(vec![Some(i32::MAX), Some(1)]),
    ] {
        assert!(matches!(
            decoded(counts, Sexp::Integer(vec![Some(1), Some(2)])),
            Err(EvalError::NotImplemented("long-vector run lengths"))
        ));
    }
}
