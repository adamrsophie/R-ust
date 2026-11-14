use super::batch_command;
use super::run_stdin;

#[test]
fn expression_arguments_preserve_invocation_and_trailing_values() {
    let code = "writeLines(commandArgs());writeLines(commandArgs(TRUE));interactive()";
    let args = [
        "--vanilla",
        "-e",
        code,
        "--args",
        "α",
        "two words",
        "--flag",
        "",
        "--args",
    ];
    let output = batch_command().args(args).output().unwrap();
    assert!(output.status.success(), "{:?}", output);
    let expected = format!(
        "{}\n--no-echo\n{}\nα\ntwo words\n--flag\n\n--args\n[1] FALSE\n",
        env!("CARGO_BIN_EXE_r_ust_cli"),
        args.join("\n")
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert!(output.stderr.is_empty());
    let output = run_stdin(
        "writeLines(commandArgs(TRUE));interactive()\n",
        &["--args", "stdin-arg"],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "stdin-arg\n[1] FALSE\n"
    );
}

#[test]
fn repeated_expressions_share_state_and_continue_incomplete_syntax() {
    for (args, expected) in [
        (vec!["-e", "x<-2 # comment", "-e", "x+3"], "[1] 5\n"),
        (vec!["-e", "sum(", "-e", "1,2)"], "[1] 3\n"),
        (
            vec!["-e", "print(1)", "-e", "q(\"no\")", "-e", "@"],
            "[1] 1\n",
        ),
    ] {
        let output = batch_command().args(&args).output().unwrap();
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
    for args in [
        vec!["-e", "1", "-f", "script.R"],
        vec!["-f", "script.R", "-e", "1"],
        vec!["-f", "script.R", "argument"],
        vec!["--file="],
    ] {
        let output = batch_command().args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn script_arguments_are_not_parsed_as_options_or_other_files() {
    let path = std::env::temp_dir().join(format!("r-ust-session-{}.R", std::process::id()));
    std::fs::write(&path, "writeLines(commandArgs(TRUE));interactive()\n").unwrap();
    for args in [
        vec![
            "-f".into(),
            path.to_string_lossy().to_string(),
            "--args".into(),
            "one".into(),
            "--help".into(),
        ],
        vec![
            format!("--file={}", path.display()),
            "--args".into(),
            "one".into(),
            "--help".into(),
        ],
    ] {
        let output = batch_command().args(&args).output().unwrap();
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "one\n--help\n[1] FALSE\n"
        );
        assert!(output.stderr.is_empty());
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn statements_execute_before_later_parse_errors() {
    for (source, status, stdout) in [
        ("q(\"no\",status=3); @", 3, ""),
        ("print(1);q(\"no\");\"unfinished", 0, "[1] 1\n"),
        ("print(1); @", 1, "[1] 1\n"),
        ("print(1); sum(\n", 1, "[1] 1\n"),
        ("{print(1); @}", 1, ""),
        ("{q(\"no\",status=3); @}", 1, ""),
        ("if(TRUE) 1\nelse 2", 1, "[1] 1\n"),
        ("{if(TRUE) 1\nelse 2}", 0, "[1] 1\n"),
        ("{if(TRUE) print(1);else(2)}", 1, ""),
    ] {
        for output in [
            batch_command().args(["-e", source]).output().unwrap(),
            run_stdin(source, &["--file=-"]),
        ] {
            assert_eq!(output.status.code(), Some(status), "{source}: {:?}", output);
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                stdout,
                "{source}"
            );
        }
    }
    let output = batch_command()
        .args(["-e", "print(1);missing_value;@"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 1\n");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("object 'missing_value' not found"));
}

#[test]
fn statements_before_incomplete_input_are_not_repeated() {
    for (source, stdout) in [
        ("print(1);x<-2;sum(\nx,3)\n", "[1] 1\n[1] 5\n"),
        ("cat(\"é\");sum(\n2,3)\n", "é[1] 5\n"),
    ] {
        let output = run_stdin(source, &[]);
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn stdin_mode_preserves_state_and_reads_multiline_expressions() {
    let source = concat!(
        "x <- 2 # keep newline after comment\n",
        "f <- function(x,\n y = 3)\n{\n x +\n y\n}\n",
        "f(x)\n",
        "sum(\n1,\n2\n)\n",
        "(1\n+2)\n",
        "cat(\"done\\n\")\n",
    );
    for args in [&[][..], &["--file=-"][..]] {
        let output = run_stdin(source, args);
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "[1] 5\n[1] 3\n[1] 3\ndone\n"
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn stdin_mode_reports_errors_without_reading_later_commands() {
    for source in [
        "print(1)\nmissing_value\nprint(2)\n",
        "print(1)\n)\nprint(2)\n",
        "print(1)\nsum(\n",
    ] {
        let output = run_stdin(source, &[]);
        assert!(!output.status.success(), "{source}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 1\n");
        assert!(!output.stderr.is_empty());
    }
    let output = run_stdin("# empty session\n", &[]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn help_mode_explains_console_and_script_usage() {
    let output = batch_command().arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in ["Usage:", "-e", "-f", "stdin", "Ctrl-D"] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    assert!(output.stderr.is_empty());
}

#[test]
fn version_mode_reports_runtime_target() {
    let output = batch_command()
        .arg("--version")
        .output()
        .expect("run r_ust_cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("R-ust 0.1.0"));
    assert!(stdout.contains("R 4.6.0"));
}
