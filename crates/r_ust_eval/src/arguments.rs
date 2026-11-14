use super::{CallArgument, EvalError, FunctionParameter, HashMap, ListElement, RuntimeState, Sexp};

#[derive(Clone, Copy, PartialEq)]
enum MatchUse {
    Unused,
    Partial,
    Exact,
}

impl RuntimeState {
    pub(super) fn bind_promised_arguments(
        &mut self,
        parameters: &[FunctionParameter],
        supplied: &[CallArgument],
        call: Option<&Sexp>,
    ) -> Result<HashMap<String, Sexp>, EvalError> {
        let dots = parameters
            .iter()
            .position(|parameter| parameter.name == "...");
        let mut formal_use = vec![MatchUse::Unused; parameters.len()];
        let mut actual_use = vec![MatchUse::Unused; supplied.len()];
        let mut matched: Vec<Option<&CallArgument>> = vec![None; parameters.len()];
        // R match.c resolves every exact tag before considering any partial tag.
        for (formal, parameter) in parameters.iter().enumerate() {
            if parameter.name == "..." {
                continue;
            }
            for (actual, argument) in supplied.iter().enumerate() {
                if argument.name.as_deref() != Some(parameter.name.as_str()) {
                    continue;
                }
                if formal_use[formal] == MatchUse::Exact {
                    return Err(duplicate_formal(&parameter.name, call));
                }
                if actual_use[actual] == MatchUse::Exact {
                    return Err(ambiguous_actual(actual, call));
                }
                matched[formal] = Some(argument);
                formal_use[formal] = MatchUse::Exact;
                actual_use[actual] = MatchUse::Exact;
            }
        }
        for (formal, parameter) in parameters
            .iter()
            .enumerate()
            .take(dots.unwrap_or(parameters.len()))
        {
            if formal_use[formal] == MatchUse::Exact {
                continue;
            }
            for (actual, argument) in supplied.iter().enumerate() {
                let Some(name) = &argument.name else {
                    continue;
                };
                if actual_use[actual] == MatchUse::Exact || !parameter.name.starts_with(name) {
                    continue;
                }
                if actual_use[actual] == MatchUse::Partial {
                    return Err(ambiguous_actual(actual, call));
                }
                if formal_use[formal] == MatchUse::Partial {
                    return Err(duplicate_formal(&parameter.name, call));
                }
                self.warn_partial_argument(name, &parameter.name, call)?;
                matched[formal] = Some(argument);
                formal_use[formal] = MatchUse::Partial;
                actual_use[actual] = MatchUse::Partial;
            }
        }
        let mut positional = supplied
            .iter()
            .zip(&mut actual_use)
            .filter(|(argument, used)| **used == MatchUse::Unused && argument.name.is_none());
        for slot in matched.iter_mut().take(dots.unwrap_or(parameters.len())) {
            if slot.is_some_and(|argument| {
                !matches!(argument.value, Sexp::MissingArg(_) | Sexp::MissingValue)
            }) {
                continue;
            }
            let Some((argument, used)) = positional.next() else {
                break;
            };
            *slot = Some(argument);
            *used = MatchUse::Partial;
        }
        let unused = supplied
            .iter()
            .zip(&actual_use)
            .filter_map(|(argument, used)| (*used == MatchUse::Unused).then_some(argument))
            .collect::<Vec<_>>();
        if dots.is_none() && !unused.is_empty() {
            return Err(matching_error(self.render_unused_arguments(&unused)?, call));
        }
        let mut scope = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(matched) {
            if let Some(argument) = argument {
                scope.insert(parameter.name.clone(), argument.value.clone());
            }
        }
        if dots.is_some() {
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

fn duplicate_formal(name: &str, call: Option<&Sexp>) -> EvalError {
    matching_error(
        format!("formal argument \"{name}\" matched by multiple actual arguments"),
        call,
    )
}

fn ambiguous_actual(index: usize, call: Option<&Sexp>) -> EvalError {
    matching_error(
        format!("argument {} matches multiple formal arguments", index + 1),
        call,
    )
}

fn matching_error(message: String, call: Option<&Sexp>) -> EvalError {
    match call {
        Some(call) => crate::SourceFault::arity(message, call),
        None => EvalError::Arity(message),
    }
}
