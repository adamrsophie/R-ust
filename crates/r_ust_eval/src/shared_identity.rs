use crate::{ordinary_values, EvalError, ListElement, Sexp};
use r_ust_sexp::bytecode_source;

#[derive(Default)]
pub(super) struct Comparison {
    visited: Vec<(Sexp, Sexp)>,
    pub(super) use_srcref: bool,
    pub(super) use_bytecode: bool,
    pub(super) ordered_attributes: bool,
}

impl Comparison {
    pub(super) fn compare(&mut self, left: &Sexp, right: &Sexp) -> Result<bool, EvalError> {
        if matches!(
            left,
            Sexp::ListObject(_) | Sexp::PairListObject(_) | Sexp::Bytecode(_)
        ) || matches!(
            right,
            Sexp::ListObject(_) | Sexp::PairListObject(_) | Sexp::Bytecode(_)
        ) {
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
        if let (Some(a), Some(b)) = (left.ordinary_list(), right.ordinary_list()) {
            if a.names_present != b.names_present
                || (self.ordered_attributes
                    && a.names_present
                    && a.class.is_some()
                    && a.class_first != b.class_first)
            {
                return Ok(false);
            }
            let classes_equal = match (a.class, b.class) {
                (None, None) => true,
                (Some(a), Some(b)) => self.compare(a, b)?,
                _ => false,
            };
            return Ok(classes_equal && self.elements(a.elements, b.elements)?);
        }
        Ok(match (left, right) {
            (Sexp::Bytecode(a), Sexp::Bytecode(b)) => {
                let a = a.parts()?;
                let b = b.parts()?;
                a.code == b.code
                    && self.compare(&a.expression, &b.expression)?
                    && self.compare(&Sexp::Vector(a.constants), &Sexp::Vector(b.constants))?
            }
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
            (
                Sexp::Expression {
                    elements: a,
                    names_present: an,
                },
                Sexp::Expression {
                    elements: b,
                    names_present: bn,
                },
            ) => an == bn && self.elements(a, b)?,
            (Sexp::Dots(a), Sexp::Dots(b)) => self.elements(a, b)?,
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
                    attributes: aa,
                },
                Sexp::Closure {
                    parameters: bp,
                    body: b,
                    environment: be,
                    attributes: ba,
                },
            ) => {
                ae == be
                    && self.parameters(ap, bp)?
                    && if self.use_bytecode {
                        self.compare(a, b)?
                    } else {
                        self.compare(&bytecode_source(a)?, &bytecode_source(b)?)?
                    }
                    && self.closure_attributes(aa, ba)?
            }
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
        if self.ordered_attributes {
            return self.elements(left, right);
        }
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

    fn closure_attributes(
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
