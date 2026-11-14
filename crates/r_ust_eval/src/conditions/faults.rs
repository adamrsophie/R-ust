use crate::{EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp};

impl EvalError {
    pub(crate) fn unhandled(self) -> Self {
        match self {
            Self::Signaled(error) => error.unhandled(),
            Self::Fresh(fault) => fault.original.unhandled(),
            Self::Condition(_) => {
                Self::Control("no function to return from, jumping to top level".into())
            }
            Self::Transfer(transfer) => Self::Control(transfer.inactive_message().into()),
            Self::WithWarnings { error, warnings } => Self::WithWarnings {
                error: Box::new(error.unhandled()),
                warnings,
            },
            error => error,
        }
    }
}

impl RuntimeState {
    pub(crate) fn eval_flow(&mut self, expr: &Sexp) -> Result<EvalFlow, EvalError> {
        let result = self.eval_flow_body(expr);
        self.signal_fresh_result(result)
    }

    pub(crate) fn signal_fresh_result<T>(
        &mut self,
        result: Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        let Err(EvalError::Fresh(fault)) = result else {
            return result;
        };
        self.conditions.message = fault.message.clone();
        if !self.conditions.has_match(&fault.classes())? {
            return Err(EvalError::Signaled(fault.original));
        }
        let call = fault
            .call
            .as_ref()
            .map_err(|_| EvalError::NotImplemented("condition call source representation"))?;
        if let Some(condition) = fault.condition(call) {
            let rooted = r_ust_sexp::ListObject::from_value(&self.current, &condition)?;
            self.signal_condition(&[
                EvaluatedArgument {
                    name: None,
                    value: Sexp::ListObject(rooted),
                },
                EvaluatedArgument {
                    name: None,
                    value: Sexp::Nil,
                },
                EvaluatedArgument {
                    name: None,
                    value: call.clone(),
                },
            ])?;
        } else {
            self.signal_simple_error(&fault.message, call)?;
        }
        self.conditions.message = fault.message;
        Err(EvalError::Signaled(fault.original))
    }

    pub(crate) fn default_condition_stop(
        &mut self,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [message, call] = args else {
            return Err(EvalError::Arity(
                "2 arguments required by '.dfltStop'".into(),
            ));
        };
        let (Sexp::String(values) | Sexp::AttributedCharacter { values, .. }) = &message.value
        else {
            return Err(EvalError::Domain("bad error message".into()));
        };
        let [message] = values.as_slice() else {
            return Err(EvalError::Domain("bad error message".into()));
        };
        let message = message.clone().unwrap_or_else(|| "NA".into());
        self.conditions.message = message.clone();
        Err(EvalError::Signaled(Box::new(EvalError::Stop {
            message,
            call_requested: !matches!(call.value, Sexp::Nil),
        })))
    }
}
