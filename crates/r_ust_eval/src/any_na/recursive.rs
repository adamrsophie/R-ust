use super::*;

pub(super) fn recursive_flag(value: &Sexp) -> Result<bool, EvalError> {
    if matches!(
        typeof_value(value),
        "logical" | "integer" | "double" | "character"
    ) && value_len(value)? > 1
    {
        return Err(EvalError::Domain(format!(
            "'length = {}' in coercion to 'logical(1)'",
            value_len(value)?
        )));
    }
    let logical = match value {
        Sexp::Factor { codes, .. } => codes.first().copied().flatten().map(|code| code != 0),
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. } => {
            coerce_logical_values(value)?.first().copied().flatten()
        }
        Sexp::Matrix { data, .. } => return recursive_flag(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::PairListObject(_)
        | Sexp::ListObject(_)
        | Sexp::Char(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Vector(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
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
        | Sexp::Parenthesized(_) => None,
    };
    logical.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))
}
