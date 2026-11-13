use std::collections::HashMap;

use super::{EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn lookup(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        let Some(scope_index) = (0..self.scopes.len())
            .rev()
            .find(|index| self.scopes[*index].contains_key(name))
        else {
            return Ok(None);
        };
        let value = self.scopes[scope_index]
            .get(name)
            .expect("scope index was selected by contains_key")
            .clone();
        let forced = match value {
            Sexp::MissingArg(name) => return Err(EvalError::MissingArgument(name)),
            Sexp::Promise {
                value: Some(value), ..
            } => *value,
            Sexp::Promise {
                expression,
                environment,
                value: None,
                supplied,
            } => {
                let forced = self.eval_in_environment(&environment, &expression)?;
                self.scopes[scope_index].insert(
                    name.to_string(),
                    Sexp::Promise {
                        expression,
                        environment,
                        value: Some(Box::new(forced.clone())),
                        supplied,
                    },
                );
                forced
            }
            other => other,
        };
        Ok(Some(forced))
    }

    pub(super) fn raw_lookup(&self, name: &str) -> Option<&Sexp> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    pub(super) fn force_value(&mut self, value: Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::MissingArg(name) => Err(EvalError::MissingArgument(name)),
            Sexp::Promise {
                value: Some(value), ..
            } => Ok(*value),
            Sexp::Promise {
                expression,
                environment,
                value: None,
                ..
            } => self.eval_in_environment(&environment, &expression),
            other => Ok(other),
        }
    }

    fn eval_in_environment(
        &mut self,
        environment: &[(String, Sexp)],
        expression: &Sexp,
    ) -> Result<Sexp, EvalError> {
        self.scopes
            .push(environment.iter().cloned().collect::<HashMap<_, _>>());
        let result = self.eval_flow(expression);
        self.scopes.pop();
        match result? {
            EvalFlow::Value(value) | EvalFlow::Return(value) => Ok(value),
            EvalFlow::Break => Err(EvalError::Control("break used inside promise".to_string())),
            EvalFlow::Next => Err(EvalError::Control("next used inside promise".to_string())),
        }
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
        for scope in &self.scopes {
            for (name, value) in scope {
                merged.insert(name.clone(), value.clone());
            }
        }
        merged.into_iter().collect()
    }
}
