use std::ops::RangeInclusive;

use crate::Sexp;

pub(super) fn lookup(name: &str) -> Option<Sexp> {
    let value = match name {
        "T" => Sexp::Logical(vec![Some(true)]),
        "F" => Sexp::Logical(vec![Some(false)]),
        "pi" => Sexp::Real(vec![std::f64::consts::PI]),
        "letters" => alphabet('a'..='z'),
        "LETTERS" => alphabet('A'..='Z'),
        "month.name" => strings(&[
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
        ]),
        "month.abb" => strings(&[
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]),
        _ => return None,
    };
    Some(value)
}

fn alphabet(range: RangeInclusive<char>) -> Sexp {
    Sexp::String(range.map(|letter| Some(letter.to_string())).collect())
}

fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}
