use super::*;

#[test]
fn run_length_atomic_storage_and_missing_boundaries_match_gnu() -> Result<(), EvalError> {
    for (raw, lengths, values) in [
        (
            Sexp::Integer(vec![Some(1), Some(1), Some(2), Some(2), Some(2), Some(1)]),
            vec![Some(2), Some(3), Some(1)],
            Sexp::Integer(vec![Some(1), Some(2), Some(1)]),
        ),
        (
            Sexp::Logical(vec![
                Some(true),
                Some(true),
                None,
                None,
                Some(false),
                Some(false),
            ]),
            vec![Some(2), Some(1), Some(1), Some(2)],
            Sexp::Logical(vec![Some(true), None, None, Some(false)]),
        ),
        (
            Sexp::String(vec![
                Some("a".into()),
                Some("a".into()),
                None,
                None,
                Some("b".into()),
            ]),
            vec![Some(2), Some(1), Some(1), Some(1)],
            Sexp::String(vec![Some("a".into()), None, None, Some("b".into())]),
        ),
    ] {
        for input in variants(&raw)? {
            let result = encoded(input)?;
            let view = result.ordinary_list().expect("rle list");
            assert_eq!(view.elements.len(), 2);
            assert_eq!(view.elements[0].name.as_deref(), Some("lengths"));
            assert_eq!(view.elements[1].name.as_deref(), Some("values"));
            assert_eq!(view.elements[0].value, Sexp::Integer(lengths.clone()));
            assert_eq!(view.elements[1].value, values);
            assert_eq!(view.class, Some(&Sexp::String(vec![Some("rle".into())])));
        }
    }
    Ok(())
}

#[test]
fn run_length_real_nan_na_signed_zero_and_infinity_preserve_endpoints() -> Result<(), EvalError> {
    let raw = Sexp::Real(vec![
        -0.0,
        0.0,
        f64::NAN,
        f64::NAN,
        r_ust_sexp::NA_REAL,
        r_ust_sexp::NA_REAL,
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ]);
    for input in variants(&raw)? {
        let result = encoded(input)?;
        let view = result.ordinary_list().unwrap();
        assert_eq!(
            view.elements[0].value,
            Sexp::Integer(vec![
                Some(2),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(2),
                Some(1)
            ])
        );
        let Sexp::Real(values) = &view.elements[1].value else {
            panic!("real endpoints")
        };
        assert_eq!(values.len(), 7);
        assert_eq!(values[0].to_bits(), 0.0_f64.to_bits());
        assert!(values[1].is_nan() && !r_ust_sexp::is_na_real(values[1]));
        assert!(values[2].is_nan() && !r_ust_sexp::is_na_real(values[2]));
        assert!(r_ust_sexp::is_na_real(values[3]) && r_ust_sexp::is_na_real(values[4]));
        assert_eq!(&values[5..], &[f64::INFINITY, f64::NEG_INFINITY]);
    }
    Ok(())
}

#[test]
fn run_length_names_label_values_and_following_transition_lengths() -> Result<(), EvalError> {
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(1), Some(2)]),
        attributes: vec![ListElement {
            name: Some("names".into()),
            value: Sexp::String(vec![Some("a".into()), Some("b".into()), Some("c".into())]),
        }],
    };
    for input in variants(&raw)? {
        let result = encoded(input)?;
        let view = result.ordinary_list().unwrap();
        assert_eq!(
            storage::payload(&view.elements[0].value)?,
            Sexp::Integer(vec![Some(2), Some(1)])
        );
        assert_eq!(
            storage::attributes(&view.elements[0].value)?,
            vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![Some("c".into()), Some("".into())])
            }]
        );
        assert_eq!(
            storage::payload(&view.elements[1].value)?,
            Sexp::Integer(vec![Some(1), Some(2)])
        );
        assert_eq!(
            storage::attributes(&view.elements[1].value)?,
            vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![Some("b".into()), Some("c".into())])
            }]
        );
    }
    Ok(())
}
