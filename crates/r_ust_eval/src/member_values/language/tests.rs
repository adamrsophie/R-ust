use crate::member_values::{extract_assign_value as assign, extract_value as extract};
use crate::{EvalError, Parser, Sexp};
use r_ust_sexp::{canonical_actual_call, CallArgument, ListElement};

type Result = std::result::Result<(), EvalError>;

fn language(source: &str) -> Sexp {
    let parsed = Parser.parse_all(source).expect("valid source");
    canonical_actual_call(&parsed.expressions[0]).expect("canonical language")
}
fn index(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}
fn name(value: Option<&str>) -> Sexp {
    Sexp::String(vec![value.map(str::to_owned)])
}

#[test]
fn language_members_sdk_canonicalizes_parser_forms_before_read_or_write() -> Result {
    let parsed = Parser
        .parse_all("a + b")
        .expect("binary expression")
        .expressions
        .remove(0);
    assert_eq!(extract(&parsed, &index(1))?, Sexp::Symbol("+".into()));
    assert_eq!(extract(&parsed, &index(2))?, Sexp::Symbol("a".into()));
    assert_eq!(
        assign(&parsed, &index(1), &Sexp::Symbol("*".into()))?,
        language("a * b")
    );
    assert_eq!(extract(&language("(f)(a)"), &index(1))?, language("(f)"));
    Ok(())
}

#[test]
fn language_members_sdk_numeric_read_matches_integer_one_index() -> Result {
    let call = language("f(alpha=a,beta=b)");
    for (subscript, expected) in [
        (index(1), Sexp::Symbol("f".into())),
        (index(2), Sexp::Symbol("a".into())),
        (Sexp::Real(vec![1.9]), Sexp::Symbol("f".into())),
        (Sexp::Logical(vec![Some(true)]), Sexp::Symbol("f".into())),
        (Sexp::Logical(vec![None]), Sexp::Nil),
        (Sexp::Integer(vec![None]), Sexp::Nil),
        (Sexp::Real(vec![f64::NAN]), Sexp::Nil),
        (Sexp::Real(vec![f64::INFINITY]), Sexp::Nil),
    ] {
        assert_eq!(extract(&call, &subscript)?, expected, "{subscript:?}");
    }
    assert_eq!(
        extract(&language("f(a)"), &index(-1))?,
        Sexp::Symbol("a".into())
    );
    assert_eq!(
        extract(&language("f(a)"), &Sexp::Real(vec![-2.9]))?,
        Sexp::Symbol("f".into())
    );
    for (subscript, message) in [
        (
            index(0),
            "attempt to select less than one element in integerOneIndex",
        ),
        (
            index(-1),
            "attempt to select more than one element in integerOneIndex",
        ),
        (index(4), "subscript out of bounds"),
        (
            Sexp::Real(vec![-1.9]),
            "invalid negative subscript in get1index <real>",
        ),
        (
            Sexp::Real(vec![0.5]),
            "attempt to select less than one element in get1index <real>",
        ),
        (
            Sexp::Nil,
            "attempt to select less than one element in get1index",
        ),
    ] {
        assert_eq!(
            extract(&call, &subscript),
            Err(EvalError::Domain(message.into()))
        );
    }
    Ok(())
}

#[test]
fn language_members_sdk_exact_names_and_symbols_preserve_first_tag() -> Result {
    let call = language("f(alpha=a,alpha=b)");
    for subscript in [name(Some("alpha")), Sexp::Symbol("alpha".into())] {
        assert_eq!(extract(&call, &subscript)?, Sexp::Symbol("a".into()));
    }
    for subscript in [name(Some("al")), name(Some("")), name(None)] {
        assert_eq!(extract(&call, &subscript)?, Sexp::Nil);
    }
    assert_eq!(
        assign(&call, &name(Some("alpha")), &Sexp::Symbol("z".into()))?,
        language("f(alpha=z,alpha=b)")
    );
    Ok(())
}

