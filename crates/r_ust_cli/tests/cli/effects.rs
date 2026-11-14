use super::batch_command;
use super::run_stdin;

#[test]
fn quit_unwinds_nested_calls_and_preserves_exit_status() {
    for (source, status, stdout) in [
        ("print(1); q(); print(2)", 0, "[1] 1\n"),
        ("quit(\"no\", 7); print(2)", 7, ""),
        (
            "f<-function(){for(i in 1:3)q(\"no\",status=3);99}; f();99",
            3,
            "",
        ),
        ("lapply(\"no\",quit);99", 0, ""),
        ("(function(x)x)(quit(\"no\",status=5));99", 5, ""),
        ("quit(\"no\",st=6,run=FALSE)", 6, ""),
        ("quit(status=4,\"no\")", 4, ""),
        ("quit(save=\"no\",s=2)", 2, ""),
        ("quit(\"no\",status=257)", 1, ""),
        ("quit(\"no\",status=-1)", 255, ""),
        (".Last<-function()print(8);q(\"no\")", 0, "[1] 8\n"),
        (".Last<-function()print(8);q(\"no\",runLast=FALSE)", 0, ""),
        (".Last<-function()q(\"no\",4,FALSE);q(\"no\",3)", 4, ""),
        (".Last<-1;q(\"no\")", 0, ""),
    ] {
        let output = batch_command().args(["-e", source]).output().unwrap();
        assert_eq!(output.status.code(), Some(status), "{source}: {:?}", output);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            stdout,
            "{source}"
        );
        assert!(output.stderr.is_empty(), "{source}: {:?}", output.stderr);
    }
}

#[test]
fn quit_validates_arguments_before_forcing_and_rejects_workspace_saving() {
    for source in [
        "q(bogus=print(9))",
        "q(\"no\",0,FALSE,print(9))",
        "q(s=\"no\")",
        "q(save=\"no\",sa=print(9))",
        "q(save=\"no\",save=\"no\")",
        "q(\"bad\")",
        "q(1)",
        "q(\"yes\")",
    ] {
        let output = batch_command().args(["-e", source]).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(output.stdout.is_empty(), "{source}: {:?}", output.stdout);
        assert!(!output.stderr.is_empty(), "{source}");
    }
    let output = run_stdin(".Last<-function()print(8)\n", &[]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 8\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn expression_mode_tracks_visibility_and_console_effects() {
    for (source, expected_stdout, expected_stderr) in [
        ("NULL; {}; if(FALSE) 1", "NULL\nNULL\n", ""),
        ("x<-1; (x<-2); x", "[1] 2\n[1] 2\n", ""),
        ("f<-function(){x<-3}; f(); (f())", "[1] 3\n", ""),
        ("for(i in 1:2) print(i)", "[1] 1\n[1] 2\n", ""),
        (
            "f<-function(){print(1);invisible(2)}; f(); (f())",
            "[1] 1\n[1] 1\n[1] 2\n",
            "",
        ),
        ("print(NULL)", "NULL\n", ""),
        ("withVisible(invisible(1))$visible", "[1] FALSE\n", ""),
        ("withVisible(print(1))$visible", "[1] 1\n[1] FALSE\n", ""),
        (
            "(function(x)x)(invisible(1)); (function(x){x;x})(invisible(2))",
            "[1] 2\n",
            "",
        ),
        ("cat(\"a\",\"b\",sep=\"-\"); print(3)", "a-b[1] 3\n", ""),
        ("writeLines(c(\"a\",\"b\"),sep=\"|\")", "a|b|", ""),
        ("message(\"a\",1)", "", "a1\n"),
        ("message(\"a\\n\")", "", "a\n\n"),
        (
            "set.seed(1); withVisible(set.seed(2))$visible",
            "[1] FALSE\n",
            "",
        ),
        ("print(\"x\",quote=FALSE)", "[1] x\n", ""),
        ("a<-1:3; (a[2]<-9); a", "[1] 9\n[1] 1 9 3\n", ""),
        ("invisible(lapply(1:2,print))", "[1] 1\n[1] 2\n", ""),
    ] {
        let output = batch_command().args(["-e", source]).output().unwrap();
        assert!(
            output.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected_stdout,
            "{source}"
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            expected_stderr,
            "{source}"
        );
    }
}

#[test]
fn expression_mode_preserves_output_before_error() {
    let output = batch_command()
        .args(["-e", "print(1); missing_value"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[1] 1\n");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("object 'missing_value' not found"));
}

#[test]
fn expression_mode_flushes_output_during_execution() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;
    let mut child = batch_command()
        .args(["-e", "print(\"ready\"); repeat {}"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
        let _ = sender.send(result);
    });
    let line = receiver.recv_timeout(Duration::from_secs(5));
    let _ = child.kill();
    child.wait().unwrap();
    reader.join().unwrap();
    assert_eq!(line.unwrap().unwrap(), "[1] \"ready\"\n");
}
