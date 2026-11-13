use r_ust_eval::Frontend;
use r_ust_sexp::{is_na_real, Sexp};

#[test]
fn addition_preserves_r_missing_and_nan_classes() {
    for (source, missing) in [("NA_real_ + 1", true), ("NaN + 1", false)] {
        let values = Frontend::default().eval_source(source).unwrap();
        let [Sexp::Real(values)] = values.as_slice() else {
            panic!("expected a real vector for {source}: {values:?}");
        };
        let [value] = values.as_slice() else {
            panic!("expected one result for {source}: {values:?}");
        };
        assert!(value.is_nan(), "{source}");
        assert_eq!(is_na_real(*value), missing, "{source}");
    }
}
