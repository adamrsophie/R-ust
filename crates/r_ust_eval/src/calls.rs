use crate::environment::{has_builtin, ApplyFunction};
use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_call(
        &mut self,
        function: &Sexp,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        let call = Sexp::Call {
            function: Box::new(function.clone()),
            arguments: arguments.to_vec(),
        };
        if let Sexp::Parenthesized(inner) = function {
            match self.require_function("(", true)? {
                ApplyFunction::Builtin(_) => {
                    let resolved = self.resolve_parenthesized_function(inner)?;
                    return self.call_resolved(resolved, arguments, &call);
                }
                resolved => {
                    let EvalFlow::Value(value) = self.call_resolved(
                        resolved,
                        &[CallArgument::positional(inner.as_ref().clone())],
                        &Sexp::Call {
                            function: Box::new(Sexp::Symbol("(".into())),
                            arguments: vec![CallArgument::positional(inner.as_ref().clone())],
                        },
                    )?;
                    return self.call_value(value, arguments, &call);
                }
            }
        }
        if let Sexp::Symbol(name) = function {
            let resolved = self.require_source_function(name, has_builtin(name), &call)?;
            return self.call_resolved(resolved, arguments, &call);
        }
        let EvalFlow::Value(value) = self.eval_flow(function)?;
        self.call_value(value, arguments, &call)
    }

    pub(super) fn eval_builtin_call(
        &mut self,
        name: &str,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        if name == ".Internal" {
            return self
                .eval_internal_call(arguments, call)
                .map(EvalFlow::Value);
        }
        if name == "...length" {
            return self.eval_dots_length(arguments).map(EvalFlow::Value);
        }
        if name == "anyNA" {
            return self.eval_any_na(arguments, call).map(EvalFlow::Value);
        }
        if matches!(name, "q" | "quit") {
            return self.eval_quit_call(arguments);
        }
        if name == "commandArgs" {
            return self.eval_command_args(arguments);
        }
        if name == "readline" {
            return self.eval_readline(arguments);
        }
        if name == "getOption" {
            return self.eval_get_option(arguments);
        }
        if matches!(name, "invisible" | "withVisible") {
            return self.eval_visibility_call(name, arguments);
        }
        if name == "with" {
            return self.eval_with_call(arguments).map(EvalFlow::Value);
        }
        if name == "subset" {
            let value = self.eval_subset_call(arguments)?;
            self.visible = true;
            return Ok(EvalFlow::Value(value));
        }
        if name == "transform" {
            let value = self.eval_transform_call(arguments)?;
            self.visible = true;
            return Ok(EvalFlow::Value(value));
        }
        if name == "apply" {
            let value = self.eval_matrix_apply_call(arguments)?;
            self.visible = true;
            return Ok(EvalFlow::Value(value));
        }
        if matches!(name, "lapply" | "sapply") {
            let value = self.eval_apply_call(name, arguments)?;
            self.visible = true;
            return Ok(EvalFlow::Value(value));
        }
        let evaluated = self.eval_arguments(
            arguments,
            matches!(
                name,
                "as.numeric" | "as.double" | "as.real" | "as.integer" | "sum" | "prod"
            ),
            Some(call),
        )?;
        self.call_builtin(name, &evaluated).map(EvalFlow::Value)
    }
}
