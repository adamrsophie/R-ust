use super::{gettext, inherits};
use crate::{EvalError, EvaluatedArgument, Sexp};
use r_ust_sexp::{AtomicVector, Environment, ListElement, ValueImport, ValueObject};

fn argument(value: Sexp) -> EvaluatedArgument {
    EvaluatedArgument { name: None, value }
}

#[test]
fn managed_inherits_reads_class_and_argument_storage() -> Result<(), EvalError> {
    let owner = Environment::global();
    let object = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7)]),
        attributes: vec![ListElement {
            name: Some("class".into()),
            value: Sexp::String(vec![Some("custom".into())]),
        }],
    };
    let what = Sexp::String(vec![Some("custom".into()), Some("absent".into())]);
    let which = Sexp::AttributedAtomic {
        values: AtomicVector::Logical(vec![Some(true)]),
        attributes: vec![],
    };
    for managed in [false, true] {
        let wrap = |value: &Sexp| -> Result<Sexp, EvalError> {
            Ok(if managed {
                Sexp::Managed(ValueObject::import(&owner, value, ValueImport::Evaluated)?)
            } else {
                value.clone()
            })
        };
        assert_eq!(
            inherits(&[
                argument(wrap(&object)?),
                argument(wrap(&what)?),
                argument(wrap(&which)?)
            ])?,
            Sexp::Integer(vec![Some(1), Some(0)])
        );
    }
    Ok(())
}

#[test]
fn untranslated_managed_gettext_retains_original_identity_and_metadata() -> Result<(), EvalError> {
    let owner = Environment::global();
    let value = ValueObject::import(
        &owner,
        &Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![Some(" kept ".into())]),
            attributes: vec![ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(7)]),
            }],
        },
        ValueImport::Evaluated,
    )?;
    let domain = ValueObject::import(&owner, &Sexp::Nil, ValueImport::Evaluated)?;
    let output = gettext(&[
        argument(Sexp::Managed(domain)),
        argument(Sexp::Managed(value.clone())),
    ])?;
    let Sexp::Managed(output) = output else {
        panic!("persistent untranslated string")
    };
    assert!(value.same_object(&output));
    Ok(())
}
