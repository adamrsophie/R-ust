use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn new_environment(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let hash = self.scalar_integer_with_warning(&arguments[0].value)?;
        let parent = match &arguments[1].value {
            Sexp::Nil => {
                return Err(crate::SourceFault::domain(
                    "use of NULL environment is defunct".into(),
                    &self.source_current_call(),
                ))
            }
            Sexp::Environment(parent) => parent,
            _ => {
                return Err(crate::SourceFault::domain(
                    "'enclos' must be an environment".into(),
                    &self.source_current_call(),
                ))
            }
        };
        if hash != Some(0) {
            let _validated_size = self.scalar_integer_with_warning(&arguments[2].value)?;
        }
        // shortcut: storage ignores validated hash/size, revisit when env.profile is ported.
        Ok(Sexp::Environment(parent.child()?))
    }
}
