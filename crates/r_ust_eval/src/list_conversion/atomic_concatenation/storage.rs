use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn concatenate(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let mut values = Vec::new();
    let mut priority = 0;
    for arg in args {
        let value = crate::list_attributes::engine::payload(&arg.value)?;
        priority = priority.max(match value {
            Sexp::Nil => 0,
            Sexp::Logical(_) => 1,
            Sexp::Integer(_) => 2,
            Sexp::Real(_) => 3,
            Sexp::String(_) => 4,
            _ => {
                return Err(EvalError::NotImplemented(
                    "default concatenation of non-atomic storage",
                ));
            }
        });
        values.push(value);
    }
    match priority {
        0 => Ok(Sexp::Nil),
        1 => {
            let mut output = Vec::new();
            for value in values {
                output.extend(crate::logical_values(&value)?);
            }
            Ok(Sexp::Logical(output))
        }
        2 => {
            let mut output = Vec::new();
            for value in values {
                output.extend(
                    crate::numeric_values(&value)?
                        .into_iter()
                        .map(crate::real_to_integer),
                );
            }
            Ok(Sexp::Integer(output))
        }
        3 => {
            let mut output = Vec::new();
            for value in values {
                output.extend(crate::numeric_values(&value)?);
            }
            Ok(Sexp::Real(output))
        }
        _ => {
            let mut output = Vec::new();
            for value in values {
                output.extend(crate::string_values(&value)?);
            }
            Ok(Sexp::String(output))
        }
    }
}
