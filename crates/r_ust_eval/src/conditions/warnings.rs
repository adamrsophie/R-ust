use crate::{CallArgument, EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};

impl RuntimeState {
    pub(crate) fn signal_simple_warning(
        &mut self,
        message: &str,
        call: &Sexp,
    ) -> Result<(), EvalError> {
        let call = r_ust_sexp::canonical_actual_call(call)
            .map_err(|_| EvalError::NotImplemented("warning call source representation"))?;
        let invocation = Sexp::Call {
            function: Box::new(Sexp::Symbol(".signalSimpleWarning".into())),
            arguments: vec![
                CallArgument::positional(Sexp::String(vec![Some(message.into())])),
                CallArgument::positional(Sexp::Call {
                    function: Box::new(Sexp::Call {
                        function: Box::new(Sexp::Symbol("::".into())),
                        arguments: vec![
                            CallArgument::positional(Sexp::Symbol("base".into())),
                            CallArgument::positional(Sexp::Symbol("quote".into())),
                        ],
                    }),
                    arguments: vec![CallArgument::positional(call.clone())],
                }),
            ],
        };
        let visible = self.visible;
        self.call_base_condition_hook(&invocation, (1, &call))?;
        self.visible = visible;
        Ok(())
    }

    pub(crate) fn default_condition_warning(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let Some([message]) = arguments[0].value.character_values() else {
            return Err(SourceFault::domain(
                "bad error message".into(),
                &self.source_current_call(),
            ));
        };
        let call = match &arguments[1].value {
            Sexp::Nil => None,
            value => Some(value),
        };
        self.default_warning(message.as_deref().unwrap_or("NA"), call)?;
        Ok(Sexp::Nil)
    }
}
