use crate::{EvalError, Sexp};
use r_ust_sexp::{ClosureEnvironment, Environment, EnvironmentRole};

pub(super) fn finalize(environment: &Environment) -> Result<(), EvalError> {
    if environment.base_bytecode_ready()? {
        return Ok(());
    }
    let namespace = environment.session_root(EnvironmentRole::BaseNamespace)?;
    let mut replacements = Vec::new();
    for (name, _) in super::iter()? {
        if super::quit_sources::ALIASES
            .iter()
            .any(|(alias, _)| *alias == name)
        {
            continue;
        }
        let Some(previous) = namespace.get_local(name)? else {
            continue;
        };
        let Sexp::Closure {
            parameters,
            body,
            environment: capture,
            attributes,
        } = &previous
        else {
            continue;
        };
        let ClosureEnvironment::Captured(owner) = capture else {
            continue;
        };
        if matches!(body.as_ref(), Sexp::Bytecode(_)) {
            continue;
        }
        if let Some(code) = crate::bytecode::compile::expression(owner, body)? {
            let replacement = Sexp::Closure {
                parameters: parameters.clone(),
                body: Box::new(Sexp::Bytecode(code)),
                environment: capture.clone(),
                attributes: attributes.clone(),
            };
            for (alias, original) in super::quit_sources::ALIASES {
                if *original == name {
                    if let Some(old_alias) = namespace.get_local(alias)? {
                        if old_alias == previous {
                            replacements.push(((*alias).into(), old_alias, replacement.clone()));
                        }
                    }
                }
            }
            replacements.push((name.into(), previous, replacement));
        }
    }
    Ok(environment.finalize_base_bytecode(&replacements)?)
}
