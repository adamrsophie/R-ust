use super::{
    profile::{DELAY, QUOTE, SIMPLE},
    LanguagePrintError, Writer,
};
use crate::{EvalError, Sexp};

impl Writer<'_> {
    pub(super) fn deparse_node(
        &mut self,
        value: &Sexp,
        context: (bool, u8),
    ) -> Result<(), LanguagePrintError> {
        if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
            if self.ancestry.iter().any(|prior| match (prior, value) {
                (Sexp::ListObject(left), Sexp::ListObject(right)) => left.same_object(right),
                (Sexp::PairListObject(left), Sexp::PairListObject(right)) => {
                    left.same_object(right)
                }
                _ => false,
            }) {
                return Err(EvalError::NotImplemented("deparse cyclic graph value").into());
            }
            self.ancestry.push(value.clone());
            let projection =
                crate::ordinary_values::read(value).map_err(LanguagePrintError::from)?;
            let result = self.deparse_node(projection.as_ref(), context);
            self.ancestry.pop();
            return result;
        }
        if let Some(view) = value.ordinary_list() {
            let attributes = self.list_attributes(view);
            return self.deparse_list(view.elements, &attributes, false);
        }
        match value {
            Sexp::Nil => self.emit("NULL"),
            Sexp::MissingValue | Sexp::MissingArg(_) => {}
            Sexp::Symbol(name) => {
                let quote = self.options() & QUOTE != 0 && !name.is_empty();
                if quote {
                    self.emit("quote(");
                }
                self.emit(&self.symbol(name)?);
                if quote {
                    self.emit(")");
                }
            }
            Sexp::Char(value) => self.emit(value.as_deref().unwrap_or("NA")),
            Sexp::Call {
                function,
                arguments,
            } => {
                let options = self.options();
                let quote = options & QUOTE != 0
                    && !matches!(function.as_ref(), Sexp::Symbol(name) if name == "~");
                if quote {
                    self.emit("quote(");
                    self.set_options(options & SIMPLE);
                } else {
                    self.set_options(options & !QUOTE);
                }
                let result = self.call(function, arguments, context);
                self.set_options(options);
                result?;
                if quote {
                    self.emit(")");
                }
            }
            Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) => {
                self.deparse_atomic(value, &[])?;
            }
            Sexp::AttributedCharacter { values, attributes } => {
                self.deparse_atomic(&Sexp::String(values.clone()), attributes)?;
            }
            Sexp::Primitive(id) => self.emit(&format!(".Primitive(\"{}\")", id.name())),
            Sexp::Environment(_) => {
                self.incomplete = true;
                self.emit("<environment>");
            }
            Sexp::Expression {
                elements,
                names_present,
            } => {
                let attributes = if *names_present {
                    vec![super::deparse_attributes::attribute(
                        "names",
                        Sexp::String(elements.iter().map(|e| e.name.clone()).collect()),
                    )]
                } else {
                    Vec::new()
                };
                self.deparse_list(elements, &attributes, true)?;
            }
            Sexp::PairList(elements) => self.deparse_pairlist(elements.as_slice())?,
            Sexp::Vector(values) => {
                let elements = values
                    .iter()
                    .map(|value| crate::ListElement {
                        name: None,
                        value: value.clone(),
                    })
                    .collect::<Vec<_>>();
                self.deparse_list(&elements, &[], false)?;
            }
            Sexp::Closure { .. } => self.deparse_closure(value)?,
            Sexp::Factor { .. } | Sexp::Matrix { .. } | Sexp::DataFrame { .. } => {
                self.deparse_special(value)?
            }
            Sexp::Promise { state } => {
                if self.options() & DELAY != 0 {
                    self.incomplete = true;
                    let expression = state
                        .expression()
                        .map_err(|e| LanguagePrintError::from(EvalError::from(e)))?;
                    let options = self.options();
                    self.emit("<promise: ");
                    self.set_options(options & !QUOTE);
                    let result = self.write(&expression);
                    self.set_options(options);
                    result?;
                    self.emit(">");
                } else {
                    let runtime = self
                        .runtime
                        .as_deref_mut()
                        .ok_or(LanguagePrintError::UnsupportedNode)?;
                    let value = runtime
                        .force_value(value.clone())
                        .map_err(LanguagePrintError::from)?;
                    self.write(&value)?;
                }
            }
            _ => {
                let canonical = r_ust_sexp::canonical_actual_call(value)
                    .map_err(|_| LanguagePrintError::UnsupportedNode)?;
                if &canonical == value {
                    return Err(LanguagePrintError::UnsupportedNode);
                }
                self.fnarg = context.0;
                self.left = context.1;
                self.write(&canonical)?;
            }
        }
        Ok(())
    }
}
