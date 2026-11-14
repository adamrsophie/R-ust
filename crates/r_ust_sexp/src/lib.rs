#![allow(non_camel_case_types)]

mod graph;
mod language;
mod language_types;
mod operators;
mod primitive;
mod promise;
mod value;

pub use language::canonical_language;
pub use language_types::{AssignmentOperator, LanguageError, MemberName, PairList};
pub use primitive::PrimitiveId;
pub use value::{ClosureEnvironment, InternalOperation, Sexp};

pub use graph::{Environment, EnvironmentRole, Force, ForceLease, GraphError, PromiseState};
pub use operators::BinaryOp;
pub use promise::{EvaluationEnvironmentId, EvaluationOrigin, EvaluationSessionId};

pub const R_VERSION: &str = "4.6.0";

// R's arithmetic.c identifies NA by the low payload word, even after NaN quieting.
pub const NA_REAL: f64 = f64::from_bits(0x7ff0_0000_0000_07a2);

pub fn is_na_real(value: f64) -> bool {
    value.is_nan() && value.to_bits() as u32 == 1954
}

pub type R_xlen_t = isize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SexpType {
    Nil = 0,
    Symbol = 1,
    List = 2,
    Closure = 3,
    Environment = 4,
    Promise = 5,
    Language = 6,
    Special = 7,
    Builtin = 8,
    Char = 9,
    Logical = 10,
    Integer = 13,
    Real = 14,
    Complex = 15,
    String = 16,
    Dots = 17,
    Any = 18,
    Vector = 19,
    Expression = 20,
    Bytecode = 21,
    ExternalPtr = 22,
    WeakRef = 23,
    Raw = 24,
    Object = 25,
    New = 30,
    Free = 31,
    Function = 99,
}

impl SexpType {
    pub const fn from_r_type(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Nil),
            1 => Some(Self::Symbol),
            2 => Some(Self::List),
            3 => Some(Self::Closure),
            4 => Some(Self::Environment),
            5 => Some(Self::Promise),
            6 => Some(Self::Language),
            7 => Some(Self::Special),
            8 => Some(Self::Builtin),
            9 => Some(Self::Char),
            10 => Some(Self::Logical),
            13 => Some(Self::Integer),
            14 => Some(Self::Real),
            15 => Some(Self::Complex),
            16 => Some(Self::String),
            17 => Some(Self::Dots),
            18 => Some(Self::Any),
            19 => Some(Self::Vector),
            20 => Some(Self::Expression),
            21 => Some(Self::Bytecode),
            22 => Some(Self::ExternalPtr),
            23 => Some(Self::WeakRef),
            24 => Some(Self::Raw),
            25 => Some(Self::Object),
            30 => Some(Self::New),
            31 => Some(Self::Free),
            99 => Some(Self::Function),
            _ => None,
        }
    }

    pub const fn as_r_type(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SexpHeader {
    sexptype: SexpType,
    named: u8,
    object: bool,
    altrep: bool,
    gp: u16,
}

impl SexpHeader {
    pub const fn new(sexptype: SexpType) -> Self {
        Self {
            sexptype,
            named: 0,
            object: false,
            altrep: false,
            gp: 0,
        }
    }

    pub const fn sexptype(&self) -> SexpType {
        self.sexptype
    }

    pub const fn named(&self) -> u8 {
        self.named
    }

    pub const fn is_object(&self) -> bool {
        self.object
    }

    pub const fn is_altrep(&self) -> bool {
        self.altrep
    }

    pub const fn gp(&self) -> u16 {
        self.gp
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallArgument {
    pub name: Option<String>,
    pub value: Sexp,
}

impl CallArgument {
    pub fn positional(value: Sexp) -> Self {
        Self { name: None, value }
    }

    pub fn named(name: impl Into<String>, value: Sexp) -> Self {
        Self {
            name: Some(name.into()),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub default: Option<Sexp>,
}

impl FunctionParameter {
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn optional(name: impl Into<String>, default: Sexp) -> Self {
        Self {
            name: name.into(),
            default: Some(default),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListElement {
    pub name: Option<String>,
    pub value: Sexp,
}

impl Sexp {
    pub const fn sexptype(&self) -> SexpType {
        match self {
            Self::Nil => SexpType::Nil,
            Self::Primitive(id) => id.sexptype(),
            Self::MissingArg(_) => SexpType::Promise,
            Self::Symbol(_) | Self::MissingValue => SexpType::Symbol,
            Self::Logical(_) => SexpType::Logical,
            Self::Integer(_) => SexpType::Integer,
            Self::Real(_) => SexpType::Real,
            Self::String(_) => SexpType::String,
            Self::Vector(_) => SexpType::Vector,
            Self::List(_) | Self::PairList(_) => SexpType::List,
            Self::Factor { .. } => SexpType::Integer,
            Self::DataFrame { .. } => SexpType::List,
            Self::Matrix { .. } => SexpType::Vector,
            Self::Dots(_) => SexpType::Dots,
            Self::Promise { .. } => SexpType::Promise,
            Self::Closure { .. } => SexpType::Closure,
            Self::Function { .. }
            | Self::If { .. }
            | Self::For { .. }
            | Self::While { .. }
            | Self::Repeat { .. }
            | Self::Break
            | Self::Next
            | Self::Block(_)
            | Self::Subset { .. }
            | Self::Subset2D { .. }
            | Self::Extract { .. }
            | Self::Dollar { .. }
            | Self::Call { .. }
            | Self::InternalCall { .. }
            | Self::Assignment { .. }
            | Self::SuperAssignment { .. }
            | Self::Subassignment { .. }
            | Self::MatrixSubassignment { .. }
            | Self::ExtractAssignment { .. }
            | Self::DollarAssignment { .. }
            | Self::NamesAssignment { .. }
            | Self::Binary { .. }
            | Self::Unary { .. } => SexpType::Language,
            Self::Parenthesized(_) => SexpType::Language,
        }
    }
}

#[cfg(test)]
mod tests;
