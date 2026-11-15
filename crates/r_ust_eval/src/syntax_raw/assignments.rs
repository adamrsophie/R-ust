use super::{named, optional, Sexp};

pub(super) fn call(expression: &Sexp) -> Option<Sexp> {
    let (operator, target, value) = match expression {
        Sexp::Assignment {
            operator,
            target,
            value,
        } => (operator.as_str(), Sexp::Symbol(target.clone()), value),
        Sexp::SuperAssignment { target, value } => ("<<-", Sexp::Symbol(target.clone()), value),
        Sexp::Subassignment {
            operator,
            target,
            index,
            value,
        } => (
            operator.as_str(),
            named(
                "[",
                vec![Sexp::Symbol(target.clone()), optional(index.as_deref())],
            ),
            value,
        ),
        Sexp::MatrixSubassignment {
            operator,
            target,
            row,
            col,
            value,
        } => (
            operator.as_str(),
            named(
                "[",
                vec![
                    Sexp::Symbol(target.clone()),
                    optional(row.as_deref()),
                    optional(col.as_deref()),
                ],
            ),
            value,
        ),
        Sexp::ExtractAssignment {
            operator,
            target,
            index,
            value,
        } => (
            operator.as_str(),
            named(
                "[[",
                vec![Sexp::Symbol(target.clone()), index.as_ref().clone()],
            ),
            value,
        ),
        Sexp::DollarAssignment {
            operator,
            target,
            name,
            value,
        } => (
            operator.as_str(),
            named("$", vec![Sexp::Symbol(target.clone()), name.to_sexp()]),
            value,
        ),
        Sexp::NamesAssignment {
            operator,
            target,
            value,
        } => (
            operator.as_str(),
            named("names", vec![Sexp::Symbol(target.clone())]),
            value,
        ),
        Sexp::MemberAssignment {
            operator,
            target,
            value,
        }
        | Sexp::ReplacementAssignment {
            operator,
            target,
            value,
        } => (operator.as_str(), target.as_ref().clone(), value),
        _ => return None,
    };
    Some(named(operator, vec![target, value.as_ref().clone()]))
}
