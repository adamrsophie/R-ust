use super::*;

#[test]
fn every_base_source_template_lowers_independently() {
    let failures = definitions()
        .filter_map(|(name, source)| {
            template(name, source)
                .err()
                .map(|error| format!("{name}: {error}"))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
