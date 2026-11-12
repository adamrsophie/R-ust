use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn batch_input_echoes_lines_continuations_and_eof() {
    let output = run(&["--quiet", "--no-save", "-e", "1+\n2\n"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "> 1+\n+ 2\n[1] 3\n> \n> \n"
    );
}

#[test]
fn no_echo_aliases_set_quiet_and_keep_values_visible() {
    for flag in ["--no-echo", "-s", "--slave"] {
        let output = run(&[
            flag,
            "-e",
            "cat(getOption('echo'),getOption('quiet'),sep=':');7",
        ]);
        assert!(output.status.success(), "{flag}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "FALSE:TRUE[1] 7\n"
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn echo_changes_apply_to_next_input_read_and_na_enables_echo() {
    let output = run(&[
        "--no-echo",
        "-e",
        "options(echo=TRUE)\n1+\n2\noptions(echo=FALSE)\n3\n",
    ]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "> 1+\n+ 2\n[1] 3\n> options(echo=FALSE)\n[1] 3\n"
    );
    let output = run(&["--no-echo", "-e", "options(echo=NA);1"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 1\n> \n");
}

#[test]
fn echoed_input_stops_at_quit_or_error_without_reading_later_lines() {
    let output = run(&["--quiet", "-e", "print(1);q('no',status=3);@\n9\n"]);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "> print(1);q('no',status=3);@\n[1] 1\n"
    );
    let output = run(&["--quiet", "-e", "1\nmissingName\n9\n"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "> 1\n[1] 1\n> missingName\n"
    );
}

#[test]
fn stdin_echo_normalizes_crlf_and_terminates_last_line() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--quiet", "--no-save", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"# comment\r\n1+\r\n2")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "> # comment\n> 1+\n+ 2\n[1] 3\n> \n"
    );
}
