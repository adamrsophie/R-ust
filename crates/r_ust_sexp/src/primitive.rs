use crate::SexpType;

/// Public primitives implemented as first-class values, distinct from internal kernels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveId {
    Sum,
    Length,
    Invisible,
    Return,
    Missing,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    IntegerDivide,
    Colon,
    Quote,
    Primitive,
}

impl PrimitiveId {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sum" => Some(Self::Sum),
            "length" => Some(Self::Length),
            "invisible" => Some(Self::Invisible),
            "return" => Some(Self::Return),
            "missing" => Some(Self::Missing),
            "quote" => Some(Self::Quote),
            ".Primitive" => Some(Self::Primitive),
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            "^" => Some(Self::Power),
            "%%" => Some(Self::Modulo),
            "%/%" => Some(Self::IntegerDivide),
            ":" => Some(Self::Colon),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Sum => "sum",
            Self::Length => "length",
            Self::Invisible => "invisible",
            Self::Return => "return",
            Self::Missing => "missing",
            Self::Quote => "quote",
            Self::Primitive => ".Primitive",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Power => "^",
            Self::Modulo => "%%",
            Self::IntegerDivide => "%/%",
            Self::Colon => ":",
        }
    }

    pub const fn sexptype(self) -> SexpType {
        match self {
            Self::Sum
            | Self::Primitive
            | Self::Length
            | Self::Invisible
            | Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Power
            | Self::Modulo
            | Self::IntegerDivide
            | Self::Colon => SexpType::Builtin,
            Self::Return | Self::Missing | Self::Quote => SexpType::Special,
        }
    }
}
