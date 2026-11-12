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
fn launcher_validation_errors_use_status_one_and_stderr() {
    for (args, message) in [
        (
            vec!["-e"],
            "ERROR: option '-e' requires a non-empty argument\n",
        ),
        (
            vec!["-e", ""],
            "ERROR: option '-e' requires a non-empty argument\n",
        ),
        (
            vec!["-e", "\n"],
            "ERROR: option '-e' requires a non-empty argument\n",
        ),
        (
            vec!["-e", "-1"],
            "ERROR: option '-e' requires a non-empty argument\n",
        ),
        (
            vec!["-f"],
            "ERROR: option '-f' requires a filename argument\n",
        ),
        (
            vec!["-f", ""],
            "ERROR: option '-f' requires a filename argument\n",
        ),
        (
            vec!["-f", "-"],
            "ERROR: option '-f' requires a filename argument\n",
        ),
        (
            vec!["-f", "--no-echo"],
            "ERROR: option '-f' requires a filename argument\n",
        ),
        (
            vec!["-e", "1", "-f"],
            "ERROR: option '-f' requires a filename argument\n",
        ),
    ] {
        let output = main_cli(&args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        assert_eq!(output.stderr, message.as_bytes(), "{args:?}");
    }
}

#[test]
fn fatal_input_conflicts_use_status_two_and_stdout() {
    let path = std::env::temp_dir().join(format!("r-ust-startup-{}.R", std::process::id()));
    std::fs::write(&path, "cat('FILE')").unwrap();
    let file = path.to_str().unwrap();
    for args in [vec!["-f", file, "-e", "1"], vec!["-e", "1", "-f", file]] {
        let output = main_cli(&args);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            output.stdout,
            b"Fatal error: cannot use -e with -f or --file\n"
        );
        assert!(output.stderr.is_empty());
    }
    let output = main_cli(&["-f", file, "-e", "1", "-e"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn file_open_failures_and_directory_input_share_frontend_policy() {
    let missing = std::env::temp_dir().join(format!("r-ust-missing-{}.R", std::process::id()));
    assert!(!missing.exists());
    let directory = std::env::temp_dir();
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        let main = binary == env!("CARGO_BIN_EXE_r_ust_cli");
        for (path, expected_status) in [(missing.as_path(), 2), (directory.as_path(), 0)] {
            let mut command = Command::new(binary);
            command.args(["--vanilla", "--no-echo"]);
            if main {
                command.arg("-f");
            }
            let output = command.arg(path).stdin(Stdio::null()).output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(expected_status),
                "{binary}: {output:?}"
            );
            assert!(output.stderr.is_empty());
            if expected_status == 2 {
                assert!(output
                    .stdout
                    .starts_with(b"Fatal error: cannot open file '"));
            } else {
                assert!(output.stdout.is_empty());
            }
        }
    }
    let output = main_cli(&["--file="]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"Fatal error: cannot open file '': No such file or directory\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn shell_help_and_validation_precede_native_startup() {
    for args in [
        vec!["--help", "-e"],
        vec!["-e", "1", "--help", "-f"],
        vec!["-f", "/no/such/r-ust-file", "--help"],
        vec!["--version", "-f", "/no/such/r-ust-file"],
    ] {
        let output = main_cli(&args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
    for args in [vec!["-e", "-1", "--help"], vec!["--version", "-e"]] {
        let output = main_cli(&args);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn long_file_stdin_selection_can_accompany_expressions() {
    for args in [
        vec!["--file=-", "-e", "cat('EXPR')"],
        vec!["-e", "cat('EXPR')", "--file=-"],
    ] {
        let output = main_cli(&args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert_eq!(output.stdout, b"EXPR");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn library_configuration_and_capture_keep_string_errors() {
    let error: String = r_ust_cli::CliConfig::from_args(["-e"]).unwrap_err();
    assert_eq!(error, "ERROR: option '-e' requires a non-empty argument");
    let missing =
        std::env::temp_dir().join(format!("r-ust-capture-missing-{}.R", std::process::id()));
    assert!(!missing.exists());
    let config =
        r_ust_cli::CliConfig::from_args(["--no-echo", "-f", missing.to_str().unwrap()]).unwrap();
    assert!(r_ust_cli::run_to_string(config)
        .unwrap_err()
        .starts_with("failed to read "));
}

#[test]
fn unsupported_launcher_escapes_keep_status_two() {
    for expression in [r"cat('a\cb')", r"cat('a\xffb')"] {
        let output = main_cli(&["-e", expression]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.starts_with(b"-e launcher "));
    }
}
