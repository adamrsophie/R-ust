use super::{
    bool_arg, console_event, data_args, dispatch_call, expand_dots_call_arguments, format_line,
    format_sexp, logical_scalar, named_arg, named_bool_arg, output_strings, positional,
    string_scalar, value_len, visibility_result, CallArgument, EvalError, EvalFlow,
    EvaluatedArgument, OutputEvent, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_arguments(
        &mut self,
        arguments: &[CallArgument],
        empty_is_error: bool,
    ) -> Result<Vec<EvaluatedArgument>, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let mut values = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.iter().enumerate() {
            if empty_is_error && matches!(argument.value, Sexp::MissingArg(_)) {
                return Err(EvalError::Arity(format!("argument {} is empty", index + 1)));
            }
            let EvalFlow::Value(value) = self.eval_flow(&argument.value)?;
            values.push(EvaluatedArgument {
                name: argument.name.clone(),
                value,
            });
        }
        Ok(values)
    }

    pub(super) fn call_builtin(
        &mut self,
        name: &str,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        self.visible = true;
        let result = match name {
            "q" | "quit" | "commandArgs" | "readline" | "getOption" => {
                let arguments = args
                    .iter()
                    .map(|arg| CallArgument {
                        name: arg.name.clone(),
                        value: arg.value.clone(),
                    })
                    .collect::<Vec<_>>();
                let result = match name {
                    "commandArgs" => self.eval_command_args(&arguments),
                    "readline" => self.eval_readline(&arguments),
                    "getOption" => self.eval_get_option(&arguments),
                    _ => self.eval_quit_call(&arguments),
                };
                let EvalFlow::Value(value) = result?;
                return Ok(value);
            }
            "anyNA" => return self.call_any_na(args),
            "options" => return self.call_options(args),
            "as.numeric" | "as.double" | "as.real" | "as.integer" => {
                self.call_numeric_coercion(name, args)?
            }
            "interactive" => {
                if !args.is_empty() {
                    return Err(EvalError::Arity(
                        "interactive() takes no arguments".to_string(),
                    ));
                }
                Sexp::Logical(vec![Some(self.interactive)])
            }
            "print" => {
                let value = positional(args, 0, "x")?.clone();
                let quote = named_bool_arg(args, &["quote"], true)?;
                let text = if !quote {
                    if let Sexp::String(values) = &value {
                        format_line(
                            values
                                .iter()
                                .map(|value| value.clone().unwrap_or_else(|| "NA".to_string()))
                                .collect(),
                        )
                    } else {
                        format_sexp(&value)
                    }
                } else {
                    format_sexp(&value)
                };
                self.emit(OutputEvent::Stdout(format!("{text}\n")))?;
                self.visible = false;
                value
            }
            "invisible" | "withVisible" => {
                if args.len() > 1 || (name == "withVisible" && args.is_empty()) {
                    return Err(EvalError::Arity(format!(
                        "invalid number of arguments to {name}()"
                    )));
                }
                let value = args
                    .first()
                    .map(|arg| arg.value.clone())
                    .unwrap_or(Sexp::Nil);
                if name == "invisible" {
                    self.visible = false;
                    value
                } else {
                    visibility_result(value, true)
                }
            }
            "cat" => {
                for unsupported in ["fill", "labels"] {
                    if let Some(value) = named_arg(args, &[unsupported]) {
                        if !matches!(value, Sexp::Nil)
                            && (unsupported == "labels"
                                || logical_scalar(value, "fill")? != Some(false))
                        {
                            return Err(EvalError::NotImplemented("cat fill and labels"));
                        }
                    }
                }
                let separator = named_arg(args, &["sep"])
                    .map(|value| {
                        if value_len(value) != 1 {
                            return Err(EvalError::NotImplemented("multiple cat separators"));
                        }
                        string_scalar(value, "sep")
                    })
                    .transpose()?
                    .unwrap_or_else(|| " ".to_string());
                let pieces = output_strings(
                    &data_args(args, &["file", "sep", "fill", "labels", "append"]),
                    true,
                )?;
                let mut text = pieces.join(&separator);
                if !pieces.is_empty() && separator.contains('\n') && !text.ends_with('\n') {
                    text.push('\n');
                }
                self.emit(console_event(named_arg(args, &["file"]), text)?)?;
                self.visible = false;
                Sexp::Nil
            }
            "writeLines" => {
                let _use_bytes = bool_arg(args, 3, &["useBytes"], Some(false))?;
                if positional(args, 1, "con").is_ok() {
                    return Err(EvalError::NotImplemented("writeLines connections"));
                }
                let value = positional(args, 0, "text")?;
                let Sexp::String(values) = value else {
                    return Err(EvalError::Type(
                        "can only write character objects".to_string(),
                    ));
                };
                let sep = positional(args, 2, "sep")
                    .ok()
                    .map(|value| string_scalar(value, "sep"))
                    .transpose()?
                    .unwrap_or_else(|| "\n".to_string());
                let mut text = String::new();
                for value in values {
                    text.push_str(value.as_deref().unwrap_or("NA"));
                    text.push_str(&sep);
                }
                self.emit(OutputEvent::Stdout(text))?;
                self.visible = false;
                Sexp::Nil
            }
            "message" => {
                if named_arg(args, &["domain"]).is_some_and(|value| !matches!(value, Sexp::Nil)) {
                    return Err(EvalError::NotImplemented("message translation domains"));
                }
                let mut text =
                    output_strings(&data_args(args, &["domain", "appendLF"]), false)?.concat();
                if named_bool_arg(args, &["appendLF"], true)? {
                    text.push('\n');
                }
                self.emit(OutputEvent::Stderr(text))?;
                self.visible = false;
                Sexp::Nil
            }
            _ => {
                let value = dispatch_call(name, args)?;
                self.visible = !matches!(name, "set.seed" | "set_seed");
                value
            }
        };
        Ok(result)
    }
}
