use std::process::{Command, Output, Stdio};

struct Files(std::path::PathBuf);

impl Drop for Files {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("cannot remove shell fixture {}: {error}", self.0.display());
        }
    }
}

fn main_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--vanilla", "--no-echo"])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn shell_validation_precedes_quoted_expression_expansion() {
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        for (source, expected) in [
            (r"\t", ""),
            (r"\x20", ""),
            (r"\040", ""),
            (r"\t-1", "[1] -1\n"),
            (r"\0-1", "[1] -1\n"),
            (r"1\n\cignored", "[1] 1\n"),
        ] {
            let output = Command::new(binary)
                .args(["--vanilla", "--no-echo", "-e", source])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(0),
                "{binary} {source:?}: {output:?}"
            );
            assert_eq!(output.stdout, expected.as_bytes(), "{binary} {source:?}");
            assert!(output.stderr.is_empty());
        }
    }
    let output = main_cli(&["-e", "n<-2", "-e", r"-n\n1"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"[1] -2\n[1] 1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn main_splits_unquoted_fields_and_reports_ignored_arguments() {
    for (args, expected) in [
        (
            vec!["--bad option", "-e", "cat('EXEC')"],
            "WARNING: unknown option '--bad'\n\nARGUMENT 'option' __ignored__\n\nEXEC",
        ),
        (
            vec!["--file", "ignored.R"],
            "WARNING: unknown option '--file'\n\nARGUMENT 'ignored.R' __ignored__\n\n",
        ),
        (
            vec!["--bad --help", "-e", "1"],
            "WARNING: unknown option '--bad'\n\nWARNING: unknown option '--help'\n\n[1] 1\n",
        ),
        (vec!["-"], "WARNING: unknown option '-'\n\n"),
        (
            vec!["-e", "1", "-"],
            "WARNING: unknown option '-'\n\n[1] 1\n",
        ),
        (vec!["--no-readline", "-e", "1"], "[1] 1\n"),
    ] {
        let output = main_cli(&args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn raw_help_and_native_version_use_different_precedence() {
    let help = main_cli(&["ignored.R", "--help", "-e"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(help.stdout.starts_with(b"Usage:"));
    assert!(help.stderr.is_empty());
    let version = main_cli(&["--bad --version", "-e", "1"]);
    assert_eq!(version.status.code(), Some(0));
    assert!(version.stdout.starts_with(b"R-ust"));
    assert!(version.stderr.is_empty());
}

#[test]
fn empty_raw_argument_stops_shell_encoding_without_dropping_tail() {
    let output = main_cli(&["", "-e", r#"cat("\\n")"#]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"ARGUMENT '' __ignored__\n\n\\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn filenames_expand_echo_escapes_before_final_field_splitting() {
    let root = std::env::temp_dir().join(format!("r-ust shell files {}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let files = Files(root);
    let file = files.0.join("part file");
    std::fs::write(&file, "cat('FILE')\n").unwrap();
    for binary in [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ] {
        for (suffix, expected) in [
            (r"\tignored", "ARGUMENT 'ignored' __ignored__\n\nFILE"),
            (r"\nignored", "ARGUMENT 'ignored' __ignored__\n\nFILE"),
            (r"\cignored", "FILE"),
        ] {
            let raw = format!("{}{suffix}", file.display());
            let mut command = Command::new(binary);
            command.args(["--vanilla", "--no-echo"]);
            if binary == env!("CARGO_BIN_EXE_r_ust_cli") {
                command.arg("-f");
            }
            let output = command.arg(&raw).stdin(Stdio::null()).output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(0),
                "{binary} {raw:?}: {output:?}"
            );
            assert_eq!(output.stdout, expected.as_bytes());
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn missing_field_after_filename_expansion_uses_compacted_argument_slots() {
    let output = main_cli(&["-f", r"\t"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"Fatal error: cannot open file '--no-echo': No such file or directory\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn malformed_argument_counts_never_execute_stale_source() {
    let root = std::env::temp_dir().join(format!("r-ust malformed args {}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let files = Files(root);
    std::fs::write(files.0.join("--no-echo"), "cat('UNINTENDED')\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_r_ust_cli"))
        .args(["--vanilla", "--no-echo", "-f", r"\t"])
        .current_dir(&files.0)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"Fatal error: missing filename after launcher expansion\n"
    );
    assert!(output.stderr.is_empty());
}
