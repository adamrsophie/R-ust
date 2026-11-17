use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn extract(
    args: &[EvaluatedArgument],
) -> Result<Option<Vec<Option<String>>>, EvalError> {
    let mut output = Vec::new();
    let mut present = false;
    for arg in args {
        let count = crate::value_len(&arg.value)?;
        let names = crate::list_attributes::engine::get(&arg.value, "names")?;
        let names = crate::coercion::storage::read(&names)?;
        let inner = match names.as_ref() {
            Sexp::Nil => None,
            value => Some(
                value
                    .character_values()
                    .filter(|values| values.len() == count)
                    .ok_or_else(|| EvalError::Type("invalid concatenation names".into()))?,
            ),
        };
        let outer = arg.name.as_deref().filter(|name| !name.is_empty());
        present |= outer.is_some() || inner.is_some();
        for index in 0..count {
            let name = inner.map(|values| &values[index]);
            output.push(new_name(outer, name, (index + 1, count)));
        }
    }
    Ok((present && !output.is_empty()).then_some(output))
}

fn new_name(
    outer: Option<&str>,
    inner: Option<&Option<String>>,
    position: (usize, usize),
) -> Option<String> {
    let Some(outer) = outer else {
        return inner.cloned().unwrap_or_else(|| Some(String::new()));
    };
    let inner = inner.map(|name| name.as_deref().unwrap_or("NA"));
    Some(match inner.filter(|name| !name.is_empty()) {
        Some(inner) => format!("{outer}.{inner}"),
        None if position.1 == 1 => outer.to_owned(),
        None => format!("{outer}{}", position.0),
    })
}
