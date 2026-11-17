use super::{attribute, check};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn coercion_deparse_names_and_factor_storage_match_gnu() -> Result<(), EvalError> {
    check(
        "nonsyntactic_names",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(4)]),
            attributes: vec![
                attribute(
                    "names",
                    Sexp::String(vec![Some("not syntactic".into()), None]),
                ),
                attribute("note", Sexp::Integer(vec![Some(7)])),
            ],
        },
    )?;
    check(
        "factor",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2), None]),
            attributes: vec![
                attribute(
                    "levels",
                    Sexp::String(vec![Some("a".into()), Some("b".into())]),
                ),
                attribute("class", Sexp::String(vec![Some("factor".into())])),
            ],
        },
    )?;
    check(
        "named_sequence",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2), Some(3)]),
            attributes: vec![
                attribute(
                    "names",
                    Sexp::String(vec![Some("a".into()), Some("b".into()), Some("c".into())]),
                ),
                attribute("note", Sexp::Integer(vec![Some(7)])),
            ],
        },
    )
}

#[test]
fn coercion_deparse_one_dim_effective_names_match_gnu() -> Result<(), EvalError> {
    check(
        "one_dim",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(3)]),
            attributes: vec![
                attribute("dim", Sexp::Integer(vec![Some(2)])),
                attribute(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![Sexp::String(vec![
                            Some("a".into()),
                            Some("b".into()),
                        ])]),
                        attributes: vec![],
                    },
                ),
            ],
        },
    )
}
