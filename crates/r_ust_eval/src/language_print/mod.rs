mod atoms;
mod calls;
mod control;
mod infix;
mod layout;
mod names;
pub(super) use names::{check_text, valid_name};
mod operators;
use crate::primitive_calls::names as primitives;
#[cfg(test)]
mod tests;

use crate::Sexp;
use layout::Writer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LanguagePrintError {
    UnsupportedNode,
    UnsupportedText,
    NonScalar,
    MalformedCall,
    InvalidText,
}

impl From<LanguagePrintError> for crate::EvalError {
    fn from(error: LanguagePrintError) -> Self {
        match error {
            LanguagePrintError::UnsupportedNode => Self::NotImplemented("language printer node"),
            LanguagePrintError::UnsupportedText => {
                Self::NotImplemented("language printer non-ASCII text profile")
            }
            LanguagePrintError::NonScalar => {
                Self::NotImplemented("language printer nonscalar leaf")
            }
            LanguagePrintError::MalformedCall => {
                Self::Type("malformed canonical language call".into())
            }
            LanguagePrintError::InvalidText => Self::Type("NUL in canonical language text".into()),
        }
    }
}

pub(super) fn format_language(value: &Sexp) -> Result<String, LanguagePrintError> {
    let mut writer = Writer::default();
    writer.write(value)?;
    Ok(writer.output)
}

impl Writer {
    fn write(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        let argument = self.fnarg;
        let left = self.left;
        self.fnarg = false;
        self.left = 0;
        let result = match value {
            Sexp::Call {
                function,
                arguments,
            } => self.call(function, arguments, (argument, left)),
            Sexp::Nil
            | Sexp::MissingValue
            | Sexp::Symbol(_)
            | Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_) => self.atom(value),
            Sexp::PairList(_)
            | Sexp::MissingArg(_)
            | Sexp::Environment(_)
            | Sexp::Primitive(_)
            | Sexp::Factor { .. }
            | Sexp::Vector(_)
            | Sexp::List(_)
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Promise { .. }
            | Sexp::Closure { .. }
            | Sexp::Function { .. }
            | Sexp::If { .. }
            | Sexp::For { .. }
            | Sexp::While { .. }
            | Sexp::Repeat { .. }
            | Sexp::Break
            | Sexp::Next
            | Sexp::Block(_)
            | Sexp::Subset { .. }
            | Sexp::Subset2D { .. }
            | Sexp::Extract { .. }
            | Sexp::Dollar { .. }
            | Sexp::InternalCall { .. }
            | Sexp::Assignment { .. }
            | Sexp::SuperAssignment { .. }
            | Sexp::Subassignment { .. }
            | Sexp::MatrixSubassignment { .. }
            | Sexp::ExtractAssignment { .. }
            | Sexp::DollarAssignment { .. }
            | Sexp::MemberAssignment { .. }
            | Sexp::NamesAssignment { .. }
            | Sexp::Binary { .. }
            | Sexp::Unary { .. }
            | Sexp::Parenthesized(_) => Err(LanguagePrintError::UnsupportedNode),
        };
        self.left = left;
        result
    }
}
