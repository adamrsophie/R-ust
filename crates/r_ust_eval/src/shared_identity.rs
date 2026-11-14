use crate::{ordinary_values, EvalError, ListElement, Sexp};

#[derive(Default)]
pub(super) struct Comparison {
    visited: Vec<(Sexp, Sexp)>,
}

impl Comparison {
    pub(super) fn compare(&mut self, left: &Sexp, right: &Sexp) -> Result<bool, EvalError> {
        if matches!(left, Sexp::ListObject(_) | Sexp::PairListObject(_))
            || matches!(right, Sexp::ListObject(_) | Sexp::PairListObject(_))
        {
            if self.visited.iter().any(|(a, b)| a == left && b == right) {
                return Ok(true);
            }
            self.visited.push((left.clone(), right.clone()));
        }
        let left = ordinary_values::read(left)?;
        let right = ordinary_values::read(right)?;
        self.projected(left.as_ref(), right.as_ref())
    }

    fn projected(&mut self, left: &Sexp, right: &Sexp) -> Result<bool, EvalError> {
        Ok(match (left, right) {
            (
                Sexp::String(a),
                Sexp::AttributedCharacter {
                    values: b,
                    attributes,
                },
            )
            | (
                Sexp::AttributedCharacter {
                    values: a,
                    attributes,
                },
                Sexp::String(b),
            ) => a == b && attributes.is_empty(),
            (
                Sexp::AttributedCharacter {
                    values: a,
                    attributes: aa,
                },
                Sexp::AttributedCharacter {
                    values: b,
                    attributes: ba,
                },
            ) => a == b && self.attributes(aa, ba)?,
            (Sexp::List(a), Sexp::List(b)) | (Sexp::Dots(a), Sexp::Dots(b)) => {
                self.elements(a, b)?
            }
            (
                Sexp::ClassedList {
                    elements: a,
                    class: ac,
                    ..
                },
                Sexp::ClassedList {
                    elements: b,
                    class: bc,
                    ..
                },
            ) => self.compare(ac, bc)? && self.elements(a, b)?,
            (Sexp::PairList(a), Sexp::PairList(b)) => self.elements(a.as_slice(), b.as_slice())?,
            (Sexp::Vector(a), Sexp::Vector(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (a, b) in a.iter().zip(b) {
                    if !self.compare(a, b)? {
                        return Ok(false);
                    }
                }
                true
            }
            (
                Sexp::DataFrame {
                    columns: a,
                    nrow: ar,
                },
                Sexp::DataFrame {
                    columns: b,
                    nrow: br,
                },
            ) => ar == br && self.elements(a, b)?,
            (
                Sexp::Matrix {
                    data: a,
                    nrow: ar,
                    ncol: ac,
                },
                Sexp::Matrix {
                    data: b,
                    nrow: br,
                    ncol: bc,
                },
            ) => ar == br && ac == bc && self.compare(a, b)?,
            (
                Sexp::Call {
                    function: a,
                    arguments: aa,
                },
                Sexp::Call {
                    function: b,
                    arguments: ba,
                },
            ) => {
                if aa.len() != ba.len() || !self.compare(a, b)? {
                    return Ok(false);
                }
                for (a, b) in aa.iter().zip(ba) {
                    if a.name != b.name || !self.compare(&a.value, &b.value)? {
                        return Ok(false);
                    }
                }
                true
            }
            (
                Sexp::Closure {
                    parameters: ap,
                    body: a,
                    environment: ae,
                },
                Sexp::Closure {
                    parameters: bp,
                    body: b,
                    environment: be,
                },
            ) => ae == be && self.parameters(ap, bp)? && self.compare(a, b)?,
            (
                Sexp::Function {
                    parameters: ap,
                    body: a,
                },
                Sexp::Function {
                    parameters: bp,
                    body: b,
                },
            ) => self.parameters(ap, bp)? && self.compare(a, b)?,
            (Sexp::Real(a), Sexp::Real(b)) => crate::identity::real_values_identical(a, b),
            _ => left == right,
        })
    }

    fn attributes(
        &mut self,
        left: &[ListElement],
        right: &[ListElement],
    ) -> Result<bool, EvalError> {
        if left.len() != right.len() {
            return Ok(false);
        }
        for a in left {
            let Some(b) = right.iter().find(|b| a.name == b.name) else {
                return Ok(false);
            };
            if !self.compare(&a.value, &b.value)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn elements(&mut self, left: &[ListElement], right: &[ListElement]) -> Result<bool, EvalError> {
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

    fn parameters(
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
