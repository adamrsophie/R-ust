mod deparse;

use super::{CallArgument, EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_dots_length(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        self.eval_arguments(arguments, false)?;
        if !arguments.is_empty() {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to '...length' which requires 0",
                arguments.len()
            )));
        }
        let Some(Sexp::Dots(values)) = self.raw_lookup("...")? else {
            return Err(EvalError::Domain(
                "incorrect context: the current call has no '...' to look in".into(),
            ));
        };
        let length = i32::try_from(values.len())
            .map_err(|_| EvalError::Domain("dots length overflow".into()))?;
        Ok(Sexp::Integer(vec![Some(length)]))
    }

    pub(super) fn lookup_dot(&mut self, index: usize) -> Result<Sexp, EvalError> {
        if index == 0 {
            return Err(EvalError::Domain(
                "indexing '...' with non-positive index 0".into(),
            ));
        }
        let Some(Sexp::Dots(values)) = self.raw_lookup("...")? else {
            return Err(EvalError::Domain(format!(
                "..{index} used in an incorrect context, no ... to look in"
            )));
        };
        let value = values
            .get(index - 1)
            .ok_or_else(|| {
                let suffix = if index == 1 { "" } else { "s" };
                EvalError::Domain(format!(
                    "the ... list contains fewer than {index} element{suffix}"
                ))
            })?
            .value
            .clone();
        if matches!(value, Sexp::MissingArg(_) | Sexp::MissingValue) {
            return Err(EvalError::MissingArgument(format!("..{index}")));
        }
        self.force_value(value)
    }
}

pub(super) fn list_character(value: &Sexp) -> Result<String, EvalError> {
    match value {
        Sexp::String(values) if values.len() == 1 => {
            Ok(values[0].clone().unwrap_or_else(|| "NA".into()))
        }
        Sexp::Logical(values) if values.len() == 1 => Ok(match values[0] {
            Some(true) => "TRUE",
            Some(false) => "FALSE",
            None => "NA",
        }
        .into()),
        Sexp::Integer(values) if values.len() == 1 => {
            Ok(values[0].map_or_else(|| "NA".into(), |value| value.to_string()))
        }
        Sexp::Real(values) if values.len() == 1 => Ok(super::format_number(values[0])),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
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
        | Sexp::Parenthesized(_) => deparse::deparse_value(value),
    }
}
