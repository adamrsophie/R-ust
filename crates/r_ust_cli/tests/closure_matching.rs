use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug)]
enum Mode {
    Expression,
    File,
    Stdin,
    RscriptFile,
    RscriptExpression,
}

const MODES: [Mode; 5] = [
    Mode::Expression,
    Mode::File,
    Mode::Stdin,
    Mode::RscriptFile,
    Mode::RscriptExpression,
];
static NEXT_SCRIPT: AtomicU64 = AtomicU64::new(0);

struct Script(PathBuf);

impl Script {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "r-ust-closure-{}-{}",
            std::process::id(),
            NEXT_SCRIPT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        let script = Self(directory);
        std::fs::write(script.0.join("closure source.R"), source).unwrap();
        script
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("cannot remove {}: {error}", self.0.display());
        }
    }
}

fn run(mode: Mode, source: &str, merged: bool) -> Output {
    let script = Script::new(source);
    let binary = match mode {
        Mode::RscriptFile | Mode::RscriptExpression => env!("CARGO_BIN_EXE_Rscript"),
        Mode::Expression | Mode::File | Mode::Stdin => env!("CARGO_BIN_EXE_r_ust_cli"),
    };
    let mut command = Command::new(binary);
    command.arg("--vanilla");
    match mode {
        Mode::Expression => {
            command.args(["--no-echo", "-e", &source.replace('\\', "\\\\")]);
        }
        Mode::RscriptExpression => {
            command.args(["-e", &source.replace('\\', "\\\\")]);
        }
        Mode::File => {
            command
                .args(["--no-echo", "-f"])
                .arg(script.0.join("closure source.R"));
        }
        Mode::RscriptFile => {
            command.arg(script.0.join("closure source.R"));
        }
        Mode::Stdin => {
            command.arg("--no-echo");
        }
    }
    if merged {
        let mut shell = Command::new("/bin/sh");
        shell
            .args(["-c", "exec \"$@\" 2>&1", "closure-matching"])
            .arg(binary)
            .args(command.get_args());
        command = shell;
    }
    let mut child = command
        .env("R_ENABLE_JIT", "0")
        .env("R_DEFAULT_PACKAGES", "NULL")
        .env("LC_ALL", "en_US.UTF-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(if matches!(mode, Mode::Stdin) {
            source.as_bytes()
        } else {
            b""
        })
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn closure_matching_four_modes() {
    for mode in MODES {
        for (source, expected) in [
            (
                "f<-function(alpha,beta)c(alpha,beta);f(1,alpha=2)",
                "[1] 2 1\n",
            ),
            (
                "g<-function(x)x+x;f<-function(x)g(x);f({cat('ONCE\\n');3L})",
                "ONCE\n[1] 6\n",
            ),
            (
                "h<-function(){g<-function(alpha=11L)alpha;f<-function(...)g(...);f(unknown=return(42L),second=print(99));99L};h()",
                "[1] 42\n",
            ),
            (
                "h<-function(){g<-function(alpha=11L)alpha;f<-function(...)g(...);for(i in 1:3){f(unknown=break);cat('NO\\n')};42L};h()",
                "[1] 42\n",
            ),
            (
                "h<-function(){g<-function(alpha=11L)alpha;f<-function(...)g(...);for(i in 1:3){f(unknown=next);cat('NO\\n')};42L};h()",
                "[1] 42\n",
            ),
        ] {
            let output = run(mode, source, false);
            assert_eq!(output.status.code(), Some(0), "{mode:?}; {output:?}");
            assert_eq!(output.stdout, expected.as_bytes(), "{mode:?}; {source}");
            assert!(output.stderr.is_empty(), "{mode:?}; {output:?}");
        }
    }
}

