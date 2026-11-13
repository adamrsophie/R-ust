use std::process::{Command, Output, Stdio};

fn main_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--vanilla", "--no-echo"])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn unknown_option_warnings_follow_file_open_order_and_stop_at_args() {
    let missing = std::env::temp_dir().join(format!("r-ust-unknown-{}.R", std::process::id()));
    assert!(!missing.exists());
    let absent = missing.to_str().unwrap();
    let before = main_cli(&["--bad-option", "-f", absent]);
    let after = main_cli(&["-f", absent, "--bad-option"]);
    for output in [&before, &after] {
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stderr.is_empty());
    }
    assert!(before
        .stdout
        .starts_with(b"WARNING: unknown option '--bad-option'\n\nFatal error:"));
    assert!(after.stdout.starts_with(b"Fatal error:"));
    let output = main_cli(&[
        "-e",
        "writeLines(commandArgs(TRUE))",
        "--args",
        "--bad-option",
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"--bad-option\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn unknown_option_configuration_retains_warnings_and_original_arguments() {
    let config = r_ust_cli::CliConfig::from_args(["--no-echo", "--bad-option", "-e", "1"]).unwrap();
    assert_eq!(config.command_args[2], "--bad-option");
    assert_eq!(
        config.startup_stdout,
        b"WARNING: unknown option '--bad-option'\n\n"
    );
    assert_eq!(
        r_ust_cli::run_to_string(config).unwrap(),
        "WARNING: unknown option '--bad-option'\n\n[1] 1\n"
    );
    for args in [["--bad-option", "--help"], ["--bad-option", "--version"]] {
        let output = main_cli(&args);
        assert_eq!(output.status.code(), Some(0));
        assert!(!output.stdout.starts_with(b"WARNING:"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn unknown_option_truncation_preserves_raw_bytes() {
    let fill = 1150 - b"WARNING: unknown option '".len() - 2;
    let option = format!("--{}α", "x".repeat(fill));
    let output = main_cli(&[&option]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout.len(), 1152);
    assert!(output.stdout.ends_with(&[0xce, b'\n']));
    assert!(output.stderr.is_empty());
    assert!(String::from_utf8(output.stdout).is_err());
    let config = r_ust_cli::CliConfig::from_args(["--no-echo", &option]).unwrap();
    assert!(r_ust_cli::run_to_string(config)
        .unwrap_err()
        .contains("startup output is not UTF-8"));
}

#[test]
fn known_unimplemented_options_keep_explicit_errors() {
    for option in [
        "--save",
        "--restore",
        "--encoding=UTF-8",
        "--max-ppsize=10000",
        "--gui=Tk",
        "-psn_0_123",
    ] {
        let output = main_cli(&[option]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.starts_with(b"unsupported CLI option "));
    }
}

#[test]
fn rscript_warns_for_long_unknown_options_before_expressions() {
    for (args, expected) in [
        (
            vec!["--bogus", "-e", "print(9)"],
            b"WARNING: unknown option '--bogus'\n\n[1] 9\n".as_slice(),
        ),
        (
            vec!["--bad-option", "-e", "1"],
            b"WARNING: unknown option '--bad-option'\n\n[1] 1\n".as_slice(),
        ),
        (
            vec!["-e", "writeLines(commandArgs(TRUE))", "--bad-option"],
            b"--bad-option\n".as_slice(),
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_Rscript"))
            .arg("--vanilla")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn unknown_options_warn_and_continue() {
    let output = main_cli(&["--bad-option", "-e", "1"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"WARNING: unknown option '--bad-option'\n\n[1] 1\n"
    );
    assert!(output.stderr.is_empty());
}
