use std::ops::RangeInclusive;

use crate::Sexp;

type Constant = (&'static str, fn() -> Sexp);

const CONSTANTS: &[Constant] = &[
    ("T", || Sexp::Logical(vec![Some(true)])),
    ("F", || Sexp::Logical(vec![Some(false)])),
    ("pi", || Sexp::Real(vec![std::f64::consts::PI])),
    ("letters", || alphabet('a'..='z')),
    ("LETTERS", || alphabet('A'..='Z')),
    ("month.name", || {
        strings(&[
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ])
    }),
    ("month.abb", || {
        strings(&[
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ])
    }),
];

pub(super) fn lookup(name: &str) -> Option<Sexp> {
    CONSTANTS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value())
}

pub(super) fn iter() -> impl Iterator<Item = (&'static str, Sexp)> {
    CONSTANTS.iter().map(|(name, value)| (*name, value()))
}

fn alphabet(range: RangeInclusive<char>) -> Sexp {
    Sexp::String(range.map(|letter| Some(letter.to_string())).collect())
}

fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}
