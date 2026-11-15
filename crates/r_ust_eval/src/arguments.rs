use super::{CallArgument, EvalError, FunctionParameter, HashMap, ListElement, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn bind_promised_arguments(
        &mut self,
        parameters: &[FunctionParameter],
        supplied: &[CallArgument],
        call: Option<&Sexp>,
    ) -> Result<HashMap<String, Sexp>, EvalError> {
        let matched = self.match_arguments(parameters, supplied, call)?;
        let unused = matched.unused;
        if !matched.dots && !unused.is_empty() {
            return Err(matching_error(self.render_unused_arguments(&unused)?, call));
        }
        let mut scope = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(matched.slots) {
            if let Some(argument) = argument {
                scope.insert(parameter.name.clone(), argument.value.clone());
            }
        }
        if matched.dots {
            scope.insert(
                "...".into(),
                Sexp::Dots(
                    unused
                        .into_iter()
                        .map(|argument| {
                            Ok(ListElement {
                                name: argument.name.clone(),
                                value: match &argument.value {
                                    Sexp::MissingArg(_) | Sexp::MissingValue => {
                                        Sexp::MissingArg(String::new())
                                    }
                                    value => value.clone(),
                                },
                            })
                        })
                        .collect::<Result<Vec<_>, EvalError>>()?,
                ),
            );
        }
        Ok(scope)
    }
}

fn matching_error(message: String, call: Option<&Sexp>) -> EvalError {
    match call {
        Some(call) => crate::SourceFault::arity(message, call),
        None => EvalError::Arity(message),
    }
}
