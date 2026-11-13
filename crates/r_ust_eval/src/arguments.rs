use super::{promise, CallArgument, EvalError, FunctionParameter, HashMap, ListElement, Sexp};

pub(super) fn bind_supplied_argument_promises(
    parameters: &[FunctionParameter],
    supplied: &[CallArgument],
    caller_environment: &[(String, Sexp)],
    local_scope: &mut HashMap<String, Sexp>,
) -> Result<(), EvalError> {
    let dots_position = parameters
        .iter()
        .position(|parameter| parameter.name == "...");
    let mut dots_values = Vec::new();
    let mut next_positional = 0usize;
    for argument in supplied {
        if let Some(name) = &argument.name {
            if !parameters.iter().any(|parameter| parameter.name == *name) {
                if dots_position.is_some() {
                    dots_values.push(ListElement {
                        name: Some(name.clone()),
                        value: promise(argument.value.clone(), caller_environment.to_vec(), true),
                    });
                    continue;
                }
                return Err(EvalError::Arity(format!("unused argument {name}")));
            }
            if local_scope.contains_key(name) {
                return Err(EvalError::Arity(format!("argument {name} matched twice")));
            }
            local_scope.insert(
                name.clone(),
                promise(argument.value.clone(), caller_environment.to_vec(), true),
            );
            continue;
        }

        while parameters.get(next_positional).is_some_and(|parameter| {
            parameter.name != "..." && local_scope.contains_key(&parameter.name)
        }) {
            next_positional += 1;
        }
        if dots_position.is_some_and(|position| next_positional >= position) {
            dots_values.push(ListElement {
                name: None,
                value: promise(argument.value.clone(), caller_environment.to_vec(), true),
            });
            continue;
        }
        let Some(parameter) = parameters.get(next_positional) else {
            if dots_position.is_some() {
                dots_values.push(ListElement {
                    name: None,
                    value: promise(argument.value.clone(), caller_environment.to_vec(), true),
                });
                continue;
            }
            return Err(EvalError::Arity(
                "too many positional arguments".to_string(),
            ));
        };
        local_scope.insert(
            parameter.name.clone(),
            promise(argument.value.clone(), caller_environment.to_vec(), true),
        );
        next_positional += 1;
    }
    if dots_position.is_some() {
        local_scope.insert("...".to_string(), Sexp::Dots(dots_values));
    }
    Ok(())
}
