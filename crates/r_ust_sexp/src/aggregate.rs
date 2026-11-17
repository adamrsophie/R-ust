use crate::{Sexp, SexpType};

/// Typed aggregate payload; names and other metadata live in the attribute container.
#[derive(Clone, Debug, PartialEq)]
pub enum AggregateVector {
    Vector(Vec<Sexp>),
    Expression(Vec<Sexp>),
}

impl AggregateVector {
    pub const fn sexptype(&self) -> SexpType {
        match self {
            Self::Vector(_) => SexpType::Vector,
            Self::Expression(_) => SexpType::Expression,
        }
    }

    pub fn values(&self) -> &[Sexp] {
        match self {
            Self::Vector(values) | Self::Expression(values) => values,
        }
    }

    pub fn values_mut(&mut self) -> &mut [Sexp] {
        match self {
            Self::Vector(values) | Self::Expression(values) => values,
        }
    }

    pub fn len(&self) -> usize {
        self.values().len()
    }
    pub fn is_empty(&self) -> bool {
        self.values().is_empty()
    }
}
