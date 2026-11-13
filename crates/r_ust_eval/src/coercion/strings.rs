use r_ust_parse::parse_numeric_prefix;
use r_ust_sexp::NA_REAL;

use super::warnings::Warnings;

#[derive(Clone, Copy, Debug)]
pub(super) enum BlankPolicy {
    C,
    Utf8,
}

impl BlankPolicy {
    pub(super) fn environment() -> Self {
        Self::locales(
            ["LC_ALL", "LC_CTYPE", "LANG"]
                .into_iter()
                .filter_map(|name| std::env::var(name).ok()),
        )
    }

    pub(super) fn locales(locales: impl IntoIterator<Item = String>) -> Self {
        let locale = locales.into_iter().find(|locale| !locale.is_empty());
        match locale {
            Some(locale) => {
                let locale = locale.to_ascii_uppercase();
                if locale.contains("UTF-8") || locale.contains("UTF8") {
                    Self::Utf8
                } else {
                    // shortcut: other locale encodings use C blanks until runtime locale support exists.
                    Self::C
                }
            }
            None if cfg!(target_os = "macos") => Self::Utf8,
            None => Self::C,
        }
    }

    pub(super) fn is_blank(self, value: &str) -> bool {
        value.chars().all(|ch| {
            matches!(ch, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r')
                || matches!(self, Self::Utf8)
                    && matches!(
                        ch,
                        '\u{1680}'
                            | '\u{2000}'..='\u{2006}'
                            | '\u{2008}'..='\u{200a}'
                            | '\u{2028}'
                            | '\u{2029}'
                            | '\u{205f}'
                            | '\u{3000}'
                    )
        })
    }
}

pub(super) fn real(input: Option<&str>, policy: BlankPolicy, warnings: &mut Warnings) -> f64 {
    let Some(input) = input else {
        return NA_REAL;
    };
    if policy.is_blank(input) {
        return NA_REAL;
    }
    match parse_numeric_prefix(input) {
        Some((value, suffix)) if policy.is_blank(suffix) => value,
        Some(_) | None => {
            warnings.generic = true;
            NA_REAL
        }
    }
}
