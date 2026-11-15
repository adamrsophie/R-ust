use super::{
    profile::{ATTRIBUTES, NICE_NAMES},
    LanguagePrintError, Writer,
};
use crate::{ListElement, Sexp};

pub(super) struct Attributes {
    pub inline_names: bool,
    pub remaining: Vec<ListElement>,
}

impl Writer<'_> {
    pub(super) fn begin_attributes(
        &mut self,
        attributes: &[ListElement],
        atomic: bool,
        force_names: bool,
    ) -> Attributes {
        let names = attributes
            .iter()
            .find(|a| a.name.as_deref() == Some("names"))
            .and_then(|a| a.value.character_values());
        let show = self.options() & ATTRIBUTES != 0;
        let nice = self.options() & NICE_NAMES != 0;
        let usable = names.is_some_and(|values| {
            values.iter().all(|name| {
                name.as_deref()
                    .is_some_and(|name| !atomic || !matches!(name, "recursive" | "use.names"))
            }) && values
                .iter()
                .any(|name| name.as_deref().is_some_and(|name| !name.is_empty()))
        });
        let inline_names = names.is_some() && nice && !force_names && (usable || !show);
        let remaining = if show {
            attributes
                .iter()
                .filter(|attribute| {
                    attribute.name.as_deref() != Some("srcref")
                        && !(inline_names && attribute.name.as_deref() == Some("names"))
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if !remaining.is_empty() {
            self.emit("structure(");
        }
        Attributes {
            inline_names,
            remaining,
        }
    }

    pub(super) fn end_attributes(
        &mut self,
        attributes: Attributes,
    ) -> Result<(), LanguagePrintError> {
        if attributes.remaining.is_empty() {
            return Ok(());
        }
        for attribute in attributes.remaining {
            if !self.active() {
                break;
            }
            let name = attribute
                .name
                .as_deref()
                .ok_or(LanguagePrintError::MalformedCall)?;
            self.emit(", ");
            self.emit(&if super::names::valid_name(name) {
                name.into()
            } else {
                self.text(name, '"')?
            });
            self.emit(" = ");
            self.fnarg = true;
            self.write(&attribute.value)?;
        }
        self.emit(")");
        Ok(())
    }

    pub(super) fn list_attributes(
        &self,
        view: r_ust_sexp::OrdinaryListView<'_>,
    ) -> Vec<ListElement> {
        let mut result = Vec::new();
        if let Some(class) = view.class.filter(|_| view.class_first) {
            result.push(attribute("class", class.clone()));
        }
        if view.names_present {
            result.push(attribute(
                "names",
                Sexp::String(view.elements.iter().map(|e| e.name.clone()).collect()),
            ));
        }
        if let Some(class) = view.class.filter(|_| !view.class_first) {
            result.push(attribute("class", class.clone()));
        }
        result
    }
}

pub(super) fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
