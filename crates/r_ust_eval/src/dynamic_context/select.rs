use super::{Context, ContextKind};
use crate::{control::ControlState, Environment, EvalError, Sexp};

impl ControlState {
    pub(crate) fn selected_contexts(&self) -> &[Context] {
        let Some(current) = self.contexts.last() else {
            return &[];
        };
        self.contexts
            .iter()
            .rposition(|context| {
                matches!(context.kind, ContextKind::Return(_))
                    && context.execution == current.sysparent
            })
            .map_or(&[], |index| &self.contexts[..=index])
    }

    pub(crate) fn selected_depth(&self) -> usize {
        self.selected_contexts()
            .iter()
            .filter(|context| matches!(context.kind, ContextKind::Return(_)))
            .count()
    }

    fn selected_frame(&self, which: i32) -> Result<Option<&Context>, EvalError> {
        let contexts = self.selected_contexts();
        let depth = self.selected_depth();
        let distance = if which > 0 {
            usize::try_from(which)
                .ok()
                .and_then(|value| depth.checked_sub(value))
        } else {
            usize::try_from(which.unsigned_abs()).ok()
        }
        .filter(|distance| *distance <= depth)
        .ok_or_else(|| {
            crate::SourceFault::domain(
                "not that many frames on the stack".into(),
                &self.source_current_call(),
            )
        })?;
        Ok(contexts
            .iter()
            .rev()
            .filter(|context| matches!(context.kind, ContextKind::Return(_)))
            .nth(distance))
    }

    pub(crate) fn frame_environment(
        &self,
        which: i32,
        global: &Environment,
    ) -> Result<Environment, EvalError> {
        if which == 0 {
            return Ok(global.clone());
        }
        Ok(self
            .selected_frame(which)?
            .map_or_else(|| global.clone(), |context| context.execution.clone()))
    }

    pub(crate) fn frame_function(&self, which: i32) -> Result<Sexp, EvalError> {
        Ok(self
            .selected_frame(which)?
            .map_or(Sexp::Nil, |context| context.callable.clone()))
    }

    pub(crate) fn parent_number(&self, generations: i32, global: &Environment) -> usize {
        let contexts: Vec<_> = self
            .selected_contexts()
            .iter()
            .filter(|context| matches!(context.kind, ContextKind::Return(_)))
            .collect();
        let mut number = contexts.len();
        for _ in 0..generations {
            let Some(context) = number.checked_sub(1).and_then(|index| contexts.get(index)) else {
                break;
            };
            if context.sysparent == *global {
                return 0;
            }
            let parent = contexts[..number]
                .iter()
                .position(|candidate| candidate.execution == context.sysparent)
                .map_or_else(
                    || number.saturating_sub(contexts.len() - number),
                    |index| index + 1,
                );
            if parent == number {
                break;
            }
            number = parent;
        }
        number
    }

    pub(crate) fn parent_environment(&self, generations: i32, global: &Environment) -> Environment {
        let Some(mut index) = self.contexts.len().checked_sub(1) else {
            return global.clone();
        };
        for generation in 1..=generations {
            let parent = &self.contexts[index].sysparent;
            let Some(found) = self.contexts[..=index].iter().rposition(|context| {
                matches!(context.kind, ContextKind::Return(_)) && context.execution == *parent
            }) else {
                return global.clone();
            };
            if generation == generations || found == index {
                return self.contexts[found].sysparent.clone();
            }
            index = found;
        }
        global.clone()
    }
}
