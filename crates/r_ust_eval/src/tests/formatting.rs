use super::{format_sexp, EvalError, Frontend, Sexp};

#[test]
fn prints_small_doubles_without_losing_precision() {
    assert_eq!(
        format_sexp(&Sexp::Real(vec![1e-13, 1.0 / i32::MAX as f64])).unwrap(),
        "[1]             1e-13 4.65661287525e-10"
    );
}

#[test]
fn distinguishes_missing_doubles_from_nan() {
    let mut frontend = Frontend::default();
    for (source, expected) in [
        ("is.nan(c(NA, NaN))", "[1] FALSE  TRUE"),
        ("c(typeof(NA_integer_), typeof(NA_real_), typeof(NA_character_))", "[1] \"integer\" \"double\" \"character\""),
        ("c(NA_real_, NaN)", "[1]  NA NaN"),
        ("is.na(c(NA_real_, NaN, Inf, 0))", "[1]  TRUE  TRUE FALSE FALSE"),
        ("is.nan(as.double(c(NA_integer_, 1)))", "[1] FALSE FALSE"),
        ("as.character(c(NA_real_, NaN))", "[1] NA \"NaN\""),
        ("as.double(c(NA_character_, \"NaN\", \"bad\"))", "[1]  NA NaN  NA"),
        ("as.double(NA) == \"NaN\"", "[1] NA"),
        ("x <- c(1, NaN); x[c(2, 3, NA)]", "[1] NaN  NA  NA"),
        ("x <- c(1, NaN); x[4] <- 9; x", "[1]   1 NaN  NA   9"),
        ("c(NA_real_ + 1, 1 + NA_real_, NaN + NA_real_, NA_real_ + NaN)", "[1] NA NA NA NA"),
        ("c(1^NA_real_, NA_real_^0, NA_real_^2, 2^NA_real_)", "[1]  1  1 NA NA"),
        ("c(is.nan(sqrt(NA_real_)), is.nan(log(NA_real_)), is.nan(gamma(NA_real_)), is.nan(dnorm(NA_real_)))", "[1] FALSE FALSE FALSE FALSE"),
        ("c(identical(NA_real_, NaN), identical(NA_real_, NA_real_), identical(NaN, 0/0))", "[1] FALSE  TRUE  TRUE"),
        ("match(c(NA_real_, NaN), c(NaN, NA_real_))", "[1] 2 1"),
        ("duplicated(c(NA_real_, NaN, NA_real_, NaN))", "[1] FALSE FALSE  TRUE  TRUE"),
        ("unique(c(NA_real_, NaN, NA_real_, NaN))", "[1]  NA NaN"),
        ("c(NA_real_ %in% NaN, NaN %in% NA_real_)", "[1] FALSE FALSE"),
        ("c(sum(NaN, NA_real_), sum(NA_real_, NaN), prod(NaN, NA_real_))", "[1] NaN  NA NaN"),
        ("c(min(NaN, NA_real_), max(NA_real_, NaN))", "[1] NA NA"),
        ("range(c(NA_real_, NaN))", "[1] NA NA"),
        ("c(median(NaN), median(as.double(NULL)), var(1), sd(NaN))", "[1] NA NA NA NA"),
        ("cumsum(c(NaN, 1, NA_real_, 2))", "[1] NaN NaN  NA  NA"),
        ("cummin(c(NaN, 1, NA_real_, 2))", "[1] NaN NaN  NA  NA"),
        ("pmin(c(NaN, NA_real_), c(NA_real_, NaN))", "[1]  NA NaN"),
        ("pmin(c(NA_real_, NaN), c(NaN, NA_real_), na.rm=TRUE)", "[1] NaN  NA"),
        ("c(mean(c(NaN, NA_real_)), mean(c(NA_real_, NaN)))", "[1] NaN  NA"),
        ("(1:3)[c(1, NA_real_, Inf, -Inf)]", "[1]  1 NA NA NA"),
        ("c(sum(c(NA_real_, NaN), na.rm=TRUE), prod(c(NA_real_, NaN), na.rm=TRUE))", "[1] 0 1"),
    ] {
        let values = frontend.eval_source(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(format_sexp(values.last().unwrap()).unwrap(), expected, "{source}");
    }
    for source in ["(1:3)[c(-1, NA)]", "(1:3)[c(-1, 2)]"] {
        assert!(
            matches!(frontend.eval_source(source), Err(EvalError::Domain(_))),
            "{source}"
        );
    }
}

#[test]
fn formats_atomic_values() {
    assert_eq!(
        format_sexp(&Sexp::Real(vec![1.0, f64::INFINITY])).unwrap(),
        "[1]   1 Inf"
    );
    assert_eq!(
        format_sexp(&Sexp::Logical(vec![Some(true), None, Some(false)])).unwrap(),
        "[1]  TRUE    NA FALSE"
    );
}
