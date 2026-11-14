use r_ust_sexp::Sexp;

pub(crate) fn as_logical(value: &Sexp) -> Option<bool> {
    match value {
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values
            .first()
            .and_then(|value| value.as_deref())
            .and_then(text_logical),
        Sexp::Char(value) => value.as_deref().and_then(text_logical),
        Sexp::Logical(values) => values.first().copied().flatten(),
        Sexp::Integer(values) => values.first().copied().flatten().map(|value| value != 0),
        Sexp::Real(values) => values
            .first()
            .filter(|value| !value.is_nan())
            .map(|value| *value != 0.0),
        Sexp::Factor { codes, .. } => codes.first().copied().flatten().map(|value| value != 0),
        Sexp::Matrix { data, .. } => as_logical(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Vector(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
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
        | Sexp::Parenthesized(_) => None,
    }
}

fn text_logical(value: &str) -> Option<bool> {
    match value {
        "T" | "TRUE" | "True" | "true" => Some(true),
        "F" | "FALSE" | "False" | "false" => Some(false),
        _ => None,
    }
}
