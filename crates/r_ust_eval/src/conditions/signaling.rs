use super::{entry::Entry, ConditionJump};
use crate::{CallArgument, EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp};
use r_ust_sexp::Environment;

impl RuntimeState {
    pub(crate) fn signal_simple_error(
        &mut self,
        message: &str,
        call: &Sexp,
    ) -> Result<(), EvalError> {
        let classes = [
            Some("simpleError".into()),
            Some("error".into()),
            Some("condition".into()),
        ];
        while let Some(entry) = self.conditions.select(&classes)? {
            self.conditions.message = message.into();
            if entry.calling()? {
                let invocation = Sexp::Call {
                    function: Box::new(Sexp::Symbol(".handleSimpleError".into())),
                    arguments: vec![
                        CallArgument::positional(entry.handler),
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
                self.call_simple_error_handler(&invocation, call)?;
            } else {
                return Err(
                    self.condition_jump(entry, (Sexp::Nil, call.clone(), Some(message.into())))?
                );
            }
        }
        self.conditions.message = message.into();
        Ok(())
    }

    pub(crate) fn signal_condition(
        &mut self,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [condition, _, call] = args else {
            return Err(EvalError::Arity(
                "3 arguments required by '.signalCondition'".into(),
            ));
        };
        let classes = match crate::list_attributes::attributes(&condition.value)? {
            Sexp::Nil => Vec::new(),
            Sexp::List(attributes) => attributes
                .into_iter()
                .find_map(
                    |attribute| match (attribute.name.as_deref(), attribute.value) {
                        (
                            Some("class"),
                            Sexp::String(classes)
                            | Sexp::AttributedCharacter {
                                values: classes, ..
                            },
                        ) => Some(classes),
                        _ => None,
                    },
                )
                .unwrap_or_default(),
            _ => return Err(EvalError::Type("invalid attributes representation".into())),
        };
        let saved = self.conditions.head.clone();
        while let Some(entry) = self.conditions.select(&classes)? {
            if entry.calling()? {
                let invocation = Sexp::Call {
                    function: Box::new(entry.handler),
                    arguments: vec![CallArgument::positional(condition.value.clone())],
                };
                self.condition_eval_in(&invocation, self.global.clone())?;
            } else {
                return Err(self
                    .condition_jump(entry, (condition.value.clone(), call.value.clone(), None))?);
            }
        }
        self.conditions.head = saved;
        Ok(Sexp::Nil)
    }

    fn condition_jump(
        &self,
        entry: Entry,
        payload: (Sexp, Sexp, Option<String>),
    ) -> Result<EvalError, EvalError> {
        let Sexp::Environment(target) = entry.target else {
            return Err(EvalError::Control(
                "no function to return from, jumping to top level".into(),
            ));
        };
        let target = self.control.condition_target(&target)?;
        let token = self.conditions.token.clone();
        entry
            .result
            .replace_shared(&[(0, payload.0), (1, payload.1), (2, entry.handler)])?;
        Ok(EvalError::Condition(ConditionJump {
            target,
            result: entry.result,
            token,
            message: payload.2,
        }))
    }

    fn condition_eval_in(
        &mut self,
        expression: &Sexp,
        environment: Environment,
    ) -> Result<EvalFlow, EvalError> {
        let previous = std::mem::replace(&mut self.current, environment);
        let result = self.eval_flow(expression);
        let result = self.signal_fresh_result(result);
        self.current = previous;
        result
    }
}
