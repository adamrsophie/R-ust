use crate::{body, language_print, EvalError, FunctionParameter, ListElement, Sexp};
use r_ust_sexp::{ClosureEnvironment, EnvironmentRole};

pub(super) fn format(value: &Sexp, quote: bool, prefix: &str) -> Result<Option<String>, EvalError> {
    let Sexp::Closure {
        parameters,
        body,
        environment,
        attributes,
    } = value
    else {
        return Err(EvalError::Type("expected closure".into()));
    };
    let mut output = match attributes
        .iter()
        .find(|attribute| attribute.name.as_deref() == Some("srcref"))
    {
        None => Some(source(parameters, body)?),
        Some(attribute) => match &attribute.value {
            Sexp::Nil => Some(source(parameters, body)?),
            Sexp::Integer(values) if values.is_empty() => None,
            Sexp::Integer(values) => Some(
                values
                    .iter()
                    .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => {
                return Err(EvalError::NotImplemented(
                    "closure source-reference print dispatch",
                ))
            }
        },
    };
    if let Sexp::Bytecode(code) = body.as_ref() {
        super::lines::append(
            &mut output,
            format!("<bytecode: {}>", code.display_address()?),
        );
    }
    if let Some(suffix) = environment_suffix(environment)? {
        super::lines::append(&mut output, suffix);
    }
    for attribute in attributes {
        let name = attribute
            .name
            .as_deref()
            .ok_or_else(|| EvalError::Type("unnamed closure attribute".into()))?;
        if matches!(name, "comment" | "srcref" | "wholeSrcref" | "srcfile") {
            continue;
        }
        let label = format!(
            "{prefix}attr(,\"{}\")",
            crate::encode_string::encode(name, None)
        );
        super::lines::append(&mut output, label.clone());
        if let Some(value) = super::character::render(&attribute.value, quote, &label)? {
            super::lines::append(&mut output, value);
        }
    }
    Ok(output)
}

fn source(parameters: &[FunctionParameter], expression: &Sexp) -> Result<String, EvalError> {
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
    language_print::format_closure(&parameters, &body).map_err(EvalError::from)
}

fn environment_suffix(environment: &ClosureEnvironment) -> Result<Option<String>, EvalError> {
    Ok(match environment {
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
    })
}
