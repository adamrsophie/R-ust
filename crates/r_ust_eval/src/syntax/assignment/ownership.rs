use super::{EvalError, Place, Sexp};

pub(super) struct Ownership {
    root: Option<Sexp>,
    references: Vec<Option<usize>>,
}

impl Place {
    pub(super) fn ownership(&self) -> Result<Ownership, EvalError> {
        let references = self
            .steps
            .iter()
            .map(|step| {
                step.reusable
                    .as_ref()
                    .map(|object| object.semantic_reference_count())
                    .transpose()
            })
            .collect::<Result<_, _>>()?;
        Ok(Ownership {
            root: self.frame.get_local(&self.root)?,
            references,
        })
    }

    pub(super) fn check_ownership(&mut self, before: Ownership) -> Result<(), EvalError> {
        let root_changed = self.frame.get_local(&self.root)? != before.root;
        let last = self.steps.len().saturating_sub(1);
        for (index, (step, count)) in self.steps.iter_mut().zip(before.references).enumerate() {
            let Some(object) = &step.reusable else {
                continue;
            };
            let shared = match count {
                Some(count) => object.semantic_reference_count()? > count,
                None => false,
            };
            if shared || (index == 0 && root_changed) {
                step.reusable = None;
                if index == last {
                    self.unshared = false;
                }
            }
        }
        Ok(())
    }
}
