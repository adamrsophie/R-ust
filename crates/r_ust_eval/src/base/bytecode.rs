use crate::{EvalError, Sexp};
use r_ust_sexp::{ClosureEnvironment, Environment, EnvironmentRole};

pub(super) fn finalize(environment: &Environment) -> Result<(), EvalError> {
    if environment.base_bytecode_ready()? {
        return Ok(());
    }
    let namespace = environment.session_root(EnvironmentRole::BaseNamespace)?;
    let mut replacements = Vec::new();
    for (name, _) in super::iter()? {
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
            replacements.push((name.into(), previous, replacement));
        }
    }
    Ok(environment.finalize_base_bytecode(&replacements)?)
}
