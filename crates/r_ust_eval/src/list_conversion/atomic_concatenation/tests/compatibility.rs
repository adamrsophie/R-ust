use super::*;
use crate::list_conversion::call_c;

#[test]
fn pure_factor_legacy_kernel_retains_levels_and_codes() -> Result<(), EvalError> {
    let value = call_c(&[
        argument(
            None,
            Sexp::Factor {
                codes: vec![Some(1), Some(2)],
                levels: vec!["a".into(), "b".into()],
                ordered: false,
            },
        ),
        argument(
            None,
            Sexp::Factor {
                codes: vec![Some(1)],
                levels: vec!["b".into()],
                ordered: false,
            },
        ),
    ])?;
    assert_eq!(
        value,
        Sexp::Factor {
            codes: vec![Some(1), Some(2), Some(2)],
            levels: vec!["a".into(), "b".into()],
            ordered: false
        }
    );
    Ok(())
}

#[test]
fn legacy_nonrecursive_lists_keep_existing_name_bridge() -> Result<(), EvalError> {
    let list = Sexp::List(vec![
        ListElement {
            name: Some("x".into()),
            value: Sexp::Integer(vec![Some(1)]),
        },
        ListElement {
            name: Some("y".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
    ]);
    let value = call_c(&[
        argument(Some("a"), list),
        argument(Some("z"), Sexp::Integer(vec![Some(3)])),
    ])?;
    let view = value.ordinary_list().expect("legacy list output");
    assert!(view.names_present);
    assert_eq!(
        view.elements
            .iter()
            .map(|element| element.name.as_deref())
            .collect::<Vec<_>>(),
        [Some("a.x"), Some("a.y"), Some("z")]
    );
    assert_eq!(view.elements[2].value, Sexp::Integer(vec![Some(3)]));
    Ok(())
}

#[test]
fn recursive_list_boundary_remains_explicit() {
    let list = Sexp::List(vec![ListElement {
        name: Some("x".into()),
        value: Sexp::Integer(vec![Some(1)]),
    }]);
    assert_eq!(
        call_c(&[
            argument(None, list),
            argument(Some("recursive"), Sexp::Logical(vec![Some(true)]))
        ]),
        Err(EvalError::NotImplemented(
            "recursive concatenation with list names"
        ))
    );
}
