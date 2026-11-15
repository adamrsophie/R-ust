use super::{profile::SIMPLE, LanguagePrintError, Writer};
use crate::{ListElement, Sexp};

impl Writer<'_> {
    pub(super) fn deparse_list(
        &mut self,
        elements: &[ListElement],
        attributes: &[ListElement],
        expression: bool,
    ) -> Result<(), LanguagePrintError> {
        let options = self.options();
        let plan = self.begin_attributes(attributes, false, false);
        self.emit(if expression { "expression(" } else { "list(" });
        if expression {
            self.set_options(options & SIMPLE);
        }
        let mut continued = false;
        for (index, element) in elements.iter().enumerate() {
            if !self.active() {
                break;
            }
            if index > 0 {
                self.emit(", ");
            }
            self.linebreak(&mut continued);
            if !self.active() {
                break;
            }
            if plan.inline_names {
                let name = element.name.as_deref().unwrap_or("NA");
                if !name.is_empty() {
                    self.emit(&self.tag_name(name)?);
                    self.emit(" = ");
                }
            }
            self.write(&element.value)?;
        }
        if continued {
            self.indent -= 1;
        }
        self.emit(")");
        self.set_options(options);
        self.end_attributes(plan)
    }

    pub(super) fn deparse_pairlist(
        &mut self,
        elements: &[ListElement],
    ) -> Result<(), LanguagePrintError> {
        let missing = elements
            .iter()
            .any(|e| matches!(e.value, Sexp::MissingValue));
        self.emit(if missing {
            "as.pairlist(alist("
        } else {
            "pairlist("
        });
        self.inlist += 1;
        for (index, element) in elements.iter().enumerate() {
            if !self.active() {
                break;
            }
            if index > 0 {
                self.emit(", ");
            }
            if let Some(name) = &element.name {
                let options = self.options();
                self.set_options(0);
                self.emit(&self.symbol(name)?);
                self.set_options(options);
                self.emit(" = ");
            }
            self.write(&element.value)?;
        }
        self.emit(if missing { "))" } else { ")" });
        self.inlist -= 1;
        Ok(())
    }
}
