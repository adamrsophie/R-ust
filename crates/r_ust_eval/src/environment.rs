use std::collections::HashMap;
use std::sync::atomic::Ordering;

use r_ust_sexp::EvaluationOrigin;

use super::{EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    fn scope_visible(&self, index: usize) -> bool {
        index >= self.lookup_start
            && (self.lookup_start != 0
                || !self.base_scopes.iter().any(|range| range.contains(&index)))
    }

    pub(super) fn lookup_function(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        for index in (self.lookup_start..self.scopes.len()).rev() {
            if !self.scope_visible(index) {
                continue;
            }
            if let Some(value) = self.scopes[index].get(name).cloned() {
                let value = self.force_value(value)?;
                if matches!(value, Sexp::Closure { .. }) {
                    return Ok(Some(value));
                }
            }
        }
        super::base::lookup(name)
    }

    pub(super) fn lookup(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        if let Some(index) = name
            .strip_prefix("..")
            .and_then(|s| s.parse::<usize>().ok())
        {
            return self.lookup_dot(index).map(Some);
        }
        match self.raw_lookup(name).cloned() {
            Some(value) => self.force_value(value).map(Some),
            None => super::base::lookup(name),
        }
    }

    pub(super) fn raw_lookup(&self, name: &str) -> Option<&Sexp> {
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .filter(|(index, _)| self.scope_visible(*index))
            .find_map(|(_, scope)| scope.get(name))
    }

    pub(super) fn force_value(&mut self, value: Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::MissingArg(name) => Err(EvalError::MissingArgument(name)),
            Sexp::Promise {
                expression,
                environment,
                origin,
                state,
                ..
            } => {
                if let Some(value) = state.value.get() {
                    return Ok((**value).clone());
                }
                if state.evaluating.swap(true, Ordering::SeqCst) {
                    return Err(EvalError::Domain(
                        "promise already under evaluation: recursive default argument reference or earlier problems?".into(),
                    ));
                }
                let result = (|| {
                    if let Some(value) = state.value.get() {
                        return Ok((**value).clone());
                    }
                    if state.interrupted.load(Ordering::SeqCst) {
                        self.record_parse_warnings(&[
                            "restarting interrupted promise evaluation".into()
                        ])?;
                    }
                    let value = self.eval_in_environment(&environment, origin, &expression)?;
                    state.value.set(Box::new(value.clone())).map_err(|_| {
                        EvalError::Control("promise completed more than once".into())
                    })?;
                    Ok(value)
                })();
                state.interrupted.store(result.is_err(), Ordering::SeqCst);
                state.evaluating.store(false, Ordering::SeqCst);
                result
            }
            other => Ok(other),
        }
    }

    fn eval_in_environment(
        &mut self,
        environment: &[(String, Sexp)],
        origin: EvaluationOrigin,
        expression: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let previous_lookup = self.lookup_start;
        let base = self.base_origins.contains(&origin);
        self.lookup_start = if base {
            self.base_scopes
                .push(self.scopes.len()..self.scopes.len() + 1);
            self.scopes.len()
        } else {
            0
        };
        self.scopes
            .push(environment.iter().cloned().collect::<HashMap<_, _>>());
        let previous = std::mem::replace(&mut self.control.origin, origin);
        let result = self.eval_flow(expression);
        self.control.origin = previous;
        self.lookup_start = previous_lookup;
        self.scopes.pop();
        if base {
            self.base_scopes.pop();
        }
        let EvalFlow::Value(value) = result?;
        Ok(value)
    }

    pub(super) fn assign_local(&mut self, name: String, value: Sexp) {
        self.scopes
            .last_mut()
            .expect("runtime always has a scope")
            .insert(name, value);
    }

    pub(super) fn assign_super(&mut self, name: String, value: Sexp) {
        let current_scope = self.scopes.len().saturating_sub(1);
        let mut updated = false;
        for scope_index in 0..current_scope {
            if !self.scope_visible(scope_index) {
                continue;
            }
            if self.scopes[scope_index].contains_key(&name) {
                self.scopes[scope_index].insert(name.clone(), value.clone());
                updated = true;
            }
        }
        if !updated {
            self.scopes
                .first_mut()
                .expect("runtime always has a global scope")
                .insert(name, value);
        }
    }

    pub(super) fn current_scope_contains(&self, name: &str) -> bool {
        self.scopes
            .last()
            .is_some_and(|scope| scope.contains_key(name))
    }

    pub(super) fn visible_environment(&self) -> Vec<(String, Sexp)> {
        let mut merged = HashMap::new();
        for (index, scope) in self.scopes.iter().enumerate() {
            if !self.scope_visible(index) {
                continue;
            }
            for (name, value) in scope {
                merged.insert(name.clone(), value.clone());
            }
        }
        merged.into_iter().collect()
    }
}
