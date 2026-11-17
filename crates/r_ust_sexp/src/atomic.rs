use crate::{Sexp, SexpType};

/// Typed atomic storage for snapshots with attributes; no payload is coerced.
#[derive(Clone, Debug, PartialEq)]
pub enum AtomicVector {
    Logical(Vec<Option<bool>>),
    Integer(Vec<Option<i32>>),
    Real(Vec<f64>),
    Raw(Vec<u8>),
    String(Vec<Option<String>>),
}

impl AtomicVector {
    pub const fn sexptype(&self) -> SexpType {
        match self {
            Self::Logical(_) => SexpType::Logical,
            Self::Integer(_) => SexpType::Integer,
            Self::Real(_) => SexpType::Real,
            Self::Raw(_) => SexpType::Raw,
            Self::String(_) => SexpType::String,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Logical(values) => values.len(),
            Self::Integer(values) => values.len(),
            Self::Real(values) => values.len(),
            Self::Raw(values) => values.len(),
            Self::String(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn into_value(self) -> Sexp {
        match self {
            Self::Logical(values) => Sexp::Logical(values),
            Self::Integer(values) => Sexp::Integer(values),
            Self::Real(values) => Sexp::Real(values),
            Self::Raw(values) => Sexp::Raw(values),
            Self::String(values) => Sexp::String(values),
        }
    }
}
