use crate::Sexp;

impl Sexp {
    pub fn character_values(&self) -> Option<&[Option<String>]> {
        match self {
            Self::String(values) | Self::AttributedCharacter { values, .. } => Some(values),
            _ => None,
        }
    }
}
