#[path = "support/rscript.rs"]
mod support;
use support::{run, stdout, Script};

#[test]
fn expressions_share_state_and_stop_option_parsing_at_user_arguments() {
    for (args, expected) in [
        (
            vec!["--vanilla", "-e", "x<-2 # comment", "-e", "x+3"],
            "[1] 5\n",
        ),
        (vec!["-e", "sum(", "-e", "1,2)"], "[1] 3\n"),
        (
            vec![
                "-e",
                "writeLines(commandArgs(TRUE))",
                "--help",
                "-e",
                "print(99)",
            ],
            "--help\n-e\nprint(99)\n",
        ),
        (
            vec!["-e", "writeLines(commandArgs(TRUE))", "--args", "one", ""],
            "--args\none\n\n",
        ),
        (vec!["-e", "interactive()"], "[1] FALSE\n"),
    ] {
        assert_eq!(stdout(run(&args)), expected, "{args:?}");
    }
}

#[test]
fn command_args_uses_rscript_defaults_and_unix_launcher_encoding() {
    let code = "x <- 1\n\twriteLines(commandArgs()[-1])";
    let encoded = "x~+~<-~+~1~n~~t~writeLines(commandArgs()[-1])";
    let output = stdout(run(&[
        "--vanilla",
        "-e",
        code,
        "two words",
        "α",
        "--args",
        "",
    ]));
    assert_eq!(
        output,
        format!(
            "--no-echo\n--no-restore\n--vanilla\n-e\n{encoded}\n--args\ntwo words\nα\n--args\n\n"
        )
    );
    assert_eq!(
        stdout(run(&["-e", "writeLines(\"~n~+~ ~+~n~ α\")"])),
        "\n+~  n~ α\n"
    );
}

#[test]
fn file_arguments_remain_literal_and_file_paths_keep_spaces() {
    let script = Script::new("writeLines(commandArgs(TRUE));interactive()\n");
    let path = script.path.to_str().unwrap();
    assert_eq!(
        stdout(run(&[
            "--vanilla",
            path,
            "--help",
            "-e",
            "print(99)",
            "--args",
            ""
        ])),
        "--help\n-e\nprint(99)\n--args\n\n[1] FALSE\n"
    );
    std::fs::write(&script.path, "writeLines(commandArgs()[-1])\n").unwrap();
    assert_eq!(
        stdout(run(&[path, "tail"])),
        format!(
            "--no-echo\n--no-restore\n--file={}\n--args\ntail\n",
            path.replace(' ', "~+~")
        )
    );
}

#[test]
fn help_version_and_argument_errors_have_script_exit_statuses() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("Usage: Rscript"));
    assert!(stdout(run(&["--help"])).starts_with("Usage: Rscript"));
    assert!(stdout(run(&["--version"])).contains("Rscript (R-ust)"));
    let script = Script::new("");
    let absent = script.directory.join("missing.R");
    let output = run(&[absent.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("Fatal error: cannot open file"));
    assert!(output.stderr.is_empty());
    let output = run(&["--vanilla", ""]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("Fatal error: cannot open file"));
    for args in [
        vec!["--vanilla"],
        vec!["-e"],
        vec!["-e", ""],
        vec!["-e", "-1"],
        vec!["-e", " \t\n"],
        vec!["-e", "  -1"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for option in [
        "--save",
        "--restore",
        "--default-packages=NULL",
        "--default-packages=stats",
    ] {
        let output = run(&[option, "-e", "print(9)"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr)
            .unwrap()
            .contains("not implemented"));
    }
}
