use crate::{Sexp, SexpType};

impl Sexp {
    pub const fn sexptype(&self) -> SexpType {
        match self {
            Self::Nil => SexpType::Nil,
            Self::Bytecode(_) => SexpType::Bytecode,
            Self::Managed(value) => value.sexptype(),
            Self::AttributedAtomic { values, .. } => values.sexptype(),
            Self::AttributedAggregate { values, .. } => values.sexptype(),
            Self::Environment(_) => SexpType::Environment,
            Self::Primitive(id) => id.sexptype(),
            Self::MissingArg(_) => SexpType::Promise,
            Self::Symbol(_) | Self::MissingValue => SexpType::Symbol,
            Self::Logical(_) => SexpType::Logical,
            Self::Integer(_) => SexpType::Integer,
            Self::Real(_) => SexpType::Real,
            Self::Raw(_) => SexpType::Raw,
            Self::String(_) | Self::AttributedCharacter { .. } => SexpType::String,
            Self::Char(_) => SexpType::Char,
            Self::Expression { .. } => SexpType::Expression,
            Self::Vector(_) => SexpType::Vector,
            Self::NamedList { .. }
            | Self::ClassedList { .. }
            | Self::List(_)
            | Self::ListObject(_) => SexpType::Vector,
            Self::PairList(_) | Self::PairListObject(_) => SexpType::List,
            Self::Factor { .. } => SexpType::Integer,
            Self::DataFrame { .. } => SexpType::Vector,
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
            | Self::ExtractArguments { .. }
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
            | Self::MemberAssignment { .. }
            | Self::ReplacementAssignment { .. }
            | Self::NamesAssignment { .. }
            | Self::Binary { .. }
            | Self::Unary { .. } => SexpType::Language,
            Self::Parenthesized(_) => SexpType::Language,
        }
    }
}
