use crate::{ListElement, Sexp};
pub(super) fn a(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
pub(super) fn i(values: &[i32]) -> Sexp {
    Sexp::Integer(values.iter().copied().map(Some).collect())
}
pub(super) fn s(values: &[Option<&str>]) -> Sexp {
    Sexp::String(values.iter().map(|v| v.map(str::to_string)).collect())
}
