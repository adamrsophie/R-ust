mod classes;
mod lookup;
mod method;

use crate::dynamic_context::{Context, ContextKind};
use crate::{CallArgument, EvalError, EvalFlow, FunctionParameter, RuntimeState, Sexp};
use lookup::{topenv, MethodEnvironments};
use method::SelectedMethod;
use r_ust_sexp::{ClosureEnvironment, EnvironmentRole};

impl RuntimeState {
    pub(super) fn eval_use_method(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let parameters = ["generic", "object"].map(|name| FunctionParameter {
            name: name.into(),
            default: None,
        });
        let matched = self.bind_promised_arguments(&parameters, arguments, Some(call))?;
        let generic = matched
            .get("generic")
            .filter(|value| !matches!(value, Sexp::MissingValue | Sexp::MissingArg(_)))
            .ok_or_else(|| EvalError::Arity("there must be a 'generic' argument".into()))?;
        let generic = self.eval_regular_argument(generic)?;
        let values = generic
            .character_values()
            .filter(|values| values.len() == 1)
            .ok_or_else(|| {
                EvalError::Type("'generic' argument must be a character string".into())
            })?;
        let generic = values[0].as_deref().unwrap_or("NA").to_owned();
        let context = self
            .control
            .contexts
            .last()
            .filter(|entry| {
                matches!(entry.kind, ContextKind::Return(_)) && entry.execution == self.current
            })
            .cloned()
            .ok_or_else(|| {
                EvalError::Domain("'UseMethod' used in an inappropriate fashion".into())
            })?;
        let environments = MethodEnvironments {
            call: context.sysparent.clone(),
            definition: topenv(&self.current)?,
        };
        let object = match matched.get("object") {
            None | Some(Sexp::MissingValue | Sexp::MissingArg(_)) => {
                self.dispatch_object(&context)?
            }
            Some(value) => self.eval_regular_argument(value)?,
        };
        let class = classes::classes(&object)?;
        let names = class
            .character_values()
            .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
        for (index, name) in names.iter().enumerate() {
            let name = format!("{generic}.{}", name.as_deref().unwrap_or("NA"));
            if let Some(callable) = self.lookup_method(&name, &environments)? {
                let base_sort_list = match &callable {
                    Sexp::Closure {
                        environment: ClosureEnvironment::Base,
                        ..
                    } => name == "sort.list",
                    Sexp::Closure {
                        environment: ClosureEnvironment::Captured(environment),
                        ..
                    } => {
                        name == "sort.list"
                            && *environment
                                == environment.session_root(EnvironmentRole::BaseNamespace)?
                    }
                    _ => false,
                };
                if base_sort_list {
                    continue;
                }
                return self.dispatch_method(
                    context,
                    SelectedMethod {
                        callable,
                        name,
                        generic,
                        class: classes::suffix(&class, index)?,
                        environments,
                    },
                );
            }
        }
        let name = format!("{generic}.default");
        if let Some(callable) = self.lookup_method(&name, &environments)? {
            return self.dispatch_method(
                context,
                SelectedMethod {
                    callable,
                    name,
                    generic,
                    class: Sexp::Nil,
                    environments,
                },
            );
        }
        let description = match names {
            [name] => name.as_deref().unwrap_or("NA").to_owned(),
            names => format!(
                "c('{}')",
                names
                    .iter()
                    .map(|name| name.as_deref().unwrap_or("NA"))
                    .collect::<Vec<_>>()
                    .join("', '")
            ),
        };
        Err(EvalError::Type(format!(
            "no applicable method for '{generic}' applied to an object of class \"{description}\""
        )))
    }

    fn dispatch_object(&mut self, context: &Context) -> Result<Sexp, EvalError> {
        let Sexp::Closure { parameters, .. } = &context.callable else {
            return Err(EvalError::Type(
                "generic 'function' is not a function".into(),
            ));
        };
        let first = parameters
            .first()
            .filter(|parameter| parameter.name != "...");
        let mut selected = None;
        if let Some(parameter) = first {
            for exact in [true, false] {
                for argument in &context.arguments {
                    if argument.name.as_ref().is_some_and(|name| {
                        if exact {
                            parameter.name == *name
                        } else {
                            parameter.name.starts_with(name)
                        }
                    }) {
                        if selected.is_some() {
                            return Err(EvalError::Arity(format!(
                                "formal argument \"{}\" matched by multiple actual arguments",
                                parameter.name
                            )));
                        }
                        selected = Some(&argument.value);
                    }
                }
                if selected.is_some() {
                    break;
                }
            }
            selected = selected.or_else(|| {
                context
                    .arguments
                    .iter()
                    .find(|argument| argument.name.is_none())
                    .map(|argument| &argument.value)
            });
        }
        let value = selected
            .or_else(|| context.arguments.first().map(|argument| &argument.value))
            .cloned()
            .unwrap_or(Sexp::Nil);
        self.force_dispatch_promise(value)
    }

    fn force_dispatch_promise(&mut self, value: Sexp) -> Result<Sexp, EvalError> {
        match value {
            promise @ Sexp::Promise { .. } => self.force_value(promise),
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests;
