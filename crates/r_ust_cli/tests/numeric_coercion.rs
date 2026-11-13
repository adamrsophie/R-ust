use std::process::{Command, Output};

const BINARIES: [&str; 2] = [
    env!("CARGO_BIN_EXE_r_ust_cli"),
    env!("CARGO_BIN_EXE_Rscript"),
];
const GENERIC: &str = "NAs introduced by coercion";
const RANGE: &str = "NAs introduced by coercion to integer range";

fn run(binary: &str, source: &str, locale: &str) -> Output {
    Command::new(binary)
        .args(["--vanilla", "--no-echo", "-e", source])
        .env("LC_ALL", locale)
        .env("LANG", locale)
        .output()
        .unwrap()
}

#[test]
fn deferred_and_immediate_coercion_warnings_reach_the_correct_output_boundary() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            for (warn, expected) in [
                (-1, "BEFORE\n[1] NA\nAFTER\n".to_string()),
                (
                    0,
                    format!("BEFORE\n[1] NA\nWarning message:\n{GENERIC} \nAFTER\n"),
                ),
                (1, format!("BEFORE\nWarning: {GENERIC}\n[1] NA\nAFTER\n")),
            ] {
                let source = format!(
                    "options(warn={warn});cat('BEFORE\\n');as.double('bad');cat('AFTER\\n')"
                );
                let output = Command::new("/bin/sh")
                    .args([
                        "-c",
                        "exec \"$1\" --vanilla --no-echo -e \"$2\" 2>&1",
                        "numeric-coercion",
                        binary,
                        &source,
                    ])
                    .env("LC_ALL", locale)
                    .env("LANG", locale)
                    .output()
                    .unwrap();
                assert!(output.status.success(), "{binary}; {locale}; {output:?}");
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    expected,
                    "{binary}; {locale}; warn={warn}"
                );
                assert!(output.stderr.is_empty());
            }
        }
    }
}

#[test]
fn promotion_and_locale_cases_match_r() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            for warn in [2, 3] {
                let output = run(
                    binary,
                    &format!(
                        "options(warn={warn});as.integer(list('2147483648','bad'));cat('AFTER')"
                    ),
                    locale,
                );
                assert!(!output.status.success(), "{binary}; {locale}");
                assert!(output.stdout.is_empty());
                assert!(String::from_utf8(output.stderr)
                    .unwrap()
                    .contains(&format!("(converted from warning) {RANGE}")));
            }
            let output = run(
                binary,
                "options(warn=-1);as.double(c('1\\u2003','1\\u00a0','1\\u2007','1\\u202f'))",
                locale,
            );
            assert!(output.status.success());
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                if locale == "C" {
                    "[1] NA NA NA NA\n"
                } else {
                    "[1]  1 NA NA NA\n"
                }
            );
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn atomic_and_list_warning_sequences_survive_both_frontends() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            for (source, warnings) in [
                (
                    "as.integer(c('2147483648','bad','bad'))",
                    vec![GENERIC, RANGE],
                ),
                (
                    "as.integer(list('2147483648','bad','bad'))",
                    vec![RANGE, GENERIC, GENERIC],
                ),
            ] {
                let output = run(binary, &format!("options(warn=1);{source}"), locale);
                assert!(output.status.success());
                assert_eq!(output.stdout, b"[1] NA NA NA\n");
                let expected = warnings
                    .iter()
                    .map(|warning| format!("Warning: {warning}\n"))
                    .collect::<String>();
                assert_eq!(
                    String::from_utf8(output.stderr).unwrap(),
                    expected,
                    "{binary}; {locale}"
                );
            }
        }
    }
}

#[test]
fn malformed_lists_fail_without_warning_or_later_output() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            let output = run(
                binary,
                "options(warn=2);as.double(list('bad',NULL));cat('AFTER')",
                locale,
            );
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            let error = String::from_utf8(output.stderr).unwrap();
            assert!(
                error.contains("'list' object cannot be coerced to type 'double'"),
                "{error}"
            );
            assert!(!error.contains(GENERIC), "{error}");
        }
    }
}

#[test]
fn missing_invalid_and_long_strings_preserve_values_and_diagnostics() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            let source = format!(
                "options(warn=1);as.double(c(NA_character_,'','bad','{}1'))",
                "0".repeat(2048)
            );
            let output = run(binary, &source, locale);
            assert!(output.status.success(), "{binary}; {locale}; {output:?}");
            assert_eq!(output.stdout, b"[1] NA NA NA  1\n");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!("Warning: {GENERIC}\n")
            );
        }
    }
}

#[test]
fn scalar_option_and_quit_tails_are_silent_even_when_warnings_promote() {
    for binary in BINARIES {
        for locale in ["C", "en_US.UTF-8"] {
            let output = run(binary, "options(warn=2);options(digits=c('0x7','bad'));cat(getOption('digits'));q(status=c('0','bad'),save='no',runLast=FALSE);cat('AFTER')", locale);
            assert!(output.status.success(), "{binary}; {locale}; {output:?}");
            assert_eq!(output.stdout, b"7");
            assert!(output.stderr.is_empty());
        }
    }
}
