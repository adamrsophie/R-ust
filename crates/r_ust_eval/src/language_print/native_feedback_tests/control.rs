use super::{call, check, expected, symbol};
use crate::language_print::{format_deparse, Profile};
use crate::{EvalError, ListElement, RuntimeState, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};

#[test]
fn managed_control_heads_formals_and_brace_lookahead_match_gnu() -> Result<(), EvalError> {
    let formals = Sexp::pairlist(vec![
        ListElement {
            name: Some("x".into()),
            value: Sexp::MissingValue,
        },
        ListElement {
            name: Some("y".into()),
            value: Sexp::Real(vec![2.0]),
        },
    ]);
    check(
        "function_formals",
        call("function", vec![formals, symbol("x"), Sexp::Nil]),
    )?;
    check(
        "nested_if",
        call(
            "{",
            vec![call(
                "if",
                vec![symbol("x"), call("{", vec![symbol("y")]), symbol("z")],
            )],
        ),
    )?;
    check(
        "for_symbol",
        call("for", vec![symbol("i"), symbol("x"), symbol("y")]),
    )?;
    check("formula", call("~", vec![symbol("x"), symbol("y")]))
}

#[test]
fn managed_formula_head_keeps_quote_expression_exception() -> Result<(), EvalError> {
    let raw = call("~", vec![symbol("x"), symbol("y")]);
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        let (lines, incomplete) = format_deparse(
            &mut RuntimeState::default(),
            value,
            Profile {
                cutoff: 60,
                backtick: false,
                control: 1027,
                lines: None,
                utf8: true,
            },
        )?;
        assert!(!incomplete);
        assert_eq!(lines.join("\n"), expected("formula_quote"));
    }
    Ok(())
}
