use crate::{ListElement, Sexp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Left,
    Equal,
}

impl AssignmentOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "<-",
            Self::Equal => "=",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberName {
    Symbol(String),
    String(String),
}

impl MemberName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Symbol(name) | Self::String(name) => name,
        }
    }

    pub fn to_sexp(&self) -> Sexp {
        match self {
            Self::Symbol(name) => Sexp::Symbol(name.clone()),
            Self::String(name) => Sexp::String(vec![Some(name.clone())]),
        }
    }
}

/// A nonempty proper pairlist; use `Sexp::pairlist` to normalize empty lists.
#[derive(Clone, Debug, PartialEq)]
pub struct PairList(Vec<ListElement>);

impl PairList {
    pub fn as_slice(&self) -> &[ListElement] {
        &self.0
    }

    pub(crate) fn elements_mut(&mut self) -> &mut [ListElement] {
        &mut self.0
    }
}

impl Sexp {
    pub fn pairlist(elements: Vec<ListElement>) -> Self {
        if elements.is_empty() {
            Self::Nil
        } else {
            Self::PairList(PairList(elements))
        }
    }
}

pub(super) fn binary_name(op: crate::BinaryOp) -> &'static str {
    match op {
        crate::BinaryOp::Add => "+",
        crate::BinaryOp::Subtract => "-",
        crate::BinaryOp::Multiply => "*",
        crate::BinaryOp::Divide => "/",
        crate::BinaryOp::Modulo => "%%",
        crate::BinaryOp::IntegerDivide => "%/%",
        crate::BinaryOp::Power => "^",
        crate::BinaryOp::Less => "<",
        crate::BinaryOp::LessEqual => "<=",
        crate::BinaryOp::Greater => ">",
        crate::BinaryOp::GreaterEqual => ">=",
        crate::BinaryOp::Equal => "==",
        crate::BinaryOp::NotEqual => "!=",
        crate::BinaryOp::And => "&",
        crate::BinaryOp::Or => "|",
        crate::BinaryOp::AndScalar => "&&",
        crate::BinaryOp::OrScalar => "||",
        crate::BinaryOp::In => "%in%",
        crate::BinaryOp::Sequence => ":",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageError {
    InternalCall,
    RuntimeValue,
}

impl std::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InternalCall => "internal dispatch has no public language representation",
            Self::RuntimeValue => "runtime value is not parser-origin language",
        })
    }
}

impl std::error::Error for LanguageError {}
