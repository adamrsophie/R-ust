use super::{dimensions::Dimensions, vector::pad};
use crate::{EvalError, Sexp};

pub(super) fn render(
    value: &Sexp,
    dim: &Dimensions,
    (offset, quote): (Option<usize>, bool),
) -> Result<String, EvalError> {
    let rows = dim.sizes[0];
    let columns = dim.sizes[1];
    let rownames: Vec<_> = (0..rows)
        .map(|i| {
            dim.labels[0]
                .as_ref()
                .map(|v| crate::encode_string::encode(v[i].as_deref().unwrap_or("<NA>"), None))
                .unwrap_or_else(|| format!("[{},]", i + 1))
        })
        .collect();
    let colnames: Vec<_> = (0..columns)
        .map(|i| {
            dim.labels[1]
                .as_ref()
                .map(|v| crate::encode_string::encode(v[i].as_deref().unwrap_or("<NA>"), None))
                .unwrap_or_else(|| format!("[,{}]", i + 1))
        })
        .collect();
    let label_width = if dim.labels[0].is_some() {
        rownames
            .iter()
            .map(|v| crate::encode_string::width(v))
            .max()
            .unwrap_or(0)
    } else {
        rows.saturating_add(1).to_string().len() + 3
    };
    let row_title = dim.titles.first().and_then(Option::as_deref);
    let label_offset = row_title.map_or(0, |title| {
        crate::encode_string::width(title)
            .saturating_sub(label_width)
            .max(2)
    });
    let row_width = label_width + label_offset;
    let mut data = Vec::new();
    let mut widths = Vec::new();
    let mut character = false;
    for (column, name) in colnames.iter().enumerate() {
        let payload = if let Some(offset) = offset {
            slice(value, offset + column * rows, rows)?
        } else {
            slice(value, 0, 0)?
        };
        let (tokens, is_character, _) = super::numeric::tokens(&payload, quote)?;
        character = is_character;
        widths.push(
            tokens
                .iter()
                .map(|v| crate::encode_string::width(v))
                .chain(std::iter::once(crate::encode_string::width(name)))
                .max()
                .unwrap_or(0),
        );
        data.push(tokens);
    }
    let mut lines = Vec::new();
    let mut first = 0;
    while first < columns {
        let mut end = first;
        let mut width = row_width;
        while end < columns && (end == first || width + widths[end] + 1 < 80) {
            width += widths[end] + 1;
            end += 1;
        }
        if let Some(Some(title)) = dim.titles.get(1) {
            lines.push(format!("{}{title}", " ".repeat(row_width)));
        }
        let mut header = pad(row_title.unwrap_or(""), row_width, true);
        for index in first..end {
            header.push(' ');
            header.push_str(&pad(&colnames[index], widths[index], character));
        }
        lines.push(header);
        for (row, name) in rownames.iter().enumerate() {
            let mut line = format!(
                "{}{}",
                " ".repeat(label_offset),
                pad(name, label_width, dim.labels[0].is_some())
            );
            if offset.is_some() {
                for column in first..end {
                    line.push(' ');
                    line.push_str(&pad(&data[column][row], widths[column], character));
                }
            }
            lines.push(line);
        }
        first = end;
    }
    if columns == 0 {
        if let Some(Some(title)) = dim.titles.get(1) {
            lines.push(format!("{}{title}", " ".repeat(row_width)));
        }
        lines.push(pad(row_title.unwrap_or(""), row_width, true));
        lines.extend(rownames.iter().map(|name| {
            format!(
                "{}{}",
                " ".repeat(label_offset),
                pad(name, label_width, dim.labels[0].is_some())
            )
        }));
    }
    Ok(lines.join("\n"))
}

fn slice(value: &Sexp, start: usize, length: usize) -> Result<Sexp, EvalError> {
    let end = start
        .checked_add(length)
        .ok_or_else(|| EvalError::Type("print matrix offset".into()))?;
    Ok(match value {
        Sexp::Logical(v) => Sexp::Logical(
            v.get(start..end)
                .ok_or_else(|| EvalError::Type("print matrix bounds".into()))?
                .to_vec(),
        ),
        Sexp::Integer(v) => Sexp::Integer(
            v.get(start..end)
                .ok_or_else(|| EvalError::Type("print matrix bounds".into()))?
                .to_vec(),
        ),
        Sexp::Real(v) => Sexp::Real(
            v.get(start..end)
                .ok_or_else(|| EvalError::Type("print matrix bounds".into()))?
                .to_vec(),
        ),
        Sexp::String(v) => Sexp::String(
            v.get(start..end)
                .ok_or_else(|| EvalError::Type("print matrix bounds".into()))?
                .to_vec(),
        ),
        _ => return Err(EvalError::Type("invalid matrix print payload".into())),
    })
}
