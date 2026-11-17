use crate::{SexpType, SyntaxPrimitive};

/// Implemented first-class primitive identities.
/// Public lookup is restricted by `from_name`; InternalEval is context-exposed only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveId {
    Syntax(SyntaxPrimitive),
    InternalEval,
    Call,
    IsPairList,
    Sum,
    List,
    AsCharacter,
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
    Expression,
    IsExpression,
    Substitute,
    Primitive,
    Internal,
    GlobalEnv,
    BaseEnv,
    EmptyEnv,
    IsEnvironment,
    NamespaceExport,
    NamespaceInternal,
    AsCall,
    AsEnvironment,
    IsSymbol,
    Unclass,
    IsInteger,
    AnyNa,
    Abs,
}

impl PrimitiveId {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "is.integer" => Some(Self::IsInteger),
            "anyNA" => Some(Self::AnyNa),
            "abs" => Some(Self::Abs),
            "unclass" => Some(Self::Unclass),
            "call" => Some(Self::Call),
            "is.pairlist" => Some(Self::IsPairList),
            "sum" => Some(Self::Sum),
            "list" => Some(Self::List),
            "as.character" => Some(Self::AsCharacter),
            "length" => Some(Self::Length),
            "invisible" => Some(Self::Invisible),
            "return" => Some(Self::Return),
            "missing" => Some(Self::Missing),
            "UseMethod" => Some(Self::UseMethod),
            "on.exit" => Some(Self::OnExit),
            "nargs" => Some(Self::Nargs),
            "quote" => Some(Self::Quote),
            "expression" => Some(Self::Expression),
            "is.expression" => Some(Self::IsExpression),
            "substitute" => Some(Self::Substitute),
            ".Primitive" => Some(Self::Primitive),
            ".Internal" => Some(Self::Internal),
            "globalenv" => Some(Self::GlobalEnv),
            "baseenv" => Some(Self::BaseEnv),
            "emptyenv" => Some(Self::EmptyEnv),
            "is.environment" => Some(Self::IsEnvironment),
            "::" => Some(Self::NamespaceExport),
            ":::" => Some(Self::NamespaceInternal),
            "as.call" => Some(Self::AsCall),
            "as.environment" => Some(Self::AsEnvironment),
            "is.symbol" | "is.name" => Some(Self::IsSymbol),
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            "^" => Some(Self::Power),
            "%%" => Some(Self::Modulo),
            "%/%" => Some(Self::IntegerDivide),
            ":" => Some(Self::Colon),
            _ => SyntaxPrimitive::from_name(name).map(Self::Syntax),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Syntax(id) => id.name(),
            Self::InternalEval => "eval",
            Self::Call => "call",
            Self::Unclass => "unclass",
            Self::IsInteger => "is.integer",
            Self::AnyNa => "anyNA",
            Self::Abs => "abs",
            Self::IsPairList => "is.pairlist",
            Self::Sum => "sum",
            Self::List => "list",
            Self::AsCharacter => "as.character",
            Self::Length => "length",
            Self::Invisible => "invisible",
            Self::Return => "return",
            Self::Missing => "missing",
            Self::UseMethod => "UseMethod",
            Self::OnExit => "on.exit",
            Self::Nargs => "nargs",
            Self::Quote => "quote",
            Self::Expression => "expression",
            Self::IsExpression => "is.expression",
            Self::Substitute => "substitute",
            Self::Primitive => ".Primitive",
            Self::Internal => ".Internal",
            Self::GlobalEnv => "globalenv",
            Self::BaseEnv => "baseenv",
            Self::EmptyEnv => "emptyenv",
            Self::IsEnvironment => "is.environment",
            Self::NamespaceExport => "::",
            Self::NamespaceInternal => ":::",
            Self::AsCall => "as.call",
            Self::AsEnvironment => "as.environment",
            Self::IsSymbol => "is.symbol",
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
            Self::Syntax(id) => id.sexptype(),
            Self::InternalEval
            | Self::IsPairList
            | Self::Sum
            | Self::List
            | Self::AsCharacter
            | Self::IsExpression
            | Self::Nargs
            | Self::GlobalEnv
            | Self::BaseEnv
            | Self::EmptyEnv
            | Self::IsEnvironment
            | Self::AsCall
            | Self::AsEnvironment
            | Self::IsSymbol
            | Self::IsInteger
            | Self::AnyNa
            | Self::Abs
            | Self::Unclass
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
            Self::Call
            | Self::Internal
            | Self::Return
            | Self::NamespaceExport
            | Self::NamespaceInternal
            | Self::Missing
            | Self::Quote
            | Self::Expression
            | Self::Substitute
            | Self::UseMethod
            | Self::OnExit => SexpType::Special,
        }
    }
}
