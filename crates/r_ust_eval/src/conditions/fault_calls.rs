use crate::{dynamic_context::ContextKind, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn source_current_call(&self) -> Sexp {
        self.control.source_current_call()
    }

    pub(crate) fn source_lexical_call(&self) -> Sexp {
        self.control
            .contexts
            .iter()
            .rev()
            .find(|entry| {
                matches!(entry.kind, ContextKind::Return(_)) && entry.execution == self.current
            })
            .map_or(Sexp::Nil, |entry| entry.call.clone())
    }

    pub(crate) fn source_stop_call(&self, requested: bool) -> Sexp {
        if !requested {
            return Sexp::Nil;
        }
        self.control
            .contexts
            .iter()
            .rev()
            .skip(1)
            .find(|entry| matches!(entry.kind, ContextKind::Return(_)))
            .map_or(Sexp::Nil, |entry| entry.call.clone())
    }
}

impl crate::control::ControlState {
    pub(crate) fn source_current_call(&self) -> Sexp {
        self.contexts
            .last()
            .map_or(Sexp::Nil, |entry| entry.call.clone())
    }
}
