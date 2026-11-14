use super::*;

pub(super) fn readline_prompt(value: &Sexp) -> Result<Option<String>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Some(String::new())),
        Sexp::Matrix { data, .. } => readline_prompt(data),
        Sexp::Factor { codes, .. } => Ok(codes
            .first()
            .map(|code| code.map_or_else(|| "NA".to_string(), |code| code.to_string()))),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::DataFrame {
            columns: values, ..
        } => values
            .first()
            .map(|element| readline_list_prompt(&element.value))
            .transpose(),
        Sexp::Vector(values) => values.first().map(readline_list_prompt).transpose(),
        _ => Ok(string_values(value)?
            .first()
            .map(|value| value.clone().unwrap_or_else(|| "NA".to_string()))),
    }
}
