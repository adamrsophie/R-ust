mod environment;
#[cfg(test)]
mod tests;
mod unused;
mod walk;

use crate::{CallArgument, EvalError, EvalFlow, FunctionParameter, RuntimeState, Sexp};
use r_ust_sexp::{canonical_actual_call, LanguageError};

impl RuntimeState {
    pub(super) fn eval_substitute(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let parameters = ["expr", "env"].map(|name| FunctionParameter {
            name: name.into(),
            default: None,
        });
        let matched = self.match_arguments(&parameters, arguments, Some(call))?;
        if !matched.unused.is_empty() {
            return Err(crate::SourceFault::arity(
                unused::render(&matched.unused)?,
                call,
            ));
        }
        let expr = matched.slots[0].map(|argument| &argument.value);
        let env = matched.slots[1].map(|argument| &argument.value);
        let environment = match env {
            None | Some(Sexp::MissingArg(_) | Sexp::MissingValue) => {
                Sexp::Environment(self.current.clone())
            }
            Some(value) => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                value
            }
        };
        let lookup = environment::Lookup::new(&environment, call)?;
        let value = canonical(expr.unwrap_or(&Sexp::MissingValue))?;
        let value = walk::Substitution::new(lookup, call).root(&value)?;
        self.visible = true;
        Ok(EvalFlow::Value(value))
    }
}

fn canonical(value: &Sexp) -> Result<Sexp, EvalError> {
    canonical_actual_call(value).map_err(|error| match error {
        LanguageError::InternalCall => {
            EvalError::NotImplemented("substitution of internal dispatch")
        }
        LanguageError::RuntimeValue => EvalError::NotImplemented("substitution canonicalization"),
    })
}

fn promise_expression(value: &Sexp) -> Result<Sexp, EvalError> {
    let mut value = value.clone();
    while let Sexp::Promise { state } = value {
        value = state.expression()?;
    }
    canonical(&r_ust_sexp::bytecode_source(&value)?)
}
