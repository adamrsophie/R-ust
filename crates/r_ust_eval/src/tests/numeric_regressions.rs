use super::{Frontend, Sexp};

const R_1E308: f64 = f64::from_bits(0x7fe1_ccf3_85eb_c8a3);

#[test]
fn follows_r_operator_precedence() {
    let mut frontend = Frontend::default();
    for (source, expected) in [
        ("-2^2", -4.0),
        ("(-2)^2", 4.0),
        ("2^-2", 0.25),
        ("2^3^2", 512.0),
        ("-2^-2", -0.25),
        ("2 ** 3", 8.0),
        ("-2**2 + 3 * 2 - 1 / 2", 1.5),
        ("2 * 3 %in% 6", 0.0),
        ("1^NaN", 1.0),
        ("NaN^0", 1.0),
        ("2 / -!3", f64::INFINITY),
        ("mean(c(1e308, 1e308))", R_1E308),
    ] {
        assert_eq!(
            frontend.eval_source(source).unwrap(),
            vec![Sexp::Real(vec![expected])],
            "{source}"
        );
    }
    for (source, expected) in [
        ("!2 == 1", vec![Some(true)]),
        ("!1:3 %in% 2:3", vec![Some(true), Some(false), Some(false)]),
        ("TRUE || FALSE | NA", vec![Some(true)]),
        ("FALSE && TRUE & NA", vec![Some(false)]),
        (
            "TRUE || FALSE | c(FALSE, FALSE)",
            vec![Some(true), Some(true)],
        ),
        (
            "FALSE && TRUE & c(TRUE, TRUE)",
            vec![Some(false), Some(false)],
        ),
        (
            "is.nan(c((-1)^Inf, (-Inf)^0.5, (-1)^0.5))",
            vec![Some(true); 3],
        ),
    ] {
        let values = frontend.eval_source(source).unwrap();
        assert_eq!(values.last(), Some(&Sexp::Logical(expected)), "{source}");
    }
    assert_eq!(
        frontend.eval_source("1:-2^2").unwrap(),
        vec![Sexp::Integer(vec![
            Some(1),
            Some(0),
            Some(-1),
            Some(-2),
            Some(-3),
            Some(-4)
        ])]
    );
}

#[test]
fn preserves_integer_storage_and_overflow() {
    let mut frontend = Frontend::default();
    for (source, expected) in [
        ("1L", vec![Some(1)]),
        ("1e2L", vec![Some(100)]),
        ("1.0L", vec![Some(1)]),
        ("c(1L, TRUE, NA_integer_)", vec![Some(1), Some(1), None]),
        ("1L + 2L", vec![Some(3)]),
        ("TRUE + FALSE", vec![Some(1)]),
        ("2L * c(1L, 2L, NA_integer_)", vec![Some(2), Some(4), None]),
        ("2147483647L + 1L", vec![None]),
        ("-2147483647L - 1L", vec![None]),
        ("50000L * 50000L", vec![None]),
        (
            "as.integer(c(-2147483648, 2147483647.9))",
            vec![None, Some(2147483647)],
        ),
        ("NULL + TRUE", vec![]),
        ("sum(1:3)", vec![Some(6)]),
        ("sum(NULL)", vec![Some(0)]),
        ("sum(NA_integer_)", vec![None]),
        ("range(1:3)", vec![Some(1), Some(3)]),
        (
            "cumsum(c(2147483647L, 1L, -1L))",
            vec![Some(2147483647), None, None],
        ),
        ("cummin(3:1)", vec![Some(3), Some(2), Some(1)]),
        ("median(1:3)", vec![Some(2)]),
        ("diff(1:3)", vec![Some(1), Some(1)]),
        ("unlist(list(1L, 2L))", vec![Some(1), Some(2)]),
        ("seq(1L, 3L, by=1L)", vec![Some(1), Some(2), Some(3)]),
        ("1:3", vec![Some(1), Some(2), Some(3)]),
        ("3:1", vec![Some(3), Some(2), Some(1)]),
        ("1:3.5", vec![Some(1), Some(2), Some(3)]),
        ("1:2.99999999", vec![Some(1), Some(2), Some(3)]),
        (
            "2147483646:2147483647.5",
            vec![Some(2147483646), Some(2147483647)],
        ),
        ("seq_len(3.9)", vec![Some(1), Some(2), Some(3)]),
        ("seq_len(0)", vec![]),
        ("seq_along(c(\"a\", \"b\"))", vec![Some(1), Some(2)]),
        ("seq(3)", vec![Some(1), Some(2), Some(3)]),
        (
            "sapply(1:3, function(x) x)",
            vec![Some(1), Some(2), Some(3)],
        ),
    ] {
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Integer(expected)),
            "{source}"
        );
    }
    for (source, expected) in [
        ("1L + 2", vec![3.0]),
        ("1L / 2L", vec![0.5]),
        ("2L ^ 3L", vec![8.0]),
        ("1.5L", vec![1.5]),
        ("2147483648L", vec![2147483648.0]),
        ("1.5:3", vec![1.5, 2.5]),
        ("1e20:1e20", vec![1e20]),
        ("sum(c(2147483647L, 1L))", vec![2147483648.0]),
        ("median(1:4)", vec![2.5]),
        ("cumprod(1:3)", vec![1.0, 2.0, 6.0]),
        ("seq(-1e308, 1e308, by=1e308)", vec![-R_1E308, 0.0, R_1E308]),
    ] {
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Real(expected)),
            "{source}"
        );
    }
    for source in ["seq_len(-1)", "seq_len(NA)", "seq_len(Inf)"] {
        assert!(frontend.eval_source(source).is_err(), "{source}");
    }
}
