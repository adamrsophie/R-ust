mod atoms;
mod deparse;
mod deparse_atoms;
mod deparse_attributes;
mod deparse_closure;
mod deparse_special;
mod deparse_values;
mod managed;
mod metadata;
mod profile;
pub(crate) use profile::format_coercion;
pub(super) use profile::{format_deparse, Profile, WARN};
mod calls;
#[cfg(test)]
mod coercion_tests;
mod control;
mod infix;
mod layout;
mod names;
pub(super) use names::{check_text, valid_name};
mod expressions;
mod operators;
use crate::primitive_calls::names as primitives;
#[cfg(test)]
mod tests;

use crate::Sexp;
use layout::Writer;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum LanguagePrintError {
    Runtime(Box<crate::EvalError>),
    UnsupportedNode,
    UnsupportedText,
    NonScalar,
    MalformedCall,
    InvalidText,
}

impl From<LanguagePrintError> for crate::EvalError {
    fn from(error: LanguagePrintError) -> Self {
        match error {
            LanguagePrintError::Runtime(error) => *error,
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

pub(super) fn format_argument_syntax(value: &Sexp) -> Result<String, LanguagePrintError> {
    let mut writer = Writer {
        strip_integer_suffix: true,
        ..Writer::default()
    };
    writer.write(value)?;
    Ok(writer.output)
}

pub(super) fn format_closure(
    parameters: &[crate::ListElement],
    body: &Sexp,
) -> Result<String, LanguagePrintError> {
    let mut writer = Writer::default();
    writer.emit("function (");
    writer.formals(parameters)?;
    writer.emit(") ");
    writer.newline();
    writer.write(body)?;
    Ok(writer.output)
}

impl Writer<'_> {
    fn write(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        if !self.active() {
            return Ok(());
        }
        if matches!(
            value,
            Sexp::Managed(_) | Sexp::ListObject(_) | Sexp::PairListObject(_)
        ) {
            return self.write_graph(value);
        }
        let argument = self.fnarg;
        let left = self.left;
        self.fnarg = false;
        self.left = 0;
        let result = if self.profile.is_some() {
            self.deparse_node(value, (argument, left))
        } else {
            match value {
                Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. } => {
                    self.deparse_typed(value)
                }
                Sexp::Managed(_) => self.write_graph(value),
                Sexp::Expression {
                    elements,
                    names_present,
                } => self.expression(elements, *names_present),
                Sexp::Call {
                    function,
                    arguments,
                } => self.call(function, arguments, (argument, left)),
                Sexp::Raw(_) => self.deparse_atomic(value, &[]),
                Sexp::Nil
                | Sexp::MissingValue
                | Sexp::Symbol(_)
                | Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_) => self.atom(value),
                Sexp::PairList(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::MissingArg(_)
                | Sexp::Char(_)
                | Sexp::ListObject(_)
                | Sexp::PairListObject(_)
                | Sexp::Environment(_)
                | Sexp::Bytecode(_)
                | Sexp::Primitive(_)
                | Sexp::Factor { .. }
                | Sexp::Vector(_)
                | Sexp::NamedList { .. }
                | Sexp::ClassedList { .. }
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
                | Sexp::ExtractArguments { .. }
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
                | Sexp::ReplacementAssignment { .. }
                | Sexp::NamesAssignment { .. }
                | Sexp::Binary { .. }
                | Sexp::Unary { .. }
                | Sexp::Parenthesized(_) => Err(LanguagePrintError::UnsupportedNode),
            }
        };
        self.left = left;
        result
    }
}

#[cfg(test)]
mod native_feedback_tests;
