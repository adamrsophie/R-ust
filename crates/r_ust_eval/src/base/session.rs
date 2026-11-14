use r_ust_sexp::{ClosureEnvironment, Environment, EnvironmentRole};

use super::{EvalError, Sexp};

pub(crate) fn initialize(environment: &Environment) -> Result<(), EvalError> {
    if environment.base_initialized()? {
        return Ok(());
    }
    let templates = super::iter()?;
    let namespace = environment.session_root(EnvironmentRole::BaseNamespace)?;
    let mut bindings = Vec::new();
    for (name, template) in templates {
        let mut value = template.clone();
        if let Sexp::Closure { environment, .. } = &mut value {
            if matches!(environment, ClosureEnvironment::Base) {
                *environment = ClosureEnvironment::Captured(namespace.clone());
            }
        }
        bindings.push((name.into(), value));
    }
    for (name, role) in [
        (".GlobalEnv", EnvironmentRole::Global),
        (".AutoloadEnv", EnvironmentRole::Autoloads),
        (".BaseNamespaceEnv", EnvironmentRole::BaseNamespace),
    ] {
        bindings.push((
            name.into(),
            Sexp::Environment(environment.session_root(role)?),
        ));
    }
    Ok(environment.install_base(&bindings)?)
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
