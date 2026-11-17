use super::*;

pub(crate) fn scalar_missing(value: &Sexp) -> Result<bool, EvalError> {
    Ok(match value {
        Sexp::Managed(value) => scalar_missing(value.borrow()?.value())?,
        Sexp::AttributedAtomic { values, .. } => scalar_missing(&values.clone().into_value())?,
        Sexp::AttributedAggregate { .. } => false,
        Sexp::Logical(values) => values.as_slice() == [None],
        Sexp::Integer(values) => values.as_slice() == [None],
        Sexp::Real(values) => values.len() == 1 && values[0].is_nan(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.as_slice() == [None]
        }
        Sexp::Factor { codes, .. } => codes.as_slice() == [None],
        Sexp::Matrix { data, .. } => scalar_missing(data)?,
        Sexp::Raw(_)
        | Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::Expression { .. }
        | Sexp::PairList(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Char(_)
        | Sexp::NamedList { .. }
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Vector(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Bytecode(_)
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
        | Sexp::Parenthesized(_) => false,
    })
}
