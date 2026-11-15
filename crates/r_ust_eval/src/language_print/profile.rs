use super::{LanguagePrintError, Writer};
use crate::{EvalError, RuntimeState, Sexp};

pub(super) const KEEP_INTEGER: u32 = 1;
pub(super) const QUOTE: u32 = 2;
pub(super) const ATTRIBUTES: u32 = 4;
pub(super) const SOURCE: u32 = 8;
pub(crate) const WARN: u32 = 16;
pub(super) const DELAY: u32 = 32;
pub(super) const KEEP_NA: u32 = 64;
pub(super) const S_COMPAT: u32 = 128;
pub(super) const HEX: u32 = 256;
pub(super) const DIGITS17: u32 = 512;
pub(super) const NICE_NAMES: u32 = 1024;
pub(super) const SIMPLE: u32 = !(QUOTE | ATTRIBUTES | DELAY);

#[derive(Clone, Copy)]
pub(crate) struct Profile {
    pub cutoff: usize,
    pub backtick: bool,
    pub control: u32,
    pub lines: Option<usize>,
    pub utf8: bool,
}

pub(crate) fn format_deparse(
    runtime: &mut RuntimeState,
    value: &Sexp,
    profile: Profile,
) -> Result<(Vec<String>, bool), EvalError> {
    let mut writer = Writer {
        runtime: Some(runtime),
        profile: Some(profile),
        ..Writer::default()
    };
    writer.write(value)?;
    let lines = writer
        .output
        .split('\n')
        .take(profile.lines.unwrap_or(usize::MAX))
        .map(str::to_owned)
        .collect();
    Ok((lines, writer.incomplete))
}

impl From<EvalError> for LanguagePrintError {
    fn from(error: EvalError) -> Self {
        Self::Runtime(Box::new(error))
    }
}

impl Writer<'_> {
    pub(super) fn options(&self) -> u32 {
        self.profile.map_or(1089, |profile| profile.control)
    }

    pub(super) fn set_options(&mut self, control: u32) {
        if let Some(profile) = &mut self.profile {
            profile.control = control;
        }
    }

    pub(super) fn check_name(&self, value: &str) -> Result<(), LanguagePrintError> {
        if self.profile.is_none() {
            return super::names::check_text(value);
        }
        if value.contains('\0') {
            Err(LanguagePrintError::InvalidText)
        } else {
            Ok(())
        }
    }

    pub(super) fn text(&self, value: &str, quote: char) -> Result<String, LanguagePrintError> {
        self.check_name(value)?;
        if self.profile.is_none_or(|profile| profile.utf8) || value.is_ascii() {
            return Ok(crate::encode_string::encode(value, Some(quote)));
        }
        let mut result = String::new();
        result.push(quote);
        for byte in value.bytes() {
            if byte.is_ascii() {
                let encoded =
                    crate::encode_string::encode(&char::from(byte).to_string(), Some(quote));
                result.push_str(&encoded[1..encoded.len() - 1]);
            } else {
                result.push_str(&format!("\\{byte:03o}"));
            }
        }
        result.push(quote);
        Ok(result)
    }

    fn quoted_name(&self, value: &str, quote: char) -> Result<String, LanguagePrintError> {
        self.check_name(value)?;
        if value.is_empty() || super::names::valid_name(value) {
            Ok(value.into())
        } else {
            self.text(value, quote)
        }
    }

    pub(super) fn caller_name(&self, value: &str) -> Result<String, LanguagePrintError> {
        self.quoted_name(
            value,
            if self.options() & S_COMPAT != 0 {
                '\''
            } else {
                '`'
            },
        )
    }

    pub(super) fn tag_name(&self, value: &str) -> Result<String, LanguagePrintError> {
        self.quoted_name(
            value,
            if self.profile.is_none_or(|profile| profile.backtick) {
                '`'
            } else {
                '"'
            },
        )
    }

    pub(super) fn symbol(&self, value: &str) -> Result<String, LanguagePrintError> {
        if self.options() & S_COMPAT != 0 {
            self.quoted_name(value, '"')
        } else if self.profile.is_none_or(|profile| profile.backtick) {
            self.quoted_name(value, '`')
        } else {
            self.check_name(value)?;
            Ok(value.into())
        }
    }
}
