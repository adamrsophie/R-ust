use super::{EvalError, EvalFlow, RuntimeState, Sexp};
use r_ust_sexp::{Environment, EvaluationOrigin, Force, GraphError, PromiseState};

impl RuntimeState {
    pub(super) fn force_value(&mut self, value: Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::MissingArg(name) => Err(EvalError::MissingArgument(name)),
            Sexp::Promise { state } => {
                let force = state.begin_force().map_err(|error| match error {
                    GraphError::PromiseUnderEvaluation => EvalError::Domain(
                        "promise already under evaluation: recursive default argument reference or earlier problems?".into()),
                    other => EvalError::Graph(other),
                })?;
                match force {
                    Force::Cached(value) => Ok(value),
                    Force::Evaluate(lease) => {
                        if lease.interrupted {
                            self.record_parse_warnings(&[
                                "restarting interrupted promise evaluation".into(),
                            ])?;
                        }
                        let saved_frame =
                            std::mem::replace(&mut self.current, lease.environment.clone());
                        let saved_origin =
                            std::mem::replace(&mut self.control.origin, lease.origin.clone());
                        let result = self.eval_flow(&lease.expression);
                        self.current = saved_frame;
                        self.control.origin = saved_origin;
                        let EvalFlow::Value(value) = result?;
                        lease.complete(&value)?;
                        Ok(value)
                    }
                }
            }
            other => Ok(other),
        }
    }
}

pub(super) fn promise(
    expression: Sexp,
    environment: Environment,
    origin: EvaluationOrigin,
    supplied: bool,
) -> Result<Sexp, EvalError> {
    if matches!(expression, Sexp::MissingArg(_) | Sexp::MissingValue) {
        return Ok(expression);
    }
    Ok(Sexp::Promise {
        state: PromiseState::new(&expression, &environment, origin, supplied)?,
    })
}

pub(super) fn is_missing_binding(value: &Sexp) -> Result<bool, EvalError> {
    match value {
        Sexp::Promise { state } if !state.supplied()? => Ok(true),
        _ => forwarded_binding_missing(value, &mut Vec::new()),
    }
}

fn forwarded_binding_missing(
    value: &Sexp,
    visited: &mut Vec<PromiseState>,
) -> Result<bool, EvalError> {
    match value {
        Sexp::MissingArg(_) | Sexp::MissingValue => Ok(true),
        Sexp::Promise { state } => {
            if visited.contains(state) {
                return Ok(true);
            }
            let Some(environment) = state.environment()? else {
                return Ok(false);
            };
            visited.push(state.clone());
            match state.expression()? {
                promise @ Sexp::Promise { .. } => forwarded_binding_missing(&promise, visited),
                Sexp::Symbol(name) => {
                    if let Some(index) = name
                        .strip_prefix("..")
                        .and_then(|value| value.parse::<usize>().ok())
                        .and_then(|value| value.checked_sub(1))
                    {
                        return match environment.get_local("...")? {
                            Some(Sexp::Dots(values)) => {
                                values.get(index).map_or(Ok(true), |value| {
                                    forwarded_binding_missing(&value.value, visited)
                                })
                            }
                            Some(Sexp::MissingArg(_) | Sexp::MissingValue) => Ok(true),
                            _ => Ok(false),
                        };
                    }
                    match environment.get_local(&name)? {
                        Some(value) => forwarded_binding_missing(&value, visited),
                        None => Ok(false),
                    }
                }
                _ => Ok(false),
            }
        }
        _ => Ok(false),
    }
}
