use super::{typed, Comparison, EvalError, ListElement};

impl Comparison {
    pub(super) fn attributes(
        &mut self,
        left: &[ListElement],
        right: &[ListElement],
    ) -> Result<bool, EvalError> {
        let left = typed::expanded_attributes(left)?;
        let right = typed::expanded_attributes(right)?;
        if self.ordered_attributes {
            return self.elements(&left, &right);
        }
        if left.len() != right.len() {
            return Ok(false);
        }
        for a in &left {
            let Some(b) = right.iter().find(|b| a.name == b.name) else {
                return Ok(false);
            };
            if !self.compare(&a.value, &b.value)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn closure_attributes(
        &mut self,
        left: &[ListElement],
        right: &[ListElement],
    ) -> Result<bool, EvalError> {
        if self.use_srcref {
            return self.attributes(left, right);
        }
        let strip = |attributes: &[ListElement]| {
            attributes
                .iter()
                .filter(|attribute| attribute.name.as_deref() != Some("srcref"))
                .cloned()
                .collect::<Vec<_>>()
        };
        self.attributes(&strip(left), &strip(right))
    }

    pub(super) fn elements(
        &mut self,
        left: &[ListElement],
        right: &[ListElement],
    ) -> Result<bool, EvalError> {
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            if a.name != b.name || !self.compare(&a.value, &b.value)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn parameters(
        &mut self,
        left: &[r_ust_sexp::FunctionParameter],
        right: &[r_ust_sexp::FunctionParameter],
    ) -> Result<bool, EvalError> {
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            if a.name != b.name {
                return Ok(false);
            }
            match (&a.default, &b.default) {
                (Some(a), Some(b)) if !self.compare(a, b)? => return Ok(false),
                (Some(_), None) | (None, Some(_)) => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }
}
