use super::*;
use crate::list_conversion::call_c;

#[test]
fn storage_priority_and_na_coercions_use_atomic_payloads() -> Result<(), EvalError> {
    check(
        call_c(&[
            argument(None, Sexp::Logical(vec![Some(true), None])),
            argument(None, Sexp::Integer(vec![Some(2)])),
        ])?,
        Sexp::Integer(vec![Some(1), None, Some(2)]),
        None,
    )?;
    check(
        call_c(&[
            argument(None, Sexp::Integer(vec![Some(1)])),
            argument(None, Sexp::Real(vec![2.5])),
        ])?,
        Sexp::Real(vec![1.0, 2.5]),
        None,
    )?;
    check(
        call_c(&[
            argument(None, Sexp::Integer(vec![Some(1), None])),
            argument(None, Sexp::Real(vec![2.5])),
            argument(None, Sexp::String(vec![Some("x".into())])),
        ])?,
        Sexp::String(vec![
            Some("1".into()),
            None,
            Some("2.5".into()),
            Some("x".into()),
        ]),
        None,
    )?;
    check(
        call_c(&[
            argument(
                None,
                Sexp::Real(vec![
                    f64::INFINITY,
                    f64::NAN,
                    r_ust_sexp::NA_REAL,
                    f64::NEG_INFINITY,
                ]),
            ),
            argument(None, Sexp::String(vec![Some("x".into())])),
        ])?,
        Sexp::String(vec![
            Some("Inf".into()),
            Some("NaN".into()),
            None,
            Some("-Inf".into()),
            Some("x".into()),
        ]),
        None,
    )?;
    Ok(())
}

#[test]
fn zero_length_storage_selects_type_but_does_not_attach_empty_names() -> Result<(), EvalError> {
    for (args, expected) in [
        (vec![], Sexp::Nil),
        (vec![argument(None, Sexp::Nil)], Sexp::Nil),
        (
            vec![argument(None, Sexp::Logical(vec![]))],
            Sexp::Logical(vec![]),
        ),
        (
            vec![argument(None, Sexp::Integer(vec![]))],
            Sexp::Integer(vec![]),
        ),
        (
            vec![
                argument(Some("a"), Sexp::Integer(vec![])),
                argument(Some("b"), Sexp::Real(vec![])),
            ],
            Sexp::Real(vec![]),
        ),
        (
            vec![
                argument(Some("a"), Sexp::Integer(vec![])),
                argument(Some("b"), Sexp::String(vec![])),
            ],
            Sexp::String(vec![]),
        ),
    ] {
        check(call_c(&args)?, expected, None)?;
    }
    Ok(())
}

#[test]
fn character_attributes_and_matrix_shape_are_dropped_except_names() -> Result<(), EvalError> {
    let value = Sexp::AttributedCharacter {
        values: vec![Some("x".into()), None],
        attributes: vec![
            ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![Some("a".into()), Some("b".into())]),
            },
            ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(7)]),
            },
        ],
    };
    check(
        call_c(&[argument(None, value)])?,
        Sexp::String(vec![Some("x".into()), None]),
        Some(vec![Some("a"), Some("b")]),
    )?;
    let value = Sexp::Matrix {
        data: Box::new(Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4)])),
        nrow: 2,
        ncol: 2,
    };
    check(
        call_c(&[argument(None, value)])?,
        Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4)]),
        None,
    )?;
    Ok(())
}
