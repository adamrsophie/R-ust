use crate::{factor_strings, format_line};
pub(super) fn format_factor(codes: &[Option<usize>], levels: &[String], ordered: bool) -> String {
    let values = factor_strings(codes, levels)
        .into_iter()
        .map(|value| value.unwrap_or_else(|| "<NA>".to_string()))
        .collect::<Vec<_>>();
    let levels_line = if ordered {
        format!("Levels: {}", levels.join(" < "))
    } else {
        format!("Levels: {}", levels.join(" "))
    };
    if values.is_empty() {
        levels_line
    } else {
        format!("{}\n{levels_line}", format_line(values))
    }
}
