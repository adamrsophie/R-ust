use crate::{EvalError, EvaluatedArgument, ListElement, RuntimeState, Sexp, SourceFault};

impl RuntimeState {
    pub(super) fn internal_ngettext(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let number = self
            .scalar_integer_with_warning(&arguments[0].value)?
            .filter(|number| *number >= 0)
            .ok_or_else(|| SourceFault::domain("invalid 'n' argument".into(), &call))?;
        for (index, name) in [(1, "msg1"), (2, "msg2")] {
            if arguments[index]
                .value
                .character_values()
                .is_none_or(|values| values.len() != 1)
            {
                return Err(SourceFault::domain(
                    format!("'{name}' must be a character string"),
                    &call,
                ));
            }
        }
        let domain = &arguments[3].value;
        let disabled = matches!(domain, Sexp::Logical(values) if values.as_slice()==[None])
            || domain
                .character_values()
                .is_some_and(|v| matches!(v.first(), Some(Some(value)) if value.is_empty()));
        if !matches!(domain, Sexp::Nil) && !disabled && domain.character_values().is_none() {
            return Err(SourceFault::domain("invalid 'domain' value".into(), &call));
        }
        if domain
            .character_values()
            .is_some_and(<[Option<String>]>::is_empty)
        {
            return Err(SourceFault::domain("invalid 'domain' value".into(), &call));
        }
        if !disabled
            && !matches!(
                crate::text_locale::current()?,
                crate::text_locale::Encoding::C
            )
        {
            return Err(EvalError::NotImplemented(
                "ngettext translation catalogs outside the C locale",
            ));
        }
        Ok(arguments[if number == 1 { 1 } else { 2 }].value.clone())
    }

    pub(super) fn internal_l10n_info(&self) -> Result<Sexp, EvalError> {
        let utf8 = matches!(
            crate::text_locale::current()?,
            crate::text_locale::Encoding::Utf8
        );
        let codeset = if utf8 {
            "UTF-8"
        } else if cfg!(target_os = "macos") {
            "US-ASCII"
        } else if cfg!(target_os = "linux") {
            "ANSI_X3.4-1968"
        } else {
            return Err(EvalError::NotImplemented("l10n_info platform codeset"));
        };
        Ok(Sexp::list_with_metadata(
            vec![
                entry("MBCS", Sexp::Logical(vec![Some(utf8)])),
                entry("UTF-8", Sexp::Logical(vec![Some(utf8)])),
                entry("Latin-1", Sexp::Logical(vec![Some(false)])),
                entry("codeset", Sexp::String(vec![Some(codeset.into())])),
            ],
            true,
            None,
        ))
    }
}

fn entry(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
