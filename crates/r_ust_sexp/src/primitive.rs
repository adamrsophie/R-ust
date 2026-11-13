use crate::SexpType;

/// Public primitives implemented as first-class values, distinct from internal kernels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveId {
    Sum,
    Length,
    Invisible,
    Return,
    Missing,
}

impl PrimitiveId {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sum" => Some(Self::Sum),
            "length" => Some(Self::Length),
            "invisible" => Some(Self::Invisible),
            "return" => Some(Self::Return),
            "missing" => Some(Self::Missing),
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
        }
    }

    pub const fn sexptype(self) -> SexpType {
        match self {
            Self::Sum | Self::Length | Self::Invisible => SexpType::Builtin,
            Self::Return | Self::Missing => SexpType::Special,
        }
    }
}
