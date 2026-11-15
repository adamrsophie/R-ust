// Adapted from R 4.6.0 src/main/eval.c and context.c.
// Copyright (C) 1998-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::dynamic_context::ContextKind;
use crate::{Environment, EvalError, RuntimeState, Sexp};
use r_ust_sexp::EnvironmentRole;

impl RuntimeState {
    pub(super) fn eval_environment(
        &mut self,
        value: &Sexp,
        enclosure: &Sexp,
    ) -> Result<Environment, EvalError> {
        let parent = match enclosure {
            Sexp::Nil => self.global.session_root(EnvironmentRole::Base)?,
            Sexp::Environment(environment) => environment.clone(),
            _ => return Err(self.eval_environment_error("enclos", &Sexp::Nil)),
        };
        match value {
            Sexp::Nil => Ok(parent),
            Sexp::Environment(environment) => Ok(environment.clone()),
            Sexp::Integer(_) | Sexp::Real(_) | Sexp::Factor { .. } => {
                self.eval_numeric_environment(value)
            }
            Sexp::Matrix { data, .. } => self.eval_environment(data, enclosure),
            value
                if matches!(
                    value.sexptype(),
                    r_ust_sexp::SexpType::List | r_ust_sexp::SexpType::Vector
                ) =>
            {
                self.eval_list_environment(value, &parent)
            }
            _ => Err(self.eval_environment_error("envir", value)),
        }
    }

    fn eval_numeric_environment(&mut self, value: &Sexp) -> Result<Environment, EvalError> {
        if crate::value_len(value)? != 1 {
            return Err(crate::SourceFault::domain(
                "numeric 'envir' arg not of length one".into(),
                &self.source_current_call(),
            ));
        }
        let number = self
            .scalar_integer_with_warning(value)?
            .ok_or_else(|| self.eval_environment_error("envir", value))?;
        if number == 0 {
            return Ok(self.global.clone());
        }
        let frames = self
            .control
            .contexts
            .iter()
            .filter(|context| matches!(context.kind, ContextKind::Return(_)));
        let depth = frames.clone().count();
        let distance = if number > 0 {
            usize::try_from(number)
                .ok()
                .and_then(|number| depth.checked_sub(number))
        } else {
            usize::try_from(number.unsigned_abs()).ok()
        };
        let distance = distance
            .filter(|distance| *distance <= depth)
            .ok_or_else(|| {
                crate::SourceFault::domain(
                    "not that many frames on the stack".into(),
                    &self.source_current_call(),
                )
            })?;
        Ok(frames
            .rev()
            .nth(distance)
            .map_or_else(|| self.global.clone(), |context| context.execution.clone()))
    }

    fn eval_environment_error(&self, name: &str, value: &Sexp) -> EvalError {
        crate::SourceFault::domain(
            format!(
                "invalid '{name}' argument of type '{}'",
                crate::typeof_value(value)
            ),
            &self.source_current_call(),
        )
    }

    fn eval_list_environment(
        &self,
        value: &Sexp,
        parent: &Environment,
    ) -> Result<Environment, EvalError> {
        let frame = parent.child()?;
        let value = crate::ordinary_values::read(value)?;
        if let Some(view) = value.ordinary_list() {
            if view.names_present {
                install(&frame, view.elements, true)?;
            }
        } else {
            match value.as_ref() {
                Sexp::PairList(elements) => install(&frame, elements.as_slice(), false)?,
                Sexp::DataFrame { columns, .. } => install(&frame, columns, true)?,
                Sexp::Vector(_) => {}
                _ => return Err(self.eval_environment_error("envir", &value)),
            }
        }
        Ok(frame)
    }
}

fn install(
    frame: &Environment,
    elements: &[crate::ListElement],
    named: bool,
) -> Result<(), EvalError> {
    for element in elements {
        let name = match element.name.as_deref() {
            Some(name) => name,
            None if named => "NA",
            None => continue,
        };
        if !name.is_empty() && !frame.contains_local(name)? {
            frame.insert(name.into(), &element.value)?;
        }
    }
    Ok(())
}
