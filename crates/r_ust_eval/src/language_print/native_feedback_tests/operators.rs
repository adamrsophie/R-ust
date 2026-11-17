use super::{call, check, symbol};
use crate::{CallArgument, EvalError, Sexp};

#[test]
fn managed_call_heads_and_nested_precedence_match_gnu() -> Result<(), EvalError> {
    for (name, value) in [
        ("sum", call("+", vec![symbol("x"), Sexp::Real(vec![2.0])])),
        (
            "product_of_sum",
            call(
                "*",
                vec![
                    call("+", vec![symbol("x"), Sexp::Real(vec![2.0])]),
                    symbol("y"),
                ],
            ),
        ),
        (
            "right_difference",
            call(
                "-",
                vec![symbol("x"), call("-", vec![symbol("y"), symbol("z")])],
            ),
        ),
        (
            "left_power",
            call(
                "^",
                vec![call("^", vec![symbol("x"), symbol("y")]), symbol("z")],
            ),
        ),
        (
            "unary_power",
            call("^", vec![call("-", vec![symbol("x")]), symbol("y")]),
        ),
        (
            "subset_sum",
            call(
                "[",
                vec![call("+", vec![symbol("x"), symbol("y")]), symbol("i")],
            ),
        ),
    ] {
        check(name, value)?;
    }
    Ok(())
}

#[test]
fn managed_dollar_members_and_computed_heads_match_gnu() -> Result<(), EvalError> {
    check(
        "dollar_string",
        call(
            "$",
            vec![symbol("x"), Sexp::String(vec![Some("simple".into())])],
        ),
    )?;
    check(
        "dollar_unusual",
        call(
            "$",
            vec![symbol("x"), Sexp::String(vec![Some("a b".into())])],
        ),
    )?;
    check("dollar_symbol", call("$", vec![symbol("x"), symbol("y")]))?;
    check(
        "computed_head",
        Sexp::Call {
            function: Box::new(call("+", vec![symbol("x"), symbol("y")])),
            arguments: vec![CallArgument::positional(symbol("z"))],
        },
    )?;
    check(
        "named_operator",
        Sexp::Call {
            function: Box::new(symbol("%foo%")),
            arguments: vec![
                CallArgument {
                    name: Some("a".into()),
                    value: symbol("x"),
                },
                CallArgument {
                    name: Some("b".into()),
                    value: symbol("y"),
                },
            ],
        },
    )
}
