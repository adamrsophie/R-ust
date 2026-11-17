use super::expected;
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{Environment, ListObject, ValueImport, ValueObject};

#[test]
fn managed_print_mp3_explicit_wrapper_cycles_are_rejected_at_direct_entry() -> Result<(), EvalError>
{
    let owner = Environment::default();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let handle = ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?;
    let wrapper = Sexp::Managed(handle);
    list.replace_shared(&[(0, wrapper.clone())])?;
    for value in [&wrapper, &Sexp::ListObject(list.clone())] {
        assert_eq!(
            super::super::format_sexp(value),
            Err(EvalError::NotImplemented("cyclic ordinary value printing"))
        );
        assert_eq!(
            super::super::character::format(value, true),
            Err(EvalError::NotImplemented("cyclic ordinary value printing"))
        );
    }
    list.replace_shared(&[(0, Sexp::Integer(vec![Some(1)]))])?;
    assert_eq!(
        super::super::format_sexp(&wrapper)?,
        expected("acyclic_list")
    );
    Ok(())
}

#[test]
fn managed_print_mp3_acyclic_explicit_wrappers_and_sibling_aliases_remain_printable(
) -> Result<(), EvalError> {
    let owner = Environment::default();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Integer(vec![Some(1)]),
        }]),
    )?;
    let handle = ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?;
    let wrapper = Sexp::Managed(handle);
    let before = list.semantic_reference_count()?;
    for value in [&wrapper, &Sexp::ListObject(list.clone())] {
        assert_eq!(super::super::format_sexp(value)?, expected("acyclic_list"));
        assert_eq!(
            super::super::character::format(value, true)?,
            expected("acyclic_list")
        );
    }
    let aliases = Sexp::List(vec![
        ListElement {
            name: None,
            value: wrapper.clone(),
        },
        ListElement {
            name: None,
            value: wrapper,
        },
    ]);
    let ordinary = Sexp::List(vec![
        ListElement {
            name: None,
            value: Sexp::ListObject(list.clone()),
        },
        ListElement {
            name: None,
            value: Sexp::ListObject(list.clone()),
        },
    ]);
    assert_eq!(
        super::super::format_sexp(&aliases)?,
        super::super::format_sexp(&ordinary)?
    );
    assert_eq!(list.semantic_reference_count()?, before);
    Ok(())
}
