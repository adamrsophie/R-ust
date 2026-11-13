use std::slice::Iter;

use super::{storage, typeof_value, EvalError, Sexp};

pub(super) enum Input<'a> {
    Integer(Integers<'a>),
    Real(&'a [f64]),
}

#[derive(Clone)]
pub(super) enum Integers<'a> {
    Logical(Iter<'a, Option<bool>>),
    Integer(Iter<'a, Option<i32>>),
    Factor(Iter<'a, Option<usize>>),
}

impl Iterator for Integers<'_> {
    type Item = Result<Option<i32>, EvalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Logical(values) => values.next().map(|value| Ok(value.map(i32::from))),
            Self::Integer(values) => values.next().copied().map(Ok),
            Self::Factor(values) => values.next().map(|value| {
                value
                    .map(i32::try_from)
                    .transpose()
                    .map_err(|_| EvalError::Domain("factor code exceeds the integer range".into()))
            }),
        }
    }
}

impl<'a> Input<'a> {
    pub(super) fn parse(value: &'a Sexp) -> Result<Self, EvalError> {
        match storage(value) {
            Sexp::Nil => Ok(Self::Integer(Integers::Integer([].iter()))),
            Sexp::Logical(values) => Ok(Self::Integer(Integers::Logical(values.iter()))),
            Sexp::Integer(values) => Ok(Self::Integer(Integers::Integer(values.iter()))),
            Sexp::Factor { codes, .. } => Ok(Self::Integer(Integers::Factor(codes.iter()))),
            Sexp::Real(values) => Ok(Self::Real(values)),
            other => Err(EvalError::Type(format!(
                "invalid 'type' ({}) of argument",
                typeof_value(other)
            ))),
        }
    }
}
