use crate::EvalError;

#[derive(Clone, Copy)]
pub(crate) enum Encoding {
    C,
    Utf8,
}

pub(crate) fn current() -> Result<Encoding, EvalError> {
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    match locale {
        Some(locale) if matches!(locale.as_str(), "C" | "POSIX") => Ok(Encoding::C),
        Some(locale)
            if locale.to_ascii_uppercase().contains("UTF-8")
                || locale.to_ascii_uppercase().contains("UTF8") =>
        {
            Ok(Encoding::Utf8)
        }
        None if cfg!(target_os = "macos") => Ok(Encoding::Utf8),
        None => Ok(Encoding::C),
        _ => Err(EvalError::NotImplemented(
            "locale encoding other than C or UTF-8",
        )),
    }
}
