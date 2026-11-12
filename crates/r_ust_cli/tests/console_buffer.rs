use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--quiet", "--no-save"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn expected_echo(source: &str, value: &str) -> Vec<u8> {
    let mut expected = b"> ".to_vec();
    expected.extend_from_slice(&source.as_bytes()[..4095]);
    expected.extend_from_slice(b"+ ");
    expected.extend_from_slice(&source.as_bytes()[4095..]);
    expected.extend_from_slice(format!("[1] {value}\n> \n").as_bytes());
    expected
}

#[test]
fn long_literals_echo_each_console_chunk_without_changing_value() {
    let source = format!("nchar(\"{}\")\n", "x".repeat(5000));
    let output = run(&source);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stdout == expected_echo(&source, "5000"),
        "chunk echo differs"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn console_echo_preserves_utf8_bytes_split_by_a_chunk_boundary() {
    let source = format!("nchar(\"{}αxx\")\n", "x".repeat(4087));
    let output = run(&source);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stdout == expected_echo(&source, "4090"),
        "raw UTF-8 echo differs"
    );
    assert!(String::from_utf8(output.stdout).is_err());
    assert!(output.stderr.is_empty());
}

#[test]
fn exactly_full_unterminated_buffer_requires_more_input() {
    let source = format!("{}1", " ".repeat(4094));
    let output = run(&source);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.ends_with(b"1+ "));
    assert!(!output.stdout.windows(3).any(|part| part == b"[1]"));
    assert!(!output.stderr.is_empty());
}

#[test]
fn full_whitespace_chunk_finishes_without_an_incomplete_statement() {
    let output = run(&" ".repeat(4095));
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.ends_with(b"+ \n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn raw_string_opener_at_buffer_end_is_an_error() {
    let source = format!("{}r\"--(v)--\"\n", " ".repeat(4091));
    let output = run(&source);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.ends_with(b"r\"--"));
    assert!(!output.stderr.is_empty());
}

#[test]
fn string_capture_rejects_non_utf8_echo_without_lossy_conversion() {
    let source = format!("nchar(\"{}αxx\")", "x".repeat(4087));
    let config = r_ust_cli::CliConfig::from_args(["--vanilla", "--quiet", "-e", &source]).unwrap();
    assert!(r_ust_cli::run_to_string(config)
        .unwrap_err()
        .contains("console output is not UTF-8"));
}

#[test]
fn string_capture_without_echo_keeps_split_utf8_source_valid() {
    let source = format!("nchar(\"{}αxx\")", "x".repeat(4087));
    let config =
        r_ust_cli::CliConfig::from_args(["--vanilla", "--no-echo", "-e", &source]).unwrap();
    let output = r_ust_cli::run_to_string(config).unwrap();
    assert_eq!(output, "[1] 4090\n");
}
