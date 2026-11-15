// Adapted from R 4.6.0 src/main/envir.c: do_assign.
// Copyright (C) 1999-2026 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn internal_assign(
        &mut self,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [name, value, environment, inherits] = args else {
            return Err(EvalError::Arity("4 arguments required by 'assign'".into()));
        };
        let call = self.source_current_call();
        let fault = |message: &str| crate::SourceFault::domain(message.into(), &call);
        let (Sexp::String(names) | Sexp::AttributedCharacter { values: names, .. }) = &name.value
        else {
            return Err(fault("invalid first argument"));
        };
        let Some(name) = names.first() else {
            return Err(fault("invalid first argument"));
        };
        if names.len() > 1 {
            self.record_warnings_at(
                &["only the first element is used as variable name".into()],
                Some(&call),
            )?;
        }
        let name = name.as_deref().unwrap_or("NA");
        if name.is_empty() {
            return Err(fault("attempt to use zero-length variable name"));
        }
        let environment = match &environment.value {
            Sexp::Nil => return Err(fault("use of NULL environment is defunct")),
            Sexp::Environment(environment) => environment,
            _ => return Err(fault("invalid 'envir' argument")),
        };
        let inherits = crate::internal::as_logical(&inherits.value)
            .ok_or_else(|| fault("invalid 'inherits' argument"))?;
        let target = if inherits {
            let mut candidate = Some(environment.clone());
            let mut target = self.global.clone();
            while let Some(frame) = candidate {
                if frame.contains_local(name)? {
                    target = frame;
                    break;
                }
                candidate = frame.parent()?;
            }
            target
        } else {
            environment.clone()
        };
        target.insert(name.into(), &value.value).map_err(|error| {
            match crate::environment_members::member_graph_error(error, name) {
                EvalError::Domain(message) => crate::SourceFault::domain(message, &call),
                other => other,
            }
        })?;
        self.visible = false;
        Ok(value.value.clone())
    }
}
