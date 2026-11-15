mod indexing;
mod names;
pub(super) use indexing::{extract, select, subset};
pub(super) use names::{assign_names, names};

use crate::{CallArgument, EvalError, EvaluatedArgument, ListElement, Sexp};
use r_ust_sexp::{canonical_actual_call, LanguageError};

pub(super) fn capture(arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
    let names_present = arguments.iter().any(|argument| argument.name.is_some());
    let elements = arguments
        .iter()
        .map(|argument| {
            let value = canonical_actual_call(&argument.value).map_err(|error| match error {
                LanguageError::InternalCall => {
                    EvalError::NotImplemented("expression capture of internal dispatch")
                }
                LanguageError::RuntimeValue => {
                    EvalError::NotImplemented("expression capture of runtime value")
                }
            })?;
            Ok(ListElement {
                name: names_present.then(|| argument.name.clone().unwrap_or_default()),
                value,
            })
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    Ok(Sexp::Expression {
        elements,
        names_present,
    })
}

pub(super) fn is_expression(
    arguments: &[EvaluatedArgument],
    call: &Sexp,
) -> Result<Sexp, EvalError> {
    let [argument] = arguments else {
        return Err(crate::SourceFault::arity(
            format!(
                "{} arguments passed to 'is.expression' which requires 1",
                arguments.len()
            ),
            call,
        ));
    };
    if let Some(name) = argument.name.as_deref().filter(|name| *name != "x") {
        return Err(crate::SourceFault::arity(
            format!("supplied argument name '{name}' does not match 'x'"),
            call,
        ));
    }
    Ok(Sexp::Logical(vec![Some(matches!(
        argument.value,
        Sexp::Expression { .. }
    ))]))
}
