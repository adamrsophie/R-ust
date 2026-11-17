use super::managed;
use crate::{EvalError, Frontend, Sexp};
use r_ust_sexp::{AtomicVector, ListElement};

#[test]
fn public_bindings_coerce_managed_atomic_and_list_cells() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let owner = frontend.runtime().global.clone();
    let input = managed(
        &owner,
        &Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![3.9, -2.2]),
            attributes: vec![ListElement {
                name: Some("class".into()),
                value: Sexp::String(vec![Some("numeric_box".into())]),
            }],
        },
    )?;
    owner.insert("x".into(), &input)?;
    let result = frontend.eval_source("as.integer(x)")?;
    assert_eq!(
        result.last().unwrap().snapshot()?,
        Sexp::Integer(vec![Some(3), Some(-2)])
    );
    let list = managed(
        &owner,
        &Sexp::Vector(vec![
            Sexp::String(vec![Some("8".into())]),
            Sexp::Integer(vec![None]),
        ]),
    )?;
    owner.insert("x".into(), &list)?;
    let result = frontend.eval_source("as.integer(x)")?;
    assert_eq!(
        result.last().unwrap().snapshot()?,
        Sexp::Integer(vec![Some(8), None])
    );
    Ok(())
}

#[test]
fn public_vector_length_reads_managed_integer_storage() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let owner = frontend.runtime().global.clone();
    owner.insert("n".into(), &managed(&owner, &Sexp::Integer(vec![Some(3)]))?)?;
    let result = frontend.eval_source("vector(\"integer\", n)")?;
    assert_eq!(
        result.last().unwrap().snapshot()?,
        Sexp::Integer(vec![Some(0); 3])
    );
    Ok(())
}

#[test]
fn public_factor_character_coercion_uses_levels() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let owner = frontend.runtime().global.clone();
    let factor = Sexp::Factor {
        codes: vec![Some(2), None, Some(1)],
        levels: vec!["low".into(), "high".into()],
        ordered: false,
    };
    owner.insert("x".into(), &managed(&owner, &factor)?)?;
    let result = frontend.eval_source("as.character(x)")?;
    assert_eq!(
        result.last().unwrap().snapshot()?,
        Sexp::String(vec![Some("high".into()), None, Some("low".into())])
    );
    Ok(())
}
