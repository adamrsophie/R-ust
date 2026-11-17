use super::*;
use crate::list_conversion::call_c;

#[test]
fn use_names_reads_first_scalar_and_keeps_default_for_na_or_invalid() -> Result<(), EvalError> {
    for (flag, use_names) in [
        (Sexp::Logical(vec![Some(false), Some(true)]), false),
        (Sexp::Integer(vec![Some(0)]), false),
        (Sexp::Real(vec![0.0]), false),
        (Sexp::String(vec![Some("false".into())]), false),
        (Sexp::Logical(vec![None]), true),
        (Sexp::Logical(vec![]), true),
        (Sexp::Nil, true),
        (Sexp::String(vec![Some("invalid".into())]), true),
        (
            Sexp::List(vec![ListElement {
                name: None,
                value: Sexp::Logical(vec![Some(false)]),
            }]),
            true,
        ),
    ] {
        check(
            call_c(&[
                argument(Some("a"), Sexp::Integer(vec![Some(1)])),
                argument(Some("use.names"), flag),
            ])?,
            Sexp::Integer(vec![Some(1)]),
            use_names.then_some(vec![Some("a")]),
        )?;
    }
    Ok(())
}

#[test]
fn controls_match_exactly_and_repeated_controls_error_after_null_compaction(
) -> Result<(), EvalError> {
    for tag in ["r", "u", "use", "use.names.extra"] {
        check(
            call_c(&[
                argument(Some("a"), Sexp::Integer(vec![Some(1)])),
                argument(Some(tag), Sexp::Logical(vec![Some(false)])),
            ])?,
            Sexp::Integer(vec![Some(1), Some(0)]),
            Some(vec![Some("a"), Some(tag)]),
        )?;
    }
    for tag in ["recursive", "use.names"] {
        let flag = argument(Some(tag), Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            call_c(&[flag.clone(), flag]),
            Err(EvalError::Domain(format!(
                "repeated formal argument '{tag}'"
            )))
        );
    }
    check(
        call_c(&[
            argument(None, Sexp::Integer(vec![Some(1)])),
            argument(Some("use.names"), Sexp::Nil),
            argument(Some("use.names"), Sexp::Logical(vec![Some(false)])),
        ])?,
        Sexp::Integer(vec![Some(1)]),
        None,
    )?;
    assert_eq!(
        call_c(&[
            argument(Some("use.names"), Sexp::Nil),
            argument(None, Sexp::Integer(vec![Some(1)])),
            argument(Some("use.names"), Sexp::Logical(vec![Some(false)]))
        ]),
        Err(EvalError::Domain(
            "repeated formal argument 'use.names'".into()
        ))
    );
    Ok(())
}

#[test]
fn recursive_atomic_and_control_only_calls_keep_atomic_semantics() -> Result<(), EvalError> {
    for flag in [
        Sexp::Logical(vec![Some(true)]),
        Sexp::Logical(vec![None]),
        Sexp::Integer(vec![]),
    ] {
        check(
            call_c(&[
                argument(Some("a"), Sexp::Integer(vec![Some(1), Some(2)])),
                argument(Some("recursive"), flag),
            ])?,
            Sexp::Integer(vec![Some(1), Some(2)]),
            Some(vec![Some("a1"), Some("a2")]),
        )?;
    }
    assert_eq!(
        call_c(&[
            argument(Some("recursive"), Sexp::Logical(vec![Some(true)])),
            argument(Some("use.names"), Sexp::Logical(vec![Some(false)]))
        ])?,
        Sexp::Nil
    );
    check(
        call_c(&[
            argument(Some("use.names"), Sexp::Logical(vec![Some(true)])),
            argument(Some("z"), Sexp::Nil),
            argument(None, Sexp::Integer(vec![Some(1)])),
        ])?,
        Sexp::Integer(vec![Some(1)]),
        None,
    )?;
    Ok(())
}
