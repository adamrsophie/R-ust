use super::{bad_stack, entry::Entry, Conditions, PairListObject};
use crate::{EvalError, Sexp};
use r_ust_sexp::Environment;

impl Conditions {
    pub(super) fn reset(&mut self, owner: &Environment, value: &Sexp) -> Result<(), EvalError> {
        owner.validate_import(value)?;
        let next = match value {
            Sexp::Nil => None,
            Sexp::PairListObject(head) => Some(head.clone()),
            _ => return Err(bad_stack()),
        };
        let mut cursor = next.clone();
        while let Some(node) = cursor {
            Entry::read(node.car()?.value)?;
            cursor = node.cdr()?;
        }
        self.head = next;
        Ok(())
    }

    pub(crate) fn unwind(&mut self, target: &Option<PairListObject>) -> Result<(), EvalError> {
        let mut cursor = self.head.clone();
        while !same_head(&cursor, target) {
            let Some(node) = cursor else {
                self.head = target.clone();
                return Ok(());
            };
            cursor = node.cdr()?;
        }
        while !same_head(&self.head, target) {
            let Some(node) = self.head.clone() else {
                return Err(bad_stack());
            };
            let entry = Entry::read(node.car()?.value)?;
            self.head = node.cdr()?;
            entry
                .object
                .replace_shared(&[(1, Sexp::Nil), (3, Sexp::Nil)])?;
        }
        Ok(())
    }
}

fn same_head(left: &Option<PairListObject>, right: &Option<PairListObject>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.same_object(right),
        (None, Some(_)) | (Some(_), None) => false,
    }
}
