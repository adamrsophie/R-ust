use super::{call, Canonicalizer, LanguageError, Sexp};

pub(super) fn canonical_assignment(
    value: &Sexp,
    canonical: &Canonicalizer,
) -> Result<Sexp, LanguageError> {
    Ok(match value {
        Sexp::Assignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![Sexp::Symbol(target.clone()), canonical.language(value)?],
        ),
        Sexp::SuperAssignment { target, value } => call(
            "<<-",
            vec![Sexp::Symbol(target.clone()), canonical.language(value)?],
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
                    vec![
                        Sexp::Symbol(target.clone()),
                        canonical.optional(index.as_deref())?,
                    ],
                ),
                canonical.language(value)?,
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
                        canonical.optional(row.as_deref())?,
                        canonical.optional(col.as_deref())?,
                    ],
                ),
                canonical.language(value)?,
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
                    vec![Sexp::Symbol(target.clone()), canonical.language(index)?],
                ),
                canonical.language(value)?,
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
                canonical.language(value)?,
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
                canonical.language(value)?,
            ],
        ),
        Sexp::ReplacementAssignment {
            operator,
            target,
            value,
        }
        | Sexp::MemberAssignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![canonical.language(target)?, canonical.language(value)?],
        ),
        Sexp::InternalCall { .. } => return Err(LanguageError::InternalCall),
        Sexp::Nil
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
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
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::Vector(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. } => return Err(LanguageError::RuntimeValue),
    })
}
