use crate::{conditions::MissingSite, EvalError, EvalFlow, RuntimeState, SourceFault};

impl RuntimeState {
    pub(crate) fn evaluated_missing(&self, name: &str) -> EvalError {
        SourceFault::missing(
            name.into(),
            self.source_lexical_call(),
            MissingSite::EvaluatedSymbol,
        )
    }

    pub(crate) fn eval_symbol(&mut self, name: &str) -> Result<EvalFlow, EvalError> {
        if name == "..." {
            return Err(SourceFault::domain(
                "'...' used in an incorrect context".into(),
                &self.source_current_call(),
            ));
        }
        self.lookup(name)?
            .ok_or_else(|| SourceFault::undefined_symbol(name, &self.source_lexical_call()))
            .map(EvalFlow::Value)
    }
}