#[test]
fn closure_matching_errors_four_modes() {
    for mode in MODES {
        for (source, stdout, stderr) in [
            (
                "options(warnPartialMatchArgs=TRUE,warn=1);f<-function(alpha){cat('BODY\\n');alpha};cat('BEFORE\\n');f(al={cat('ARG\\n');1},alpha=2,alpha=3);cat('AFTER\\n')",
                "BEFORE\n",
                "argument error: formal argument \"alpha\" matched by multiple actual arguments\n",
            ),
            (
                "g<-function()0;g(unused=print(9));cat('AFTER\\n')",
                "",
                "argument error: unused argument (unused = print(9))\n",
            ),
            (
                "g<-function(alpha=11L)alpha;f<-function(...)g(...);f(unknown=print(9));cat('AFTER\\n')",
                "[1] 9\n",
                "argument error: unused argument (unknown = 9)\n",
            ),
            (
                "g<-function()0;f<-function(...){x<-list(...);g(...)};f(unused={cat('ONCE\\n');9L});cat('AFTER\\n')",
                "ONCE\n",
                "argument error: unused argument (unused = 9)\n",
            ),
            (
                "f<-function(alpha=11L,beta=22L)list(alpha,beta);f(unknown={cat('UNUSED\\n');9L});cat('AFTER\\n')",
                "",
                "argument error: unused argument (unknown = {\n    cat(\"UNUSED\\n\")\n    9\n})\n",
            ),
            (
                "f<-function(alpha=11L,beta=22L)list(alpha=alpha,beta=beta);f(alpha=,al=);cat('AFTER\\n')",
                "",
                "argument error: unused argument (alist(al = ))\n",
            ),
            (
                "g<-function(alpha=11L)alpha;f<-function(...)g(...);f(unknown={cat('RETURN\\n');return(9L)});cat('AFTER\\n')",
                "RETURN\n",
                "control-flow error: no function to return from, jumping to top level\n",
            ),
            (
                "maker<-function(x)function()x;h<-function()maker(return(42L));p<-h();p();cat('AFTER\\n')",
                "",
                "control-flow error: no function to return from, jumping to top level\n",
            ),
        ] {
            let output = run(mode, source, false);
            assert_eq!(output.status.code(), Some(1), "{mode:?}; {output:?}");
            assert_eq!(output.stdout, stdout.as_bytes(), "{mode:?}; {source}");
            assert_eq!(output.stderr, stderr.as_bytes(), "{mode:?}; {source}");
        }
    }
}

#[test]
fn partial_warning_policy_preserves_merged_event_order() {
    let warning = "partial argument match of 'al' to 'alpha'";
    for mode in MODES {
        for (enabled, level, status, expected) in [
            (
                "FALSE",
                2,
                0,
                "BEFORE\nBODY\nARG\n[1] 2\nAFTER\n".to_string(),
            ),
            (
                "TRUE",
                -1,
                0,
                "BEFORE\nBODY\nARG\n[1] 2\nAFTER\n".to_string(),
            ),
            (
                "TRUE",
                0,
                0,
                format!("BEFORE\nBODY\nARG\n[1] 2\nWarning message:\n{warning} \nAFTER\n"),
            ),
            (
                "TRUE",
                1,
                0,
                format!("BEFORE\nWarning: {warning}\nBODY\nARG\n[1] 2\nAFTER\n"),
            ),
            (
                "TRUE",
                2,
                1,
                format!("BEFORE\ndomain error: (converted from warning) {warning}\n"),
            ),
        ] {
            let source = format!(
                "options(warnPartialMatchArgs={enabled},warn={level});f<-function(alpha){{cat('BODY\\n');alpha}};cat('BEFORE\\n');f(al={{cat('ARG\\n');2L}});cat('AFTER\\n')"
            );
            let output = run(mode, &source, true);
            assert_eq!(output.status.code(), Some(status), "{mode:?}; {source}");
            assert_eq!(output.stdout, expected.as_bytes(), "{mode:?}; {source}");
            assert!(output.stderr.is_empty(), "{mode:?}; {output:?}");
        }
    }
}
