use std::collections::HashSet;

use super::values::{same_snapshot, StoredValue};
use super::{Environment, EnvironmentRole, GraphError};
use crate::Sexp;

impl Environment {
    pub fn base_bytecode_ready(&self) -> Result<bool, GraphError> {
        let base = self.session_root(EnvironmentRole::Base)?;
        let store = self.0.heap()?.lock();
        Ok(store.frame(base.0.id())?.bytecode_ready)
    }

    pub fn finalize_base_bytecode(
        &self,
        replacements: &[(String, Sexp, Sexp)],
    ) -> Result<(), GraphError> {
        let base = self.session_root(EnvironmentRole::Base)?;
        let heap = self.0.heap()?;
        let mut converted = Vec::new();
        let mut names = HashSet::new();
        for (name, previous, replacement) in replacements {
            if !names.insert(name) {
                return Err(GraphError::InvalidBytecode);
            }
            validate_replacement(previous, replacement)?;
            converted.push((
                name.clone(),
                StoredValue::new(previous, heap)?,
                StoredValue::new(replacement, heap)?,
            ));
        }
        let mut store = heap.lock();
        let frame = store.frame_mut(base.0.id())?;
        if frame.bytecode_ready {
            return Ok(());
        }
        if !frame.initialized {
            return Err(GraphError::WrongKind);
        }
        for (name, previous, _) in &converted {
            let current = frame.bindings.get(name).ok_or(GraphError::WrongKind)?;
            if frame.assignment_pending.contains(name)
                || !same_snapshot(&current.value, &previous.value)?
            {
                return Err(GraphError::WrongKind);
            }
        }
        for (name, _, replacement) in &converted {
            frame
                .bindings
                .insert(name.clone(), replacement.copy_edges());
        }
        frame.bytecode_ready = true;
        for (_, previous, replacement) in &converted {
            store.replace_value_references(previous, replacement);
        }
        store.collect();
        Ok(())
    }
}

fn validate_replacement(previous: &Sexp, replacement: &Sexp) -> Result<(), GraphError> {
    let (
        Sexp::Closure {
            parameters: ap,
            body: ab,
            environment: ae,
            attributes: aa,
        },
        Sexp::Closure {
            parameters: bp,
            body: bb,
            environment: be,
            attributes: ba,
        },
    ) = (previous, replacement)
    else {
        return Err(GraphError::WrongKind);
    };
    let Sexp::Bytecode(code) = bb.as_ref() else {
        return Err(GraphError::WrongKind);
    };
    let source = crate::canonical_actual_call(ab).map_err(|_| GraphError::InvalidBytecode)?;
    let original = Sexp::Closure {
        parameters: ap.clone(),
        body: Box::new(source),
        environment: ae.clone(),
        attributes: aa.clone(),
    };
    let uncompiled = Sexp::Closure {
        parameters: bp.clone(),
        body: Box::new(code.source()?),
        environment: be.clone(),
        attributes: ba.clone(),
    };
    if !same_snapshot(&original, &uncompiled)? {
        return Err(GraphError::InvalidBytecode);
    }
    Ok(())
}
