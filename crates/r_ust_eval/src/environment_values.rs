use crate::{value_len, EvalError, EvaluatedArgument, RuntimeState};
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

impl RuntimeState {
    pub(super) fn environment_root(
        &mut self,
        name: &str,
        role: EnvironmentRole,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        arity(name, arguments.len(), 0)?;
        let frame = self.current.session_root(role)?;
        self.visible = true;
        Ok(Sexp::Environment(frame))
    }

    pub(super) fn is_environment(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        arity("is.environment", arguments.len(), 1)?;
        if let Some(name) = arguments[0].name.as_deref().filter(|name| *name != "x") {
            return Err(EvalError::Arity(format!(
                "supplied argument name '{name}' does not match 'x'"
            )));
        }
        self.visible = true;
        Ok(Sexp::Logical(vec![Some(matches!(
            arguments[0].value,
            Sexp::Environment(_)
        ))]))
    }
}

fn arity(name: &str, count: usize, required: usize) -> Result<(), EvalError> {
    if count != required {
        let suffix = if count == 1 { "" } else { "s" };
        return Err(EvalError::Arity(format!(
            "{count} argument{suffix} passed to '{name}' which requires {required}"
        )));
    }
    Ok(())
}

pub(super) fn parent(value: &Sexp) -> Result<Sexp, EvalError> {
    let Sexp::Environment(frame) = value else {
        return Err(EvalError::Type("argument is not an environment".into()));
    };
    frame
        .parent()?
        .map(Sexp::Environment)
        .ok_or_else(|| EvalError::Domain("the empty environment has no parent".into()))
}

pub(super) fn name(value: &Sexp) -> Result<Sexp, EvalError> {
    let name = if let Sexp::Environment(frame) = value {
        frame.name()?
    } else {
        ""
    };
    Ok(Sexp::String(vec![Some(name.into())]))
}
