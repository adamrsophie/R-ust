use crate::{EvalError, EvaluatedArgument, ListElement, Sexp};
use r_ust_sexp::AtomicVector;

mod compatibility;
mod controls;
mod managed;
mod naming;
mod public;
mod storage;

fn argument(name: Option<&str>, value: Sexp) -> EvaluatedArgument {
    EvaluatedArgument {
        name: name.map(str::to_owned),
        value,
    }
}

fn named(values: Vec<Option<i32>>, names: Vec<Option<&str>>) -> Sexp {
    Sexp::AttributedAtomic {
        values: AtomicVector::Integer(values),
        attributes: vec![ListElement {
            name: Some("names".into()),
            value: Sexp::String(
                names
                    .into_iter()
                    .map(|name| name.map(str::to_owned))
                    .collect(),
            ),
        }],
    }
}

fn check(value: Sexp, payload: Sexp, names: Option<Vec<Option<&str>>>) -> Result<(), EvalError> {
    assert_eq!(crate::list_attributes::engine::payload(&value)?, payload);
    let expected = names
        .map(|names| {
            vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(
                    names
                        .into_iter()
                        .map(|name| name.map(str::to_owned))
                        .collect(),
                ),
            }]
        })
        .unwrap_or_default();
    assert_eq!(crate::list_attributes::engine::stored(&value)?, expected);
    Ok(())
}
