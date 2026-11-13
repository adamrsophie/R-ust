use crate::{match_keys, string_values, EvalError, MatchKey, Sexp};

pub(super) fn match_indices(
    x: &Sexp,
    table: &Sexp,
    nomatch: Option<i32>,
) -> Result<Vec<Option<i32>>, EvalError> {
    let mut table_keys = match_keys(table)?;
    let mut x_keys = match_keys(x)?;
    let character = [x, table].into_iter().any(|mut value| {
        while let Sexp::Matrix { data, .. } = value {
            value = data;
        }
        matches!(value, Sexp::String(_) | Sexp::Factor { .. })
    });
    if character {
        table_keys = match_keys(&Sexp::String(string_values(table)?))?;
        x_keys = match_keys(&Sexp::String(string_values(x)?))?;
    } else {
        for key in table_keys.iter_mut().chain(x_keys.iter_mut()) {
            match key {
                MatchKey::Logical(value) => {
                    *key = MatchKey::Number(f64::from(u8::from(*value)).to_bits());
                }
                MatchKey::Missing | MatchKey::Nan | MatchKey::Number(_) | MatchKey::String(_) => {}
            }
        }
    }
    x_keys
        .into_iter()
        .map(
            |key| match table_keys.iter().position(|candidate| *candidate == key) {
                Some(position) => i32::try_from(position + 1)
                    .map(Some)
                    .map_err(|_| EvalError::Domain("long vector match index".into())),
                None => Ok(nomatch),
            },
        )
        .collect()
}
