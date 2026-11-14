use super::*;

pub(super) fn value_len(value: &Sexp) -> usize {
    match value {
        Sexp::Nil => 0,
        Sexp::MissingArg(_) => 1,
        Sexp::Primitive(_) | Sexp::Environment(_) => 1,
        Sexp::Symbol(_) | Sexp::MissingValue => 1,
        Sexp::PairList(values) => values.as_slice().len(),
        Sexp::Call { arguments, .. } => 1 + arguments.len(),
        Sexp::Logical(values) => values.len(),
        Sexp::Integer(values) => values.len(),
        Sexp::Real(values) => values.len(),
        Sexp::String(values) => values.len(),
        Sexp::Vector(values) => values.len(),
        Sexp::List(values) => values.len(),
        Sexp::Factor { codes, .. } => codes.len(),
        Sexp::DataFrame { columns, .. } => columns.len(),
        Sexp::Matrix { data, .. } => value_len(data),
        Sexp::Dots(values) => values.len(),
        Sexp::Promise { .. } => 1,
        Sexp::Closure { .. }
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
        | Sexp::Parenthesized(_) => 1,
    }
}
