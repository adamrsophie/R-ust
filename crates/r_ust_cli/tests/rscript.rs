use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_Rscript"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: Output) -> String {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

struct Script {
    directory: PathBuf,
    path: PathBuf,
}

impl Script {
    fn new(source: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("r-ust-rscript-{}-{unique}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("script with spaces.R");
        std::fs::write(&path, source).unwrap();
        Self { directory, path }
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.directory);
    }
}

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
fn file_echo_preserves_eof_and_trailing_blank_lines() {
    for (source, expected) in [
        ("options(echo=TRUE)\n1\n", "> 1\n[1] 1\n> \n"),
        ("options(echo=TRUE)\n1", "> 1\n[1] 1\n> \n"),
        ("options(echo=TRUE)\n# comment\n", "> # comment\n> \n"),
        ("options(echo=TRUE)\n\n", "> \n> \n"),
    ] {
        let script = Script::new(source);
        assert_eq!(
            stdout(run(&["--vanilla", script.path.to_str().unwrap()])),
            expected,
            "{source:?}"
        );
    }
}

fn run_file_bytes(source: &[u8]) -> Vec<Output> {
    let script = Script::new("");
    std::fs::write(&script.path, source).unwrap();
    [
        env!("CARGO_BIN_EXE_r_ust_cli"),
        env!("CARGO_BIN_EXE_Rscript"),
    ]
    .into_iter()
    .map(|binary| {
        let mut command = Command::new(binary);
        command.args(["--vanilla", "--no-echo"]);
        if binary == env!("CARGO_BIN_EXE_r_ust_cli") {
            command.arg("-f");
        }
        command
            .arg(&script.path)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    })
    .collect()
}

#[test]
fn file_quit_does_not_decode_unread_bytes() {
    for output in run_file_bytes(b"cat('BEFORE')\nq('no')\n\x80\n") {
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"BEFORE");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn file_read_error_preserves_output_from_earlier_lines() {
    for output in run_file_bytes(b"cat('BEFORE')\n\x80\n") {
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"BEFORE");
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn file_capture_stops_before_unread_bytes() {
    let script = Script::new("");
    std::fs::write(&script.path, b"cat('BEFORE')\nq('no')\n\x80\n").unwrap();
    let config = r_ust_cli::CliConfig::from_args([
        "--vanilla",
        "--no-echo",
        "-f",
        script.path.to_str().unwrap(),
    ])
    .unwrap();
    assert_eq!(r_ust_cli::run_to_string(config).unwrap(), "BEFORE");
}

#[test]
fn stdin_is_batch_input_with_state_and_continuations() {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_Rscript"))
        .args(["--vanilla", "-", "stdin-arg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"x<-2\nsum(\nx,3)\nwriteLines(commandArgs(TRUE))\ninteractive()\n")
        .unwrap();
    assert_eq!(
        stdout(child.wait_with_output().unwrap()),
        "[1] 5\nstdin-arg\n[1] FALSE\n"
    );
}

#[test]
fn quit_and_errors_use_shared_runtime_exit_handling() {
    for (code, status, expected) in [
        (
            ".Last<-function()print(8);q(\"no\",status=3);@",
            3,
            "[1] 8\n",
        ),
        (".Last<-function()print(8);1+1", 0, "[1] 2\n[1] 8\n"),
        ("print(1);missing_value;print(2)", 1, "[1] 1\n"),
        ("print(1);@", 1, "[1] 1\n"),
    ] {
        let output = run(&["-e", code]);
        assert_eq!(output.status.code(), Some(status), "{code}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected,
            "{code}"
        );
    }
}

#[test]
fn joined_shebang_options_and_verbose_mode_work() {
    assert_eq!(
        stdout(run(&["--vanilla\t--quiet", "-e", "interactive()"])),
        "[1] FALSE\n"
    );
    let output = run(&["--verbose", "--vanilla", "-e", "1+1"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 2\n");
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("R-ust Rscript arguments:") && diagnostic.contains("--no-restore"));
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

#[test]
fn executes_without_r_on_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_Rscript"))
        .args(["--vanilla", "-e", "sum(1:5)"])
        .env_clear()
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(stdout(output), "[1] 15\n");
}

#[cfg(unix)]
#[test]
fn executable_shebang_script_uses_native_frontend() {
    use std::os::unix::fs::PermissionsExt;
    let script =
        Script::new("#!/usr/bin/env Rscript\nwriteLines(commandArgs(TRUE));interactive()\n");
    std::fs::set_permissions(&script.path, std::fs::Permissions::from_mode(0o755)).unwrap();
    let binary_dir = std::path::Path::new(env!("CARGO_BIN_EXE_Rscript"))
        .parent()
        .unwrap();
    let path = std::env::join_paths(std::iter::once(binary_dir.to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();
    let output = Command::new(&script.path)
        .arg("from-shebang")
        .env("PATH", path)
        .output()
        .unwrap();
    assert_eq!(stdout(output), "from-shebang\n[1] FALSE\n");
}
