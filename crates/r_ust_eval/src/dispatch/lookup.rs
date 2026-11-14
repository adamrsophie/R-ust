use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{Environment, EnvironmentRole};

pub(super) struct MethodEnvironments {
    pub(super) call: Environment,
    pub(super) definition: Environment,
}

pub(super) fn topenv(start: &Environment) -> Result<Environment, EvalError> {
    let mut current = Some(start.clone());
    while let Some(frame) = current {
        let namespace = match frame.get_local(".__NAMESPACE__.")? {
            Some(Sexp::Environment(info)) => info.get_local("spec")?.is_some_and(|spec| {
                spec.character_values()
                    .is_some_and(|values| !values.is_empty())
            }),
            _ => false,
        };
        if matches!(
            frame.role()?,
            EnvironmentRole::Global | EnvironmentRole::Base | EnvironmentRole::BaseNamespace
        ) || frame.contains_local(".packageName")?
            || frame
                .name_attribute()?
                .is_some_and(|name| name.starts_with("package:"))
            || namespace
        {
            return Ok(frame);
        }
        current = frame.parent()?;
    }
    Ok(start.session_root(EnvironmentRole::Global)?)
}

impl RuntimeState {
    pub(super) fn lookup_method(
        &mut self,
        name: &str,
        environments: &MethodEnvironments,
    ) -> Result<Option<Sexp>, EvalError> {
        crate::base::initialize(&environments.call)?;
        let top = topenv(&environments.call)?;
        let mut current = Some(environments.call.clone());
        while let Some(frame) = current {
            if let Some(value) = self.method_in_frame(name, &frame)? {
                return Ok(Some(value));
            }
            if frame == top {
                break;
            }
            current = frame.parent()?;
        }
        let definition = match environments.definition.role()? {
            EnvironmentRole::Base => environments
                .definition
                .session_root(EnvironmentRole::BaseNamespace)?,
            _ => environments.definition.clone(),
        };
        if let Some(table) = definition.get_local(".__S3MethodsTable__.")? {
            if let Sexp::Environment(table) = self.force_dispatch_promise(table)? {
                if let Some(value) = table.get_local(name)? {
                    let value = self.force_dispatch_promise(value)?;
                    return Ok(is_function(&value).then_some(value));
                }
            }
        }
        current = match top.role()? {
            EnvironmentRole::Global => Some(top.session_root(EnvironmentRole::Base)?),
            _ => top.parent()?,
        };
        while let Some(frame) = current {
            if let Some(value) = self.method_in_frame(name, &frame)? {
                return Ok(Some(value));
            }
            current = match frame.role()? {
                EnvironmentRole::Global => Some(frame.session_root(EnvironmentRole::Base)?),
                _ => frame.parent()?,
            };
        }
        Ok(None)
    }

    fn method_in_frame(
        &mut self,
        name: &str,
        frame: &Environment,
    ) -> Result<Option<Sexp>, EvalError> {
        match frame.get_local(name)? {
            Some(value) => {
                let value = self.force_dispatch_promise(value)?;
                Ok(is_function(&value).then_some(value))
            }
            None => Ok(None),
        }
    }
}

fn is_function(value: &Sexp) -> bool {
    matches!(value, Sexp::Closure { .. } | Sexp::Primitive(_))
}
