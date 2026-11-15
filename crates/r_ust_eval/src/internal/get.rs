use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::{EnvironmentRole, InternalOperation};

impl RuntimeState {
    pub(super) fn internal_get(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let name = binding_name(&arguments[0].value, &call)?;
        let environment = match &arguments[1].value {
            Sexp::Environment(environment) => environment.clone(),
            Sexp::Integer(_) | Sexp::Real(_) => {
                let Sexp::Environment(environment) =
                    self.dynamic_frame(InternalOperation::SysFrame, &arguments[1..2])?
                else {
                    return Err(EvalError::Control("invalid frame lookup result".into()));
                };
                environment
            }
            Sexp::Nil => {
                return Err(SourceFault::domain(
                    "use of NULL environment is defunct".into(),
                    &call,
                ))
            }
            _ => {
                return Err(SourceFault::domain(
                    "invalid 'envir' argument".into(),
                    &call,
                ))
            }
        };
        self.global
            .validate_import(&Sexp::Environment(environment.clone()))?;
        let requested_mode = arguments[2]
            .value
            .character_values()
            .and_then(|values| values.first())
            .ok_or_else(|| SourceFault::domain("invalid 'mode' argument".into(), &call))?
            .as_deref()
            .unwrap_or("NA");
        let mode = mode_name(requested_mode).ok_or_else(|| {
            SourceFault::domain(format!("invalid 'mode' argument '{requested_mode}'"), &call)
        })?;
        let inherits = super::as_logical(&arguments[3].value)
            .ok_or_else(|| SourceFault::domain("invalid 'inherits' argument".into(), &call))?;
        let mut current = Some(environment);
        while let Some(frame) = current {
            if frame.role()? == EnvironmentRole::Empty {
                break;
            }
            if let Some(value) = frame.get_local(name)? {
                if matches!(value, Sexp::MissingArg(_) | Sexp::MissingValue) {
                    if mode == "any" || mode == "symbol" {
                        return Err(SourceFault::missing(
                            name.into(),
                            call,
                            crate::conditions::MissingSite::FunctionBinding,
                        ));
                    }
                } else {
                    let value = self.force_value(value)?;
                    let kind = crate::metadata::typeof_value(&value);
                    if mode == "any" || mode_name(kind) == Some(mode) {
                        return Ok(value);
                    }
                }
            }
            current = if inherits { frame.parent()? } else { None };
        }
        let message = if mode == "any" {
            format!("object '{name}' not found")
        } else {
            format!("object '{name}' of mode '{requested_mode}' was not found")
        };
        Err(SourceFault::domain(message, &call))
    }
}

fn binding_name<'a>(value: &'a Sexp, call: &Sexp) -> Result<&'a str, EvalError> {
    match value {
        Sexp::Symbol(name) => Ok(name),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            if values.len() > 1 && values[0].as_deref().is_some_and(|name| !name.is_empty()) {
                return Err(SourceFault::domain(
                    "first argument has length > 1".into(),
                    call,
                ));
            }
            match values.as_slice() {
                [Some(name)] if !name.is_empty() => Ok(name),
                _ => Err(SourceFault::domain("invalid first argument".into(), call)),
            }
        }
        _ => Err(SourceFault::domain("invalid first argument".into(), call)),
    }
}

fn mode_name(mode: &str) -> Option<&str> {
    Some(match mode {
        "numeric" | "double" | "integer" => "numeric",
        "function" | "closure" | "builtin" | "special" => "function",
        "name" | "symbol" => "symbol",
        "any" | "NULL" | "pairlist" | "language" | "environment" | "promise" | "logical"
        | "character" | "char" | "complex" | "raw" | "list" | "expression" | "..."
        | "externalptr" | "weakref" | "S4" => mode,
        _ => return None,
    })
}