#[test]
fn language_members_sdk_assignment_extends_with_null_and_tagged_cells() -> Result {
    let call = language("f(a)");
    let rhs = Sexp::Symbol("z".into());
    assert_eq!(assign(&call, &index(4), &rhs)?, language("f(a,NULL,z)"));
    assert_eq!(
        assign(&call, &name(Some("new")), &rhs)?,
        language("f(a,new=z)")
    );
    assert_eq!(assign(&call, &name(None), &rhs)?, language("f(a,`NA`=z)"));
    assert_eq!(
        assign(&call, &index(1), &index(17))?,
        Sexp::Call {
            function: Box::new(index(17)),
            arguments: vec![CallArgument::positional(Sexp::Symbol("a".into()))],
        }
    );
    assert_eq!(
        assign(&call, &name(Some("")), &rhs),
        Err(EvalError::Domain(
            "attempt to use zero-length variable name".into()
        ))
    );
    Ok(())
}

#[test]
fn language_members_sdk_assignment_logical_negative_and_na_rules() -> Result {
    let call = language("f(a,b)");
    let rhs = Sexp::Symbol("z".into());
    for subscript in [
        index(0),
        Sexp::Logical(vec![Some(false)]),
        Sexp::Integer(vec![None]),
        Sexp::Real(vec![f64::NAN]),
        Sexp::Real(vec![f64::INFINITY]),
        Sexp::Real(vec![f64::NEG_INFINITY]),
        Sexp::Real(vec![0.5]),
    ] {
        assert_eq!(assign(&call, &subscript, &rhs)?, call, "{subscript:?}");
    }
    for subscript in [
        index(-1),
        index(-4),
        Sexp::Logical(vec![Some(true)]),
        Sexp::Logical(vec![None]),
    ] {
        assert_eq!(
            assign(&call, &subscript, &rhs),
            Err(EvalError::Domain("invalid subscript in list assign".into()))
        );
    }
    assert_eq!(
        assign(&language("f(a)"), &index(-1), &rhs)?,
        language("f(z)")
    );
    assert_eq!(assign(&language("f()"), &index(-2), &rhs)?, language("z()"));
    assert_eq!(assign(&language("f()"), &index(-1), &rhs)?, language("f()"));
    assert_eq!(
        assign(&call, &Sexp::Nil, &rhs),
        Err(EvalError::Domain(
            "internal error: index -1 from length 0".into()
        ))
    );
    Ok(())
}

#[test]
fn language_members_sdk_deletion_preserves_cell_type_and_tags() -> Result {
    let call = language("f(alpha=a,beta=b)");
    assert_eq!(assign(&call, &index(2), &Sexp::Nil)?, language("f(beta=b)"));
    assert_eq!(assign(&call, &index(-1), &Sexp::Nil)?, language("f()"));
    assert_eq!(
        assign(&call, &index(1), &Sexp::Nil)?,
        Sexp::pairlist(vec![
            ListElement {
                name: Some("alpha".into()),
                value: Sexp::Symbol("a".into())
            },
            ListElement {
                name: Some("beta".into()),
                value: Sexp::Symbol("b".into())
            },
        ])
    );
    assert_eq!(
        assign(&call, &index(-2), &Sexp::Nil)?,
        Sexp::pairlist(vec![ListElement {
            name: Some("alpha".into()),
            value: Sexp::Symbol("a".into())
        },])
    );
    for subscript in [index(-4), Sexp::Logical(vec![Some(true)])] {
        assert_eq!(assign(&call, &subscript, &Sexp::Nil)?, Sexp::Nil);
    }
    for subscript in [
        index(0),
        Sexp::Logical(vec![None]),
        Sexp::Real(vec![f64::NAN]),
    ] {
        assert_eq!(assign(&call, &subscript, &Sexp::Nil)?, call);
    }
    for subscript in [index(4), name(Some("unknown")), name(None), name(Some(""))] {
        assert_eq!(
            assign(&call, &subscript, &Sexp::Nil),
            Err(EvalError::Domain("subscript out of bounds".into()))
        );
    }
    Ok(())
}

mod lifetime;
