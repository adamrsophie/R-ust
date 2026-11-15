use crate::{dynamic_context::ContextKind, EvalError, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::{Environment, EnvironmentRole};

impl RuntimeState {
    pub(crate) fn as_environment(&mut self, value: &Sexp, call: &Sexp) -> Result<Sexp, EvalError> {
        if value.ordinary_list().is_some() {
            return Err(EvalError::NotImplemented(
                "as.environment list2env conversion",
            ));
        }
        match value {
            Sexp::Environment(_) => Ok(value.clone()),
            Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
                let name = values.first().and_then(Option::as_deref).unwrap_or("NA");
                let role = match name {
                    ".GlobalEnv" => EnvironmentRole::Global,
                    "package:base" => EnvironmentRole::Base,
                    "Autoloads" => EnvironmentRole::Autoloads,
                    _ => {
                        return Err(SourceFault::domain(
                            format!("no item called \"{name}\" on the search list"),
                            call,
                        ))
                    }
                };
                Ok(Sexp::Environment(self.global.session_root(role)?))
            }
            Sexp::Integer(_) | Sexp::Real(_) => {
                let values = self.integer_values_with_warning(value, call)?;
                let environments = values
                    .iter()
                    .map(|value| self.environment_position(*value, call))
                    .collect::<Result<Vec<_>, _>>()?;
                match environments.as_slice() {
                    [] => Err(SourceFault::domain("invalid 'pos' argument".into(), call)),
                    [environment] => Ok(Sexp::Environment(environment.clone())),
                    _ => Ok(Sexp::List(
                        environments
                            .into_iter()
                            .map(|environment| crate::ListElement {
                                name: None,
                                value: Sexp::Environment(environment),
                            })
                            .collect(),
                    )),
                }
            }
            Sexp::Nil => Err(SourceFault::domain(
                "using 'as.environment(NULL)' is defunct".into(),
                call,
            )),
            Sexp::List(_) | Sexp::ClassedList { .. } | Sexp::ListObject(_) => Err(
                EvalError::NotImplemented("as.environment list2env conversion"),
            ),
            _ => Err(SourceFault::domain(
                "invalid object for 'as.environment'".into(),
                call,
            )),
        }
    }

    fn environment_position(
        &self,
        position: Option<i32>,
        call: &Sexp,
    ) -> Result<Environment, EvalError> {
        match position {
            Some(-1) => self
                .control
                .contexts
                .iter()
                .rev()
                .find(|context| matches!(context.kind, ContextKind::Return(_)))
                .map(|context| context.sysparent.clone())
                .ok_or_else(|| SourceFault::domain("no enclosing environment".into(), call)),
            Some(mut position) if position > 0 => {
                let mut environment = self.global.clone();
                while position > 1 && environment.role()? != EnvironmentRole::Empty {
                    environment = environment
                        .parent()?
                        .ok_or_else(|| EvalError::Control("missing search-path parent".into()))?;
                    position -= 1;
                }
                if position == 1 {
                    Ok(environment)
                } else {
                    Err(SourceFault::domain("invalid 'pos' argument".into(), call))
                }
            }
            _ => Err(SourceFault::domain("invalid 'pos' argument".into(), call)),
        }
    }
}
