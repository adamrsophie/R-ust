pub(super) fn format_numeric_line(mut values: Vec<String>, empty: &str) -> String {
    if values.is_empty() {
        return empty.into();
    }
    let width = values.iter().map(String::len).max().unwrap_or(0);
    for value in &mut values {
        if value.len() < width {
            *value = format!("{value:>width$}");
        }
    }
    format_line(values)
}

pub(super) fn format_line(values: Vec<String>) -> String {
    if values.is_empty() {
        "[1]".to_string()
    } else {
        format!("[1] {}", values.join(" "))
    }
}
