use super::{
    profile::{DIGITS17, HEX, KEEP_INTEGER, KEEP_NA, S_COMPAT},
    LanguagePrintError, Writer,
};
use crate::{ListElement, Sexp};

impl Writer<'_> {
    pub(super) fn deparse_atomic(
        &mut self,
        value: &Sexp,
        attributes: &[ListElement],
    ) -> Result<(), LanguagePrintError> {
        let length = match value {
            Sexp::Logical(v) => v.len(),
            Sexp::Integer(v) => v.len(),
            Sexp::Real(v) => v.len(),
            Sexp::String(v) => v.len(),
            _ => return Err(LanguagePrintError::UnsupportedNode),
        };
        let sequence = if let Sexp::Integer(values) = value {
            integer_sequence(values)
        } else {
            None
        };
        let plan = self.begin_attributes(attributes, true, sequence.is_some() || length == 0);
        let names = attributes
            .iter()
            .find(|a| a.name.as_deref() == Some("names"))
            .and_then(|a| a.value.character_values());
        if let Some(sequence) = sequence {
            self.emit(&sequence);
        } else if length == 0 {
            self.emit(match value {
                Sexp::Logical(_) => "logical(0)",
                Sexp::Integer(_) => "integer(0)",
                Sexp::Real(_) => "numeric(0)",
                _ => "character(0)",
            });
        } else {
            let options = self.options();
            let s_compatible = options & S_COMPAT != 0;
            let integer = options & KEEP_INTEGER != 0;
            let all_na = match value {
                Sexp::Integer(values) => {
                    (options & KEEP_NA != 0 || integer) && values.iter().all(Option::is_none)
                }
                Sexp::Real(values) => {
                    options & KEEP_NA != 0 && values.iter().all(|v| r_ust_sexp::is_na_real(*v))
                }
                Sexp::String(values) => {
                    options & KEEP_NA != 0 && values.iter().all(Option::is_none)
                }
                _ => false,
            };
            let surround = if s_compatible {
                match value {
                    Sexp::Integer(_) if integer => Some("as.integer("),
                    Sexp::Real(_) if all_na => Some("as.double("),
                    Sexp::String(_) if all_na => Some("as.character("),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(open) = surround {
                self.emit(open);
            }
            let vector = length > 1 || plan.inline_names;
            if vector {
                self.emit("c(");
            }
            for index in 0..length {
                if !self.active() {
                    break;
                }
                if plan.inline_names {
                    if let Some(name) = names.and_then(|values| values.get(index)) {
                        let name = name.as_deref().unwrap_or("NA");
                        if !name.is_empty() {
                            self.emit(&self.tag_name(name)?);
                            self.emit(" = ");
                        }
                    }
                }
                let token = self.atomic_token(value, index, all_na && !s_compatible)?;
                self.emit(&token);
                if index + 1 < length {
                    self.emit(", ");
                }
                if length > 1 && self.len > self.profile.map_or(60, |p| p.cutoff) {
                    self.newline();
                }
            }
            if vector {
                self.emit(")");
            }
            if surround.is_some() {
                self.emit(")");
            }
        }
        self.end_attributes(plan)
    }

    fn atomic_token(
        &self,
        value: &Sexp,
        index: usize,
        typed_na: bool,
    ) -> Result<String, LanguagePrintError> {
        let options = self.options();
        Ok(match value {
            Sexp::Logical(values) => match values[index] {
                Some(true) => "TRUE",
                Some(false) => "FALSE",
                None => "NA",
            }
            .into(),
            Sexp::Integer(values) => match values[index] {
                Some(value) if value == i32::MIN => {
                    return Err(LanguagePrintError::UnsupportedNode)
                }
                Some(value) => {
                    if options & KEEP_INTEGER != 0 && options & S_COMPAT == 0 {
                        format!("{value}L")
                    } else {
                        value.to_string()
                    }
                }
                None => if typed_na { "NA_integer_" } else { "NA" }.into(),
            },
            Sexp::Real(values) => {
                let value = values[index];
                if value.is_finite() && options & S_COMPAT == 0 && options & (HEX | DIGITS17) != 0 {
                    crate::internal::sprintf::deparse_real(value, options & HEX != 0)
                        .map_err(LanguagePrintError::from)?
                } else {
                    let mut result = crate::real_literal::format_real(value, typed_na);
                    if options & S_COMPAT != 0 && value.is_finite() && !result.contains(['.', 'e'])
                    {
                        result.push('.');
                    }
                    result
                }
            }
            Sexp::String(values) => match &values[index] {
                Some(value) => self.text(value, '"')?,
                None => if typed_na { "NA_character_" } else { "NA" }.into(),
            },
            _ => return Err(LanguagePrintError::UnsupportedNode),
        })
    }
}

fn integer_sequence(values: &[Option<i32>]) -> Option<String> {
    let [Some(first), Some(second), ..] = values else {
        return None;
    };
    let step = second.checked_sub(*first)?;
    if !matches!(step, 1 | -1) {
        return None;
    }
    if !values.windows(2).all(|pair| match pair {
        [Some(left), Some(right)] => right.checked_sub(*left) == Some(step),
        _ => false,
    }) {
        return None;
    }
    Some(format!("{first}:{}", values.last().copied().flatten()?))
}
