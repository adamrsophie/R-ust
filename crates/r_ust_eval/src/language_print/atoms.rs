use super::{names, LanguagePrintError, Writer};
use crate::Sexp;
impl Writer<'_> {
    pub(super) fn atom(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        let text = match value {
            Sexp::Nil => "NULL".into(),
            Sexp::MissingValue => String::new(),
            Sexp::Symbol(text) => names::name(text)?,
            Sexp::Logical(values) => match values.as_slice() {
                [Some(true)] => "TRUE".into(),
                [Some(false)] => "FALSE".into(),
                [None] => "NA".into(),
                [] | [_, _, ..] => return Err(LanguagePrintError::NonScalar),
            },
            Sexp::Integer(values) => match values.as_slice() {
                // R reserves INT_MIN for NA; Option::None represents it here.
                [Some(value)] if *value == i32::MIN => {
                    return Err(LanguagePrintError::UnsupportedNode)
                }
                [Some(value)] if self.strip_integer_suffix => value.to_string(),
                [Some(value)] => format!("{value}L"),
                [None] if self.strip_integer_suffix => "NA".into(),
                [None] => "NA_integer_".into(),
                [] | [_, _, ..] => return Err(LanguagePrintError::NonScalar),
            },
            Sexp::Real(values) => match values.as_slice() {
                [value] => crate::real_literal::format_real(*value, true),
                [] | [_, _, ..] => return Err(LanguagePrintError::NonScalar),
            },
            Sexp::String(values) => match values.as_slice() {
                [Some(value)] => {
                    names::check_text(value)?;
                    crate::encode_string::encode(value, Some('"'))
                }
                [None] => "NA_character_".into(),
                [] | [_, _, ..] => return Err(LanguagePrintError::NonScalar),
            },
            _ => return Err(LanguagePrintError::UnsupportedNode),
        };
        self.emit(&text);
        Ok(())
    }
}
