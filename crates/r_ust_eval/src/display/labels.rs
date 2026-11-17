pub(super) fn name(value: &str) -> String {
    let encoded = crate::encode_string::encode(value, None);
    if crate::language_print::valid_name(&encoded) {
        encoded
    } else {
        format!("`{encoded}`")
    }
}
