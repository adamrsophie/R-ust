use super::{EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn lookup_function(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        let mut current = Some(self.current.clone());
        while let Some(frame) = current {
            if let Some(value) = frame.get_local(name)? {
                let value = self.force_value(value)?;
                if matches!(value, Sexp::Closure { .. }) {
                    return Ok(Some(value));
                }
            }
            current = frame.parent()?;
        }
        super::base::lookup(name)
            .map(|value| value.filter(|value| matches!(value, Sexp::Closure { .. })))
    }

    pub(super) fn lookup(&mut self, name: &str) -> Result<Option<Sexp>, EvalError> {
        if let Some(index) = name
            .strip_prefix("..")
            .and_then(|s| s.parse::<usize>().ok())
        {
            return self.lookup_dot(index).map(Some);
        }
        match self.raw_lookup(name)? {
            Some(value) => self.force_value(value).map(Some),
            None => super::base::lookup(name),
        }
    }

    pub(super) fn raw_lookup(&self, name: &str) -> Result<Option<Sexp>, EvalError> {
        Ok(self.current.get(name)?)
    }

    pub(super) fn assign_local(&mut self, name: String, value: Sexp) -> Result<(), EvalError> {
        Ok(self.current.insert(name, &value)?)
    }

    pub(super) fn assign_super(&mut self, name: String, value: Sexp) -> Result<(), EvalError> {
        Ok(self.current.assign_parent(name, &value)?)
    }
}
