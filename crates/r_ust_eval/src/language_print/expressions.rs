use super::{LanguagePrintError, Writer};
use crate::{CallArgument, ListElement, Sexp};

impl Writer<'_> {
    pub(super) fn expression(
        &mut self,
        elements: &[ListElement],
        names_present: bool,
    ) -> Result<(), LanguagePrintError> {
        let arguments = elements
            .iter()
            .map(|element| CallArgument {
                name: if names_present {
                    match element.name.as_deref() {
                        Some("") => None,
                        Some(name) => Some(name.into()),
                        None => Some("NA".into()),
                    }
                } else {
                    None
                },
                value: element.value.clone(),
            })
            .collect::<Vec<_>>();
        self.call(&Sexp::Symbol("expression".into()), &arguments, (false, 0))
    }
}
