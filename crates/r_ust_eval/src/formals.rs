use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{canonical_language, LanguageError, ListElement};

impl RuntimeState {
    pub(super) fn formals(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::Closure { parameters, .. } => {
                let elements = parameters
                    .iter()
                    .map(|parameter| {
                        let value = match &parameter.default {
                            Some(default) => {
                                canonical_language(default).map_err(|error| match error {
                                    LanguageError::InternalCall => {
                                        EvalError::NotImplemented("formals internal default")
                                    }
                                    LanguageError::RuntimeValue => {
                                        EvalError::NotImplemented("formals runtime default")
                                    }
                                })?
                            }
                            None => Sexp::MissingValue,
                        };
                        Ok(ListElement {
                            name: Some(parameter.name.clone()),
                            value,
                        })
                    })
                    .collect::<Result<Vec<_>, EvalError>>()?;
                Ok(Sexp::pairlist(elements))
            }
            Sexp::Primitive(_) => Ok(Sexp::Nil),
            Sexp::Nil
            | Sexp::Environment(_)
            | Sexp::MissingArg(_)
            | Sexp::MissingValue
            | Sexp::Symbol(_)
            | Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::Vector(_)
            | Sexp::List(_)
            | Sexp::PairList(_)
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Promise { .. }
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
            | Sexp::MemberAssignment { .. }
            | Sexp::NamesAssignment { .. }
            | Sexp::Binary { .. }
            | Sexp::Unary { .. }
            | Sexp::Parenthesized(_) => {
                self.record_parse_warnings(&["argument is not a function".into()])?;
                Ok(Sexp::Nil)
            }
        }
    }
}
