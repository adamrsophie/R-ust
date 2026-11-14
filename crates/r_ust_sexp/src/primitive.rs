use crate::SexpType;

/// Public primitives implemented as first-class values, distinct from internal kernels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveId {
    Sum,
    Length,
    Invisible,
    Return,
    Missing,
    UseMethod,
    OnExit,
    Nargs,
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
    GlobalEnv,
    BaseEnv,
    EmptyEnv,
    IsEnvironment,
}

impl PrimitiveId {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sum" => Some(Self::Sum),
            "length" => Some(Self::Length),
            "invisible" => Some(Self::Invisible),
            "return" => Some(Self::Return),
            "missing" => Some(Self::Missing),
            "UseMethod" => Some(Self::UseMethod),
            "on.exit" => Some(Self::OnExit),
            "nargs" => Some(Self::Nargs),
            "quote" => Some(Self::Quote),
            ".Primitive" => Some(Self::Primitive),
            "globalenv" => Some(Self::GlobalEnv),
            "baseenv" => Some(Self::BaseEnv),
            "emptyenv" => Some(Self::EmptyEnv),
            "is.environment" => Some(Self::IsEnvironment),
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
            Self::UseMethod => "UseMethod",
            Self::OnExit => "on.exit",
            Self::Nargs => "nargs",
            Self::Quote => "quote",
            Self::Primitive => ".Primitive",
            Self::GlobalEnv => "globalenv",
            Self::BaseEnv => "baseenv",
            Self::EmptyEnv => "emptyenv",
            Self::IsEnvironment => "is.environment",
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
            | Self::Nargs
            | Self::GlobalEnv
            | Self::BaseEnv
            | Self::EmptyEnv
            | Self::IsEnvironment
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
            Self::Return | Self::Missing | Self::Quote | Self::UseMethod | Self::OnExit => {
                SexpType::Special
            }
        }
    }
}
