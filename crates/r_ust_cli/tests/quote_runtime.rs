use std::process::Command;

const CASES: &str = include_str!("../../r_ust_eval/tests/fixtures/quote_runtime/cases.tsv");
const REFERENCE: &str = include_str!("../../r_ust_eval/tests/fixtures/quote_runtime/reference.tsv");
const MAPPING: &str = include_str!("../../r_ust_eval/tests/fixtures/quote_runtime/mapping.tsv");
const ERRORS: &str = include_str!("../../r_ust_eval/tests/fixtures/quote_runtime/errors.tsv");

struct Fixture {
    cases: &'static str,
    reference: &'static str,
    mapping: &'static str,
    errors: &'static str,
}

fn decode(value: &str) -> String {
    assert!(value.len().is_multiple_of(2));
    let bytes: Vec<u8> = value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn quote_rscript_preserves_independent_r_output_status_and_inert_payloads() {
    assert_eq!(
        check_fixture(Fixture {
            cases: CASES,
            reference: REFERENCE,
            mapping: MAPPING,
            errors: ERRORS
        }),
        (67, 63)
    );
}

#[test]
fn accessor_rscript_preserves_independent_r_handles_effects_and_validation() {
    assert_eq!(
        check_fixture(Fixture {
            cases: include_str!("../../r_ust_eval/tests/fixtures/primitive_accessor/cases.tsv"),
            reference: include_str!(
                "../../r_ust_eval/tests/fixtures/primitive_accessor/reference.tsv"
            ),
            mapping: include_str!("../../r_ust_eval/tests/fixtures/primitive_accessor/mapping.tsv"),
            errors: include_str!("../../r_ust_eval/tests/fixtures/primitive_accessor/errors.tsv"),
        }),
        (62, 54)
    );
}

fn check_fixture(fixture: Fixture) -> (usize, usize) {
    let inputs: Vec<_> = fixture.cases.lines().skip(1).collect();
    let values: Vec<_> = fixture.reference.lines().skip(1).collect();
    let mappings: Vec<_> = fixture.mapping.lines().skip(1).collect();
    let count = inputs.len();
    assert_eq!(count, values.len());
    assert_eq!(count, mappings.len());
    let mut checked = 0;
    for ((input, expected), mapping) in inputs.into_iter().zip(values).zip(mappings) {
        let fields: Vec<_> = input.split('\t').collect();
        let observed: Vec<_> = expected.split('\t').collect();
        let (id, kind) = mapping.split_once('\t').unwrap();
        assert_eq!(fields[0], id);
        assert_eq!(observed[0], id);
        if kind != "direct" {
            continue;
        }
        let source = format!(
            "options(warn=1L,warnPartialMatchArgs=FALSE,keep.source=FALSE,digits=7L,scipen=0L,width=80L,deparse.cutoff=60L)\n{}\n{}\n{}\ncat(\"AFTER_SUBJECT_SENTINEL\\n\")\n",
            decode(fields[2]), decode(fields[3]), decode(fields[4])
        );
        let output = Command::new(env!("CARGO_BIN_EXE_Rscript"))
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("R_ENABLE_JIT", "0")
            .env("R_DEFAULT_PACKAGES", "base")
            .args(["--vanilla", "-e", &source])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(observed[12].parse().unwrap()),
            "{id}"
        );
        assert_eq!(
            output.stdout,
            decode(observed[13]).as_bytes(),
            "{id} stdout"
        );
        let stderr = if observed[2] == "error" {
            let row = fixture
                .errors
                .lines()
                .skip(1)
                .find(|row| row.split('\t').next() == Some(id))
                .unwrap();
            let payload = decode(row.split('\t').nth(1).unwrap());
            let prefix =
                if id.starts_with("missing_") || matches!(id, "first_error" | "second_error") {
                    ""
                } else if id.starts_with("tag_") {
                    "domain error: "
                } else if fields[1] == "type" {
                    "type error: "
                } else if fields[1] == "name" {
                    "domain error: "
                } else {
                    "argument error: "
                };
            format!("{prefix}{payload}\n")
        } else {
            let conditions = decode(observed[10]);
            conditions
                .lines()
                .collect::<Vec<_>>()
                .chunks(3)
                .map(|condition| format!("Warning: {}\n", condition[1]))
                .collect()
        };
        assert_eq!(output.stderr, stderr.as_bytes(), "{id} stderr");
        checked += 1;
    }
    (count, checked)
}

#[test]
fn quote_printing_failure_preserves_inert_payload_and_metadata_access() {
    for (source, status, expected) in [
        ("quote({cat('FORCED'); f('é')})", 1, ""),
        (
            "typeof(quote({cat('FORCED'); f('é')}))",
            0,
            "[1] \"language\"\n",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_Rscript"))
            .args(["--vanilla", "-e", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(status));
        assert_eq!(output.stdout, expected.as_bytes());
        if status == 1 {
            assert_eq!(
                output.stderr,
                b"language printer non-ASCII text profile is not implemented yet\n"
            );
        } else {
            assert!(output.stderr.is_empty());
        }
    }
}
