use crate::{is_builtin, CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_call(
        &mut self,
        function: &Sexp,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        if let Sexp::Parenthesized(function) = function {
            return self.eval_call(function, arguments);
        }
        if let Sexp::Symbol(name) = function {
            if name == "missing" {
                return self.eval_missing(arguments);
            }
            if name == "return" {
                return self.eval_return(arguments);
            }
            if let Some(value) = self.lookup_function(name)? {
                return self.call_value(value, arguments);
            }
            if name == ".Internal" {
                return self.eval_internal_call(arguments).map(EvalFlow::Value);
            }
            if name == "...length" {
                return self.eval_dots_length(arguments).map(EvalFlow::Value);
            }
            if is_builtin(name) {
                if name == "anyNA" {
                    return self.eval_any_na(arguments).map(EvalFlow::Value);
                }
                if matches!(name.as_str(), "q" | "quit") {
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
                if matches!(name.as_str(), "invisible" | "withVisible") {
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
                if matches!(name.as_str(), "lapply" | "sapply") {
                    let value = self.eval_apply_call(name, arguments)?;
                    self.visible = true;
                    return Ok(EvalFlow::Value(value));
                }
                let evaluated = self.eval_arguments(
                    arguments,
                    matches!(
                        name.as_str(),
                        "as.numeric" | "as.double" | "as.real" | "as.integer" | "sum" | "prod"
                    ),
                )?;
                return self.call_builtin(name, &evaluated).map(EvalFlow::Value);
            }
            return Err(EvalError::UndefinedFunction(name.clone()));
        }

        let EvalFlow::Value(value) = self.eval_flow(function)?;
        self.call_value(value, arguments)
    }
}
