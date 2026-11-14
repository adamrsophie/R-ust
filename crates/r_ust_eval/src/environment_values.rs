use crate::{value_len, EvalError};
use r_ust_sexp::{Environment, EnvironmentRole, Sexp};

pub(super) fn length(value: &Sexp) -> Result<Sexp, EvalError> {
    let length = match value {
        Sexp::Environment(frame) => frame.local_len()?,
        other => value_len(other),
    };
    let length = i32::try_from(length)
        .map_err(|_| EvalError::Domain("length exceeds supported integer range".into()))?;
    Ok(Sexp::Integer(vec![Some(length)]))
}

pub(super) fn format(frame: &Environment) -> Result<String, EvalError> {
    let identity = match frame.role()? {
        EnvironmentRole::Global => "R_GlobalEnv".into(),
        EnvironmentRole::Base => "base".into(),
        EnvironmentRole::BaseNamespace => "namespace:base".into(),
        EnvironmentRole::Empty => "R_EmptyEnv".into(),
        EnvironmentRole::Autoloads | EnvironmentRole::Ordinary => frame.display_identity()?,
    };
    let mut text = format!("<environment: {identity}>");
    if let Some(name) = frame.name_attribute()? {
        text.push_str(&format!("\nattr(,\"name\")\n[1] \"{name}\""));
    }
    Ok(text)
}

pub(super) fn coercion_error(target: &str) -> EvalError {
    EvalError::Type(format!(
        "cannot coerce type 'environment' to vector of type '{target}'"
    ))
}
