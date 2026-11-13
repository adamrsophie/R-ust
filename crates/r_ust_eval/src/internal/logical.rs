use r_ust_sexp::Sexp;

pub(super) fn as_logical(value: &Sexp) -> Option<bool> {
    match value {
        Sexp::String(values) => {
            values
                .first()
                .and_then(|value| value.as_deref())
                .and_then(|value| match value {
                    "T" | "TRUE" | "True" | "true" => Some(true),
                    "F" | "FALSE" | "False" | "false" => Some(false),
                    _ => None,
                })
        }
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
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
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
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => None,
    }
}
