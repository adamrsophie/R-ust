// Adapted from R 4.6.0 src/main/eval.c: do_eval.
// Copyright (C) 1998-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

mod environment;

use crate::dynamic_context::ReturnContext;
use crate::{CallArgument, EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp};
use r_ust_sexp::PrimitiveId;

impl RuntimeState {
    pub(crate) fn internal_eval(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [expression, environment, enclosure] = arguments else {
            return Err(crate::SourceFault::arity(
                format!(
                    "{} arguments passed to 'eval' which requires 3",
                    arguments.len()
                ),
                &self.source_current_call(),
            ));
        };
        let selected = self.eval_environment(&environment.value, &enclosure.value)?;
        let expression = &expression.value;
        if matches!(expression, Sexp::Promise { .. }) {
            let EvalFlow::Value(value) = self.eval_flow(expression)?;
            return Ok(value);
        }
        if !matches!(
            expression,
            Sexp::Expression { .. } | Sexp::Symbol(_) | Sexp::MissingValue
        ) && expression.sexptype() != r_ust_sexp::SexpType::Language
        {
            return Ok(expression.clone());
        }
        let entry = ReturnContext {
            execution: selected,
            sysparent: self.current.clone(),
            callable: Sexp::Primitive(PrimitiveId::InternalEval),
            arguments: arguments
                .iter()
                .map(|argument| CallArgument {
                    name: argument.name.clone(),
                    value: argument.value.clone(),
                })
                .collect(),
            call: self.source_current_call(),
            generic: false,
        };
        let EvalFlow::Value(value) =
            self.with_return_context(entry, |runtime| match expression {
                Sexp::Expression { elements, .. } => {
                    let mut value = Sexp::Nil;
                    for element in elements {
                        let EvalFlow::Value(next) = runtime.eval_eval_element(&element.value)?;
                        value = next;
                    }
                    Ok(EvalFlow::Value(value))
                }
                value => runtime.eval_eval_element(value),
            })?;
        Ok(value)
    }

    fn eval_eval_element(&mut self, expression: &Sexp) -> Result<EvalFlow, EvalError> {
        if matches!(expression, Sexp::MissingValue) {
            return Err(self.evaluated_missing(""));
        }
        self.eval_flow(expression)
    }
}
