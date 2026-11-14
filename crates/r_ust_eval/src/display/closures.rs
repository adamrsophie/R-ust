use crate::{body, language_print, EvalError, FunctionParameter, ListElement, Sexp};
use r_ust_sexp::{ClosureEnvironment, EnvironmentRole};

pub(super) fn format(
    parameters: &[FunctionParameter],
    expression: &Sexp,
    environment: &ClosureEnvironment,
) -> Result<String, EvalError> {
    let body = body::source(expression)?;
    let parameters = parameters
        .iter()
        .map(|parameter| {
            Ok(ListElement {
                name: Some(parameter.name.clone()),
                value: match &parameter.default {
                    Some(default) => body::source(default)?,
                    None => Sexp::MissingValue,
                },
            })
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    let mut output = language_print::format_closure(&parameters, &body)?;
    let suffix = match environment {
        ClosureEnvironment::Base => Some("<environment: namespace:base>".into()),
        ClosureEnvironment::Captured(frame) => match frame.role()? {
            EnvironmentRole::Global => None,
            EnvironmentRole::Base => Some("<environment: base>".into()),
            EnvironmentRole::BaseNamespace => Some("<environment: namespace:base>".into()),
            EnvironmentRole::Empty => Some("<environment: R_EmptyEnv>".into()),
            EnvironmentRole::Ordinary | EnvironmentRole::Autoloads => {
                Some(format!("<environment: {}>", frame.display_identity()?))
            }
        },
    };
    if let Some(suffix) = suffix {
        output.push('\n');
        output.push_str(&suffix);
    }
    Ok(output)
}
