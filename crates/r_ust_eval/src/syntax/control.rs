use super::{
    arity, at_call, CallArgument, EvalError, EvalFlow, RuntimeState, Sexp, SourceFault,
    SyntaxPrimitive,
};

impl RuntimeState {
    pub(super) fn call_control_syntax(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        use SyntaxPrimitive as P;
        match id {
            P::If => {
                let condition = arguments.first().map_or(&Sexp::Nil, |arg| &arg.value);
                let EvalFlow::Value(condition) = self.eval_syntax_operand(condition, call)?;
                let take_then =
                    crate::condition_truth(&condition).map_err(|error| at_call(error, call))?;
                let selected = arguments.get(if take_then { 1 } else { 2 });
                if !take_then && selected.is_none() {
                    self.visible = false;
                    return Ok(EvalFlow::Value(Sexp::Nil));
                }
                self.eval_syntax_operand(selected.map_or(&Sexp::Nil, |arg| &arg.value), call)
            }
            P::Block => {
                let mut result = EvalFlow::Value(Sexp::Nil);
                for argument in arguments {
                    result = self.eval_syntax_operand(&argument.value, call)?;
                }
                Ok(result)
            }
            P::Function => self.syntax_function(arguments, call),
            P::For | P::While | P::Repeat | P::Break | P::Next => {
                self.syntax_loop(id, arguments, call)
            }
            P::AndScalar | P::OrScalar => {
                let [left, right] = arguments else {
                    return Err(SourceFault::arity(arity(id, arguments.len(), 2), call));
                };
                let op = if id == P::AndScalar {
                    crate::BinaryOp::AndScalar
                } else {
                    crate::BinaryOp::OrScalar
                };
                self.eval_selected_binary(op, (&operand(&left.value), &operand(&right.value)))
                    .map_err(|error| at_call(error, call))
            }
            _ => Err(EvalError::Domain(
                "control syntax dispatch invariant".into(),
            )),
        }
    }

    fn syntax_loop(
        &mut self,
        id: SyntaxPrimitive,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        use SyntaxPrimitive as P;
        let expected = match id {
            P::For => 3,
            P::While => 2,
            P::Repeat => 1,
            _ => 0,
        };
        if arguments.len() != expected {
            return Err(SourceFault::arity(
                arity(id, arguments.len(), expected),
                call,
            ));
        }
        let result = match (id, arguments) {
            (P::For, [variable, sequence, body]) => {
                let Sexp::Symbol(variable) = &variable.value else {
                    return Err(SourceFault::type_error(
                        "non-symbol loop variable".into(),
                        call,
                    ));
                };
                self.eval_for(variable, &operand(&sequence.value), &operand(&body.value))
            }
            (P::While, [condition, body]) => {
                self.eval_while(&operand(&condition.value), &operand(&body.value))
            }
            (P::Repeat, [body]) => self.eval_repeat(&operand(&body.value)),
            (P::Break, []) => Err(self
                .control
                .loop_signal(&self.current, crate::ControlTransfer::Break)),
            (P::Next, []) => Err(self
                .control
                .loop_signal(&self.current, crate::ControlTransfer::Next)),
            _ => Err(EvalError::Domain("loop syntax dispatch invariant".into())),
        };
        if result.is_ok() {
            self.visible = false;
        }
        result.map_err(|error| at_call(error, call))
    }
}

fn operand(value: &Sexp) -> Sexp {
    match value {
        Sexp::MissingValue => Sexp::MissingArg(String::new()),
        other => other.clone(),
    }
}
