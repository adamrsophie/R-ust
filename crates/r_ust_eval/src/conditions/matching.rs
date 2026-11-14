use super::{entry::Entry, Conditions, PairListObject};
use crate::EvalError;

type Match = (Entry, Option<PairListObject>);

impl Conditions {
    pub(super) fn select(
        &mut self,
        classes: &[Option<String>],
    ) -> Result<Option<Entry>, EvalError> {
        match self.find(classes)? {
            Some((entry, next)) => {
                self.head = next;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    pub(super) fn has_match(&self, classes: &[Option<String>]) -> Result<bool, EvalError> {
        Ok(self.find(classes)?.is_some())
    }

    fn find(&self, classes: &[Option<String>]) -> Result<Option<Match>, EvalError> {
        let mut cursor = self.head.clone();
        while let Some(node) = cursor {
            let entry = Entry::read(node.car()?.value)?;
            cursor = node.cdr()?;
            if classes.iter().any(|class| {
                class.as_deref().unwrap_or("NA") == entry.class.as_deref().unwrap_or("NA")
            }) {
                return Ok(Some((entry, cursor)));
            }
        }
        Ok(None)
    }
}
