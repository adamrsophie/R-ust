use super::*;
use crate::list_conversion::call_c;

#[test]
fn atomic_names_keep_outer_prefix_blank_na_and_duplicate_tags() -> Result<(), EvalError> {
    let data = named(
        vec![Some(1), Some(2), Some(3)],
        vec![None, Some(""), Some("x")],
    );
    for (tag, names) in [
        (None, vec![None, Some(""), Some("x")]),
        (
            Some("outer"),
            vec![Some("outer.NA"), Some("outer2"), Some("outer.x")],
        ),
    ] {
        check(
            call_c(&[argument(tag, data.clone())])?,
            Sexp::Integer(vec![Some(1), Some(2), Some(3)]),
            Some(names),
        )?;
    }
    check(
        call_c(&[
            argument(Some("a"), Sexp::Integer(vec![Some(1)])),
            argument(Some("a"), Sexp::Integer(vec![Some(2)])),
        ])?,
        Sexp::Integer(vec![Some(1), Some(2)]),
        Some(vec![Some("a"), Some("a")]),
    )?;
    for (value, expected) in [
        (Sexp::Integer(vec![Some(1)]), vec![Some("outer")]),
        (named(vec![Some(1)], vec![Some("")]), vec![Some("outer")]),
        (named(vec![Some(1)], vec![None]), vec![Some("outer.NA")]),
        (
            Sexp::Integer(vec![Some(1), Some(2)]),
            vec![Some("outer1"), Some("outer2")],
        ),
    ] {
        check(
            call_c(&[argument(Some("outer"), value.clone())])?,
            crate::list_attributes::engine::payload(&value)?,
            Some(expected),
        )?;
    }
    Ok(())
}

#[test]
fn zero_length_names_and_first_null_control_name_presence() -> Result<(), EvalError> {
    for empty in [named(vec![], vec![]), Sexp::Integer(vec![])] {
        check(
            call_c(&[
                argument(Some("empty"), empty),
                argument(None, Sexp::Integer(vec![Some(1)])),
            ])?,
            Sexp::Integer(vec![Some(1)]),
            Some(vec![Some("")]),
        )?;
    }
    check(
        call_c(&[
            argument(None, named(vec![], vec![])),
            argument(None, Sexp::Integer(vec![Some(1)])),
        ])?,
        Sexp::Integer(vec![Some(1)]),
        Some(vec![Some("")]),
    )?;
    check(
        call_c(&[
            argument(Some("z"), Sexp::Nil),
            argument(None, Sexp::Integer(vec![Some(1)])),
        ])?,
        Sexp::Integer(vec![Some(1)]),
        Some(vec![Some("")]),
    )?;
    check(
        call_c(&[
            argument(None, Sexp::Integer(vec![Some(1)])),
            argument(Some("z"), Sexp::Nil),
        ])?,
        Sexp::Integer(vec![Some(1)]),
        None,
    )?;
    check(
        call_c(&[argument(Some(""), Sexp::Integer(vec![Some(1)]))])?,
        Sexp::Integer(vec![Some(1)]),
        None,
    )?;
    check(
        call_c(&[argument(None, named(vec![], vec![]))])?,
        Sexp::Integer(vec![]),
        None,
    )?;
    Ok(())
}

#[test]
fn one_dimensional_dimnames_supply_names_while_other_metadata_is_dropped() -> Result<(), EvalError>
{
    let value = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(2)]),
        attributes: vec![
            ListElement {
                name: Some("dim".into()),
                value: Sexp::Integer(vec![Some(2)]),
            },
            ListElement {
                name: Some("dimnames".into()),
                value: Sexp::List(vec![ListElement {
                    name: None,
                    value: Sexp::String(vec![Some("x".into()), Some("y".into())]),
                }]),
            },
            ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(7)]),
            },
        ],
    };
    check(
        call_c(&[argument(None, value)])?,
        Sexp::Integer(vec![Some(1), Some(2)]),
        Some(vec![Some("x"), Some("y")]),
    )?;
    let mut value = named(vec![Some(1), Some(2)], vec![Some("a"), Some("b")]);
    let Sexp::AttributedAtomic { attributes, .. } = &mut value else {
        unreachable!()
    };
    attributes.push(ListElement {
        name: Some("class".into()),
        value: Sexp::String(vec![Some("custom".into())]),
    });
    attributes.push(ListElement {
        name: Some("note".into()),
        value: Sexp::Integer(vec![Some(7)]),
    });
    check(
        call_c(&[argument(None, value)])?,
        Sexp::Integer(vec![Some(1), Some(2)]),
        Some(vec![Some("a"), Some("b")]),
    )?;
    Ok(())
}
