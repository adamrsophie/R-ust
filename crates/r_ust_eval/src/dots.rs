use super::{CallArgument, EvalError, Sexp};
use r_ust_sexp::Environment;

pub(super) fn expand_dots_call_arguments(
    arguments: &[CallArgument],
    environment: &Environment,
) -> Result<Vec<CallArgument>, EvalError> {
    let mut expanded = Vec::new();
    for argument in arguments {
        if argument.name.is_none() && matches!(&argument.value, Sexp::Symbol(name) if name == "...")
        {
            let value = environment
                .get("...")?
                .ok_or_else(|| EvalError::UndefinedSymbol("...".into()))?;
            let Sexp::Dots(values) = value else {
                return Err(EvalError::Type("... is not a dots object".into()));
            };
            expanded.extend(values.into_iter().map(|element| CallArgument {
                name: element.name,
                value: element.value,
            }));
        } else {
            expanded.push(argument.clone());
        }
    }
    Ok(expanded)
}
