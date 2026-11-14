use super::{as_logical, character::character_values};
use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn stop(args: &[EvaluatedArgument], call: &Sexp) -> Result<Sexp, EvalError> {
    let call_requested = as_logical(&args[0].value).unwrap_or(true);
    let message = match &args[1].value {
        Sexp::Nil => String::new(),
        value @ (Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::Vector(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
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
        | Sexp::Call { .. }
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
        | Sexp::Parenthesized(_)) => character_values(value)?
            .first()
            .cloned()
            .ok_or_else(|| crate::SourceFault::domain(" [invalid string in stop(.)]".into(), call))?
            .unwrap_or_else(|| "NA".into()),
    };
    Err(crate::SourceFault::stop(message, call_requested, call))
}
