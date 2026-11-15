use crate::{CallArgument, EvalError, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::EnvironmentRole;

impl RuntimeState {
    pub(super) fn eval_namespace(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let [package, member] = arguments else {
            return Err(SourceFault::arity(
                format!(
                    "{} arguments passed to namespace selection which requires 2",
                    arguments.len()
                ),
                call,
            ));
        };
        let package = name(&package.value, "package")?;
        let member = name(&member.value, "name")?;
        if package != "base" {
            return Err(EvalError::NotImplemented("loading non-base namespaces"));
        }
        crate::base::initialize(&self.current)?;
        let namespace = self.current.session_root(EnvironmentRole::BaseNamespace)?;
        let value = namespace.get_local(member)?.ok_or_else(|| {
            if crate::environment::has_builtin(member) {
                EvalError::NotImplemented("unported first-class base namespace binding")
            } else {
                SourceFault::domain(format!("object '{member}' not found"), &Sexp::Nil)
            }
        })?;
        self.force_value(value)
    }
}

fn name<'a>(value: &'a Sexp, label: &str) -> Result<&'a str, EvalError> {
    match value {
        Sexp::Symbol(name) if !name.is_empty() => Ok(name),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            match values.as_slice() {
                [Some(name)] if !name.is_empty() => Ok(name),
                _ => Err(EvalError::Domain(format!("bad {label} name"))),
            }
        }
        _ => Err(EvalError::Domain(format!("bad {label} name"))),
    }
}
