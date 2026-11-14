use super::{call, canonical_language, optional, LanguageError, Sexp};

pub(super) fn canonical_assignment(value: &Sexp) -> Result<Sexp, LanguageError> {
    Ok(match value {
        Sexp::Assignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![Sexp::Symbol(target.clone()), canonical_language(value)?],
        ),
        Sexp::SuperAssignment { target, value } => call(
            "<<-",
            vec![Sexp::Symbol(target.clone()), canonical_language(value)?],
        ),
        Sexp::Subassignment {
            operator,
            target,
            index,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[",
                    vec![Sexp::Symbol(target.clone()), optional(index.as_deref())?],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::MatrixSubassignment {
            operator,
            target,
            row,
            col,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[",
                    vec![
                        Sexp::Symbol(target.clone()),
                        optional(row.as_deref())?,
                        optional(col.as_deref())?,
                    ],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::ExtractAssignment {
            operator,
            target,
            index,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[[",
                    vec![Sexp::Symbol(target.clone()), canonical_language(index)?],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::DollarAssignment {
            operator,
            target,
            name,
            value,
        } => call(
            operator.as_str(),
            vec![
                call("$", vec![Sexp::Symbol(target.clone()), name.to_sexp()]),
                canonical_language(value)?,
            ],
        ),
        Sexp::NamesAssignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![
                call("names", vec![Sexp::Symbol(target.clone())]),
                canonical_language(value)?,
            ],
        ),
        Sexp::MemberAssignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![canonical_language(target)?, canonical_language(value)?],
        ),
        Sexp::InternalCall { .. } => return Err(LanguageError::InternalCall),
        Sexp::Nil
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Call { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_)
        | Sexp::Block(_)
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. } => return Err(LanguageError::RuntimeValue),
    })
}
