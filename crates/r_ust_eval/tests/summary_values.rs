use r_ust_eval::Frontend;
use r_ust_sexp::{is_na_real, Sexp};

#[test]
fn summaries_match_independent_r_storage_class_and_exact_bits() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in include_str!("fixtures/summary-r-4.6.0.tsv").lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        if fields[5] != "-" {
            continue;
        }
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[1]);
        let actual = match result.as_ref().ok().and_then(|values| values.last()) {
            Some(Sexp::Real(values)) if values.len() == 1 => {
                let value = values[0];
                let class = if is_na_real(value) {
                    "NA"
                } else if value.is_nan() {
                    "NaN"
                } else {
                    "number"
                };
                Some(("double", class, format!("{:016x}", value.to_bits())))
            }
            Some(Sexp::Integer(values)) if values.len() == 1 => Some((
                "integer",
                if values[0].is_none() { "NA" } else { "number" },
                values[0].map_or_else(|| "NA".into(), |value| value.to_string()),
            )),
            _ => None,
        };
        let expected = Some((fields[2], fields[3], fields[4].to_string()));
        if actual != expected {
            failures.push(format!(
                "{}: {}: {actual:?}; expected {expected:?}; result {result:?}",
                fields[0], fields[1]
            ));
        }
        assert_eq!(
            fields[6], "-",
            "fixture now requires a warning assertion: {line}"
        );
        assert!(
            frontend.runtime_mut().take_output().is_empty(),
            "{}",
            fields[1]
        );
        checked += 1;
    }
    assert_eq!(checked, 3798);
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
