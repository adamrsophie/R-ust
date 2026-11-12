use std::io::Write;
use std::process::{Command, Output, Stdio};

use r_ust_cli::{run_to_string, CliConfig, CliMode};

fn run(binary: &str, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn omitted_expressions_preserve_invocation_and_read_noninteractive_stdin() {
    let source = format!("#{}{}", "x".repeat(1120), "é".repeat(5000));
    let mut expected =
        format!("WARNING: '-e {source}' omitted as input is too long\n").into_bytes();
    expected.truncate(1151);
    expected.push(b'\n');
    assert!(std::str::from_utf8(&expected).is_err());
    expected.extend_from_slice(b"6121:FALSE:literal");
    for (binary, rscript) in [
        (env!("CARGO_BIN_EXE_r_ust_cli"), false),
        (env!("CARGO_BIN_EXE_Rscript"), true),
    ] {
        let mut args = vec!["--no-echo", "-e", &source];
        if !rscript {
            args.push("--args");
        }
        args.push("literal");
        let output = run(binary, &args,
            "a<-commandArgs(FALSE);cat(nchar(a[length(a)-2]),':',interactive(),':',commandArgs(TRUE),sep='')\n");
        assert!(output.status.success(), "{binary}: {output:?}");
        assert_eq!(output.stdout, expected, "{binary}");
        assert!(output.stderr.is_empty(), "{binary}: {output:?}");
    }
}

#[test]
fn full_buffer_omits_later_expression_without_reading_stdin() {
    let source = format!("#{}", "x".repeat(9997));
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        let output = run(
            binary,
            &["--no-echo", "-e", &source, "-e", "cat('NEVER')"],
            "cat('STDIN')\n",
        );
        assert!(output.status.success(), "{binary}: {output:?}");
        assert_eq!(
            output.stdout, b"WARNING: '-e cat('NEVER')' omitted as input is too long\n\n",
            "{binary}"
        );
        assert!(output.stderr.is_empty(), "{binary}: {output:?}");
    }
}

#[test]
fn admission_counts_decoded_prior_source_and_encoded_new_argument() {
    let first = format!("#{}", " ".repeat(3332));
    let second = format!("#{}", " ".repeat(2221));
    let config =
        CliConfig::from_args(["--no-echo", "-e", &first, "-e", &second, "-e", "cat('A')"]).unwrap();
    assert!(config.startup_stdout.is_empty());
    assert_eq!(
        config.mode,
        CliMode::Expression(format!("{first}\n{second}\ncat('A')"))
    );
}

#[test]
fn string_capture_preserves_startup_text_and_rejects_non_utf8_output() {
    let source = format!("#{}", "x".repeat(10000));
    let config = CliConfig::from_args(["--no-echo", "-e", &source, "-e", "cat('OK')"]).unwrap();
    let mut expected = String::from_utf8(config.startup_stdout.clone()).unwrap();
    expected.push_str("OK");
    assert_eq!(run_to_string(config).unwrap(), expected);
    let source = format!("#{}{}", "x".repeat(1120), "é".repeat(5000));
    let config = CliConfig::from_args(["--no-echo", "-e", &source, "-e", "cat('OK')"]).unwrap();
    assert!(run_to_string(config)
        .unwrap_err()
        .contains("startup output is not UTF-8"));
}

#[test]
fn omitted_expressions_can_accompany_an_explicit_file_in_either_order() {
    let source = format!("#{}", "x".repeat(10000));
    for args in [
        ["--file", "example.R", "-e", &source],
        ["-e", &source, "--file", "example.R"],
    ] {
        let config = CliConfig::from_args(args).unwrap();
        assert_eq!(config.mode, CliMode::File("example.R".into()));
        assert!(!config.startup_stdout.is_empty());
    }
}
