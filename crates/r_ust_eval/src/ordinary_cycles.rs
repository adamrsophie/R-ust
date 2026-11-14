use crate::{ordinary_values, EvalError, Sexp};

pub(super) fn check(value: &Sexp, boundary: &'static str) -> Result<(), EvalError> {
    Walker {
        path: Vec::new(),
        boundary,
    }
    .visit(value)
}

struct Walker {
    path: Vec<Sexp>,
    boundary: &'static str,
}

impl Walker {
    fn visit(&mut self, value: &Sexp) -> Result<(), EvalError> {
        let shared = matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_));
        if shared {
            if self.path.contains(value) {
                return Err(EvalError::NotImplemented(self.boundary));
            }
            self.path.push(value.clone());
        }
        let projection = ordinary_values::read(value)?;
        self.projected(projection.as_ref())?;
        if shared {
            self.path.pop();
        }
        Ok(())
    }

    fn projected(&mut self, value: &Sexp) -> Result<(), EvalError> {
        if let Sexp::ClassedList { class, .. } = value {
            self.visit(class)?;
        }
        match value {
            Sexp::AttributedCharacter {
                attributes: values, ..
            }
            | Sexp::List(values)
            | Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::DataFrame {
                columns: values, ..
            }
            | Sexp::Dots(values) => {
                for slot in values {
                    self.visit(&slot.value)?;
                }
            }
            Sexp::PairList(values) => {
                for slot in values.as_slice() {
                    self.visit(&slot.value)?;
                }
            }
            Sexp::Vector(values) => {
                for value in values {
                    self.visit(value)?;
                }
            }
            Sexp::Matrix { data, .. } => self.visit(data)?,
            _ => {}
        }
        Ok(())
    }
}
