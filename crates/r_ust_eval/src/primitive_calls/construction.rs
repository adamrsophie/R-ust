// Adapted from GNU R coerce.c do_call, GPL-2.0-or-later.
// Copyright R Core Team, R Foundation, Robert Gentleman and Ross Ihaka.
use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault};

impl RuntimeState {
    pub(super) fn construct_call(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let Some((first, rest)) = arguments.split_first() else {
            return Err(SourceFault::domain("'name' is missing".into(), call));
        };
        if let Some(name) = first.name.as_deref().filter(|name| *name != "name") {
            return Err(SourceFault::arity(
                format!("supplied argument name '{name}' does not match 'name'"),
                call,
            ));
        }
        let EvalFlow::Value(value) = self.eval_syntax_operand(&first.value, call)?;
        let names = if crate::is_character_value(&value) {
            crate::string_values(&value)?
        } else {
            Vec::new()
        };
        let [name] = names.as_slice() else {
            return Err(SourceFault::domain(
                "first argument must be a character string".into(),
                call,
            ));
        };
        let name = name.as_deref().unwrap_or("NA");
        if name == ".Internal" {
            return Err(SourceFault::domain(
                "illegal usage".into(),
                &self.source_current_call(),
            ));
        }
        if name.is_empty() {
            return Err(SourceFault::domain(
                "attempt to use zero-length variable name".into(),
                &self.source_current_call(),
            ));
        }
        let mut values = Vec::with_capacity(rest.len());
        for argument in rest {
            let EvalFlow::Value(value) = self.eval_syntax_operand(&argument.value, call)?;
            values.push(CallArgument {
                name: argument.name.clone(),
                value,
            });
        }
        let result = Sexp::Call {
            function: Box::new(Sexp::Symbol(name.into())),
            arguments: values,
        };
        self.visible = true;
        Ok(result)
    }
}
