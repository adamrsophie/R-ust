use super::classes;
use crate::{EvalError, Sexp};
use r_ust_sexp::{AtomicVector, Environment, ListElement, ValueImport, ValueObject};

#[test]
fn typed_dispatch_class_chains_keep_numeric_suffix_and_dimension_precedence(
) -> Result<(), EvalError> {
    let owner = Environment::global();
    for (dimensions, expected) in [
        (vec![], vec!["integer", "numeric"]),
        (vec![4], vec!["array", "integer", "numeric"]),
        (vec![2, 2], vec!["matrix", "array", "integer", "numeric"]),
    ] {
        let attributes = if dimensions.is_empty() {
            vec![]
        } else {
            vec![ListElement {
                name: Some("dim".into()),
                value: Sexp::Integer(dimensions.into_iter().map(Some).collect()),
            }]
        };
        let raw = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1); 4]),
            attributes,
        };
        let expected = Sexp::String(
            expected
                .into_iter()
                .map(|value| Some(value.into()))
                .collect(),
        );
        assert_eq!(classes(&raw)?, expected);
        let managed = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
        assert_eq!(classes(&managed)?, expected);
    }
    Ok(())
}

#[test]
fn managed_explicit_class_takes_precedence_over_array_and_storage() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Real(vec![1.0; 4]),
        attributes: vec![
            ListElement {
                name: Some("dim".into()),
                value: Sexp::Integer(vec![Some(2), Some(2)]),
            },
            ListElement {
                name: Some("class".into()),
                value: Sexp::String(vec![Some("first".into()), Some("second".into())]),
            },
        ],
    };
    let managed = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
    assert_eq!(
        classes(&managed)?.snapshot()?,
        Sexp::String(vec![Some("first".into()), Some("second".into())])
    );
    Ok(())
}
