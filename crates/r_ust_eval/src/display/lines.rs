// None is no printed line; Some("") is one empty printed line.
pub(super) fn append(output: &mut Option<String>, value: String) {
    match output {
        Some(output) => {
            output.push('\n');
            output.push_str(&value);
        }
        None => *output = Some(value),
    }
}
