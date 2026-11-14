mod bitwise;
mod conditions;
mod constants;
mod definitions;
mod encode_string;
mod environments;
mod evaluation;
mod formals;
mod list_values;
mod printing;
mod registry;
mod rowsum;
mod run_length;
mod sequence;
mod session;

use super::{EvalError, Parser, Sexp};

pub(super) use registry::iter;
pub(super) use session::initialize;

fn lower(value: &mut Sexp) -> Result<(), EvalError> {
    match value {
        Sexp::Call {
            function,
            arguments,
        } => {
            if let Sexp::Symbol(name) = function.as_ref() {
                if let Some(operation) = rowsum::resolve(name) {
                    for argument in arguments.iter_mut() {
                        lower(&mut argument.value)?;
                    }
                    *value = Sexp::InternalCall {
                        operation,
                        arguments: std::mem::take(arguments),
                    };
                    return Ok(());
                }
            }
            if matches!(function.as_ref(), Sexp::Symbol(name) if name == ".Internal") {
                let [argument] = arguments.as_mut_slice() else {
                    return Err(EvalError::Arity("invalid base .Internal call".into()));
                };
                let Sexp::Call {
                    function,
                    arguments,
                } = &mut argument.value
                else {
                    return Err(EvalError::Type("invalid base .Internal argument".into()));
                };
                let Sexp::Symbol(name) = function.as_ref() else {
                    return Err(EvalError::Type("invalid base internal identity".into()));
                };
                super::internal::resolve(name)
                    .ok_or(EvalError::NotImplemented("unresolved R internal"))?;
                for argument in arguments.iter_mut() {
                    lower(&mut argument.value)?;
                }
            } else {
                lower(function)?;
                for argument in arguments {
                    lower(&mut argument.value)?;
                }
            }
        }
        Sexp::Function { parameters, body } => {
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    lower(default)?;
                }
            }
            lower(body)?;
        }
        Sexp::Dollar { value, .. } => lower(value)?,
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            lower(condition)?;
            lower(then_branch)?;
            if let Some(branch) = else_branch {
                lower(branch)?;
            }
        }
        Sexp::Block(values) => {
            for value in values {
                lower(value)?;
            }
        }
        Sexp::ReplacementAssignment { target, value, .. } => {
            lower(target)?;
            lower(value)?;
        }
        Sexp::ExtractArguments { value, arguments } => {
            lower(value)?;
            for argument in arguments {
                lower(&mut argument.value)?;
            }
        }
        Sexp::Subset { value, index } => {
            lower(value)?;
            if let Some(index) = index {
                lower(index)?;
            }
        }
        Sexp::Extract { value, index } => {
            lower(value)?;
            lower(index)?;
        }
        Sexp::Assignment { value, .. } | Sexp::Unary { value, .. } | Sexp::Parenthesized(value) => {
            lower(value)?
        }
        Sexp::Binary { left, right, .. } => {
            lower(left)?;
            lower(right)?;
        }
        Sexp::Nil | Sexp::Symbol(_) | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::String(_) => {}
        Sexp::MissingArg(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Real(_)
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
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Subset2D { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::NamesAssignment { .. } => {
            return Err(EvalError::NotImplemented("base bootstrap expression"))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_internal_rejects_unresolved_identities() {
        let mut expression = Parser
            .parse_all(".Internal(unknown_internal())")
            .unwrap()
            .expressions
            .remove(0);
        assert_eq!(
            lower(&mut expression),
            Err(EvalError::NotImplemented("unresolved R internal"))
        );
    }
}
