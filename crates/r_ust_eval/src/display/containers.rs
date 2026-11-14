use super::format_sexp;
use crate::{factor_strings, format_number, sequence_elements, EvalError, ListElement, Sexp};
pub(super) fn format_data_frame(columns: &[ListElement], nrow: usize) -> Result<String, EvalError> {
    if columns.is_empty() {
        return Ok("data.frame()".to_string());
    }
    let names = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            column
                .name
                .clone()
                .unwrap_or_else(|| format!("X{}", index + 1))
        })
        .collect::<Vec<_>>();
    let column_values = columns
        .iter()
        .map(|column| data_frame_display_values(&column.value))
        .collect::<Result<Vec<_>, _>>()?;
    let mut lines = Vec::new();
    lines.push(
        std::iter::once(" ".to_string())
            .chain(names)
            .collect::<Vec<_>>()
            .join(" "),
    );
    for row in 0..nrow {
        let mut cells = vec![(row + 1).to_string()];
        for values in &column_values {
            cells.push(values.get(row).cloned().unwrap_or_else(|| "NA".to_string()));
        }
        lines.push(cells.join(" "));
    }
    Ok(lines.join("\n"))
}

fn data_frame_display_values(value: &Sexp) -> Result<Vec<String>, EvalError> {
    Ok(match value {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values
            .iter()
            .map(|value| match value {
                Some(true) => "TRUE".to_string(),
                Some(false) => "FALSE".to_string(),
                None => "NA".to_string(),
            })
            .collect(),
        Sexp::Integer(values) => values
            .iter()
            .map(|value| value.map_or_else(|| "NA".to_string(), |value| value.to_string()))
            .collect(),
        Sexp::Real(values) => values.iter().map(|value| format_number(*value)).collect(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values
            .iter()
            .map(|value| match value {
                Some(value) => format!("{value:?}"),
                None => "NA".to_string(),
            })
            .collect(),
        Sexp::Factor { codes, levels, .. } => factor_strings(codes, levels)
            .into_iter()
            .map(|value| value.unwrap_or_else(|| "NA".to_string()))
            .collect(),
        Sexp::Matrix { data, .. } => data_frame_display_values(data)?,
        other => sequence_elements(other)?
            .iter()
            .map(format_sexp)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub(super) fn format_matrix(data: &Sexp, nrow: usize, ncol: usize) -> Result<String, EvalError> {
    let values = matrix_display_values(data)?;
    if nrow == 0 || ncol == 0 {
        return Ok(format!("<{nrow} x {ncol} matrix>"));
    }
    let mut lines = Vec::new();
    lines.push(
        std::iter::once("     ".to_string())
            .chain((1..=ncol).map(|col| format!("[,{col}]")))
            .collect::<Vec<_>>()
            .join(" "),
    );
    for row in 0..nrow {
        let mut cells = vec![format!("[{},]", row + 1)];
        for col in 0..ncol {
            let index = row + col * nrow;
            cells.push(
                values
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| "NA".to_string()),
            );
        }
        lines.push(cells.join(" "));
    }
    Ok(lines.join("\n"))
}

fn matrix_display_values(data: &Sexp) -> Result<Vec<String>, EvalError> {
    Ok(match data {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values
            .iter()
            .map(|value| match value {
                Some(true) => "TRUE".to_string(),
                Some(false) => "FALSE".to_string(),
                None => "NA".to_string(),
            })
            .collect(),
        Sexp::Integer(values) => values
            .iter()
            .map(|value| value.map_or_else(|| "NA".to_string(), |value| value.to_string()))
            .collect(),
        Sexp::Real(values) => values.iter().map(|value| format_number(*value)).collect(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values
            .iter()
            .map(|value| match value {
                Some(value) => format!("{value:?}"),
                None => "NA".to_string(),
            })
            .collect(),
        Sexp::Factor { codes, levels, .. } => factor_strings(codes, levels)
            .into_iter()
            .map(|value| value.unwrap_or_else(|| "NA".to_string()))
            .collect(),
        Sexp::Matrix { data, .. } => matrix_display_values(data)?,
        other => sequence_elements(other)?
            .iter()
            .map(format_sexp)
            .collect::<Result<Vec<_>, _>>()?,
    })
}
