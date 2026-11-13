use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{is_na_real, Sexp};

#[test]
fn floor_operators_preserve_r_types_values_and_precedence() {
    for (source, expected) in [
        ("-7L %% 3L", Sexp::Integer(vec![Some(2)])),
        ("7L %% -3L", Sexp::Integer(vec![Some(-2)])),
        ("-7L %/% 3L", Sexp::Integer(vec![Some(-3)])),
        ("7L %/% -3L", Sexp::Integer(vec![Some(-3)])),
        ("TRUE %% TRUE", Sexp::Integer(vec![Some(0)])),
        ("NA_integer_ %/% 2L", Sexp::Integer(vec![None])),
        ("1L %% 0L", Sexp::Integer(vec![None])),
        ("1L %/% 0L", Sexp::Integer(vec![None])),
        ("NULL %% TRUE", Sexp::Integer(vec![])),
        ("as.double(NULL) %/% 1:3", Sexp::Real(vec![])),
        ("1 + 7 %% 3 * 2", Sexp::Real(vec![3.0])),
        ("20 %/% 3 %/% 2", Sexp::Real(vec![3.0])),
        ("1:4 %% 3", Sexp::Real(vec![1.0, 2.0, 0.0, 1.0])),
        ("2^3 %% 3", Sexp::Real(vec![2.0])),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source).unwrap(),
            vec![expected],
            "{source}"
        );
    }
}

#[test]
fn floor_operators_match_independent_r_binary64_fixtures() {
    for row in include_str!("fixtures/floor-operators-r-4.6.0.tsv").lines() {
        if row.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = row.split('\t').collect();
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[0]).unwrap();
        let Sexp::Real(values) = &result[0] else {
            panic!("expected real: {row}")
        };
        let value = values[0];
        let actual = if is_na_real(value) {
            "NA".to_string()
        } else if value.is_nan() {
            "NaN".to_string()
        } else {
            format!("{:016x}", value.to_bits())
        };
        assert_eq!(actual, fields[1], "{}", fields[0]);
        let warnings = frontend.runtime_mut().take_output();
        assert_eq!(
            warnings.is_empty(),
            fields[2] == "0",
            "{}: {warnings:?}",
            fields[0]
        );
    }
}

#[test]
fn floor_operator_warnings_obey_policy_and_order() {
    for warn in [-1, 0, 1, 2] {
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(warn)]))
            .unwrap();
        let result = frontend.eval_source_printed("1:3 %% 1:2");
        let warning = "longer object length is not a multiple of shorter object length";
        let value = OutputEvent::Stdout("[1] 0 0 0\n".into());
        let expected = match warn {
            -1 => vec![value],
            0 => vec![
                value,
                OutputEvent::Stderr(format!("Warning message:\n{warning} \n")),
            ],
            1 => vec![OutputEvent::Stderr(format!("Warning: {warning}\n")), value],
            _ => Vec::new(),
        };
        assert_eq!(
            frontend.runtime_mut().take_output(),
            expected,
            "warn={warn}"
        );
        if warn == 2 {
            assert!(result.unwrap_err().to_string().contains(warning));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn floor_operators_keep_matrix_shapes_and_factor_dispatch() {
    for (source, expected) in [
        (
            "matrix(1:4,2) %/% c(2,3)",
            Sexp::Matrix {
                data: Box::new(Sexp::Real(vec![0.0, 0.0, 1.0, 1.0])),
                nrow: 2,
                ncol: 2,
            },
        ),
        ("matrix(1:4,2) %% as.double(NULL)", Sexp::Real(vec![])),
        (
            "matrix(as.double(NULL),0,2) %% 1",
            Sexp::Matrix {
                data: Box::new(Sexp::Real(vec![])),
                nrow: 0,
                ncol: 2,
            },
        ),
        ("factor(c('a','b')) %% 2", Sexp::Logical(vec![None, None])),
        (
            "factor(as.character(NULL)) %% 1:3",
            Sexp::Logical(vec![None, None, None]),
        ),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source).unwrap(),
            vec![expected],
            "{source}"
        );
    }
    let mut frontend = Frontend::default();
    let error = frontend
        .eval_source("matrix(1:4,2) %% 1:6")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("dims [product 4] do not match the length of object [6]"),
        "{error}"
    );
    assert!(error.contains("longer object length"), "{error}");
    assert!(Frontend::default()
        .eval_source("matrix(1:4,2) %% matrix(1:4,1)")
        .unwrap_err()
        .to_string()
        .contains("non-conformable arrays"));
}
